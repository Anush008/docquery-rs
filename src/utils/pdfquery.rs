#![allow(unused_mut)]

use actix_web::web::Bytes;
use image::ImageFormat;
use lazy_static::lazy_static;
use lopdf::Document;
use ndarray::ArrayView1;
use rayon::prelude::*;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::fs::File;
use std::io::Cursor;
use std::{
    collections::HashMap,
    io::Write,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use super::helpers::{ask_gpt, cosine_similarity, preprocess_text};

lazy_static! {
    static ref PDF_COLLECTION: Mutex<HashMap<String, Vec<String>>> = {
        let mut pdf_collection: HashMap<String, Vec<String>> = HashMap::new();
        Mutex::new(pdf_collection)
    };
    static ref EMBEDDINGS_COLLECTION: Mutex<HashMap<String, Vec<Vec<f32>>>> = {
        let mut embeddings_collection: HashMap<String, Vec<Vec<f32>>> = HashMap::new();
        Mutex::new(embeddings_collection)
    };
}

pub fn chunk(
    pdf: Bytes,
    pool: &Arc<Mutex<SentenceEmbeddingsModel>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let doc = Document::load_mem(&pdf.to_vec())?;
    let pages = doc.get_pages();
    if pages.len() > 200 {
        return Err("PDF is too large!".into());
    }
    let mut pages: Vec<String> = (1..=pages.len())
        .into_iter()
        .map(|page_num| {
            let text = doc.extract_text(&[page_num.try_into().unwrap()]).unwrap();
            format!("[{}] {}", page_num, preprocess_text(text))
        })
        .collect();
    pages.push(format!("[0] Total pages in the PDF - {}", pages.len()));
    let key = Uuid::new_v4().to_string();
    let model = pool.lock().expect("Mutex lock poisoned");
    let embeddings = model.encode(&pages).expect("Embedding failed");
    let mut embeddings_collection = EMBEDDINGS_COLLECTION.lock()?;
    embeddings_collection.insert(key.clone(), embeddings);
    let mut pdf_collection = PDF_COLLECTION.lock()?;
    pdf_collection.insert(key.clone(), pages);
    Ok(key)
}

pub async fn store_jpg(jpg: Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let image = image::load_from_memory(&jpg)?;

    let target_file_size = 3 * 1024 * 1024;
    let mut resized_image = image.resize_to_fill(1024, 1024, image::imageops::FilterType::Triangle);
    let mut buffer = Cursor::new(Vec::new());
    resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;
    while buffer.get_ref().len() > target_file_size {
        resized_image = resized_image.resize_to_fill(
            resized_image.width() / 2,
            resized_image.height() / 2,
            image::imageops::FilterType::Triangle,
        );
        buffer = Cursor::new(Vec::new());
        resized_image.write_to(&mut buffer, ImageFormat::Jpeg)?;
    }

    let key = Uuid::new_v4().to_string() + ".jpg";
    let path = format!("./images/{}", &key);
    let mut compressed_file = File::create(path)?;
    compressed_file.write_all(&buffer.into_inner())?;

    Ok(key)
}

pub async fn query(
    id: &str,
    question: &str,
    pool: &Arc<Mutex<SentenceEmbeddingsModel>>,
) -> Result<String, Box<dyn std::error::Error>> {
    let embeddings_collection = EMBEDDINGS_COLLECTION.lock()?;
    let pdf_collection = PDF_COLLECTION.lock()?;
    let pdf = pdf_collection.get(id).ok_or("Invalid PDF ID")?;
    let embeddings = embeddings_collection
        .get(id)
        .ok_or("Invalid Embeddings ID!")?;
    let model = pool.lock().expect("Model lock poisoned");
    let question_embedding = model.encode(&[question])?;
    drop(model);
    let similarities: Vec<f32> = embeddings
        .par_iter()
        .map(|embedding| {
            cosine_similarity(
                ArrayView1::from(&question_embedding[0]),
                ArrayView1::from(embedding),
            )
        })
        .collect();
    let mut indexed_vec: Vec<(usize, &f32)> = similarities.par_iter().enumerate().collect();
    indexed_vec.par_sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    let indices: Vec<usize> = indexed_vec.iter().map(|x| x.0).take(3).collect();
    let content = format!(
        "PDF chunks:\n {}\n{}\n{}\nUser-query: {}",
        &pdf[indices[0]], &pdf[indices[1]], &pdf[indices[2]], question
    );
    let response = ask_gpt(content).await;
    Ok(response?)
}

pub fn clear() -> Result<(), Box<dyn std::error::Error>> {
    let mut embeddings_collection = EMBEDDINGS_COLLECTION.lock()?;
    let mut pdf_collection = PDF_COLLECTION.lock()?;
    embeddings_collection.clear();
    pdf_collection.clear();
    Ok(())
}
