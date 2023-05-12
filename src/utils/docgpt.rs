#![allow(unused_mut)]

use actix_web::web::{self, Bytes};
use lazy_static::lazy_static;
use lopdf::Document;
use ndarray::ArrayView1;
use regex::Regex;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::{collections::HashMap, sync::Mutex};
use uuid::Uuid;
use rayon::prelude::*;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").expect("Invalid regex");
    static ref RE_SPACE: Regex = Regex::new(r"\s+").expect("Invalid regex");
    static ref PDF_COLLECTION: Mutex<HashMap<String, Vec<String>>> = {
        let mut pdf_collection: HashMap<String, Vec<String>> = HashMap::new();
        Mutex::new(pdf_collection)
    };
    static ref EMBEDDINGS_COLLECTION: Mutex<HashMap<String, Vec<Vec<f32>>>> = {
        let mut embeddings_collection: HashMap<String, Vec<Vec<f32>>> = HashMap::new();
        Mutex::new(embeddings_collection)
    };
}

fn preprocess(text: String) -> String {
    RE_SPACE
        .replace_all(&RE_NL.replace_all(&text, ""), " ")
        .to_string()
}

pub fn chunk(pdf: Bytes, model: web::Data<Mutex<SentenceEmbeddingsModel>>) -> Result<String, Box<dyn std::error::Error>> {
    let model = model.lock().expect("Model lock is poisoned");
    let doc = Document::load_mem(&pdf.to_vec())?;
    let mut embeddings: Vec<Vec<f32>> = Vec::new();
    let mut chunks: Vec<String> = Vec::new();
    let pages = doc.get_pages();
    chunks.push(format!("[0] Total pages in the PDF - {}", pages.len()));
    embeddings.append(&mut model.encode(&[&chunks.first().ok_or("Page number embeddings failed to generate")?])?);
    for page_num in 1..=pages.len() {
        let text = doc.extract_text(&[page_num.try_into()?])?;
        let text = preprocess(text);
        let mut chunk: Vec<String> = text
            .chars()
            .collect::<Vec<char>>()
            .chunks(200)
            .map(|chunk| chunk.par_iter().collect::<String>())
            .map(|s: String| format!("[{page_num}] {s}"))
            .map(|s: String| {
                let mut embedding = model.encode(&[&s]).expect("Encoding failed");
                embeddings.append(&mut embedding);
                s
            })
            .collect();
        chunks.append(&mut chunk);
    }
    let key = Uuid::new_v4().to_string();
    let mut embeddings_collection = EMBEDDINGS_COLLECTION.lock()?;
    embeddings_collection.insert(key.clone(), embeddings);
    let mut pdf_collection = PDF_COLLECTION.lock()?;
    pdf_collection.insert(key.clone(), chunks);
    Ok(key)
}

pub fn query(id: &str, question: &str, model: web::Data<Mutex<SentenceEmbeddingsModel>>) -> Result<String, Box<dyn std::error::Error>> {
    let model = model.lock().expect("Model lock is poisoned");
    let embeddings_collection = EMBEDDINGS_COLLECTION.lock()?;
    let pdf_collection = PDF_COLLECTION.lock()?;
    let pdf = pdf_collection.get(id).ok_or("Invalid PDF ID")?;
    let embeddings = embeddings_collection.get(id).ok_or("Invalid Embeddings ID")?;
    let question_embedding = model.encode(&[question])?;
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
    let response = format!("{}\n{}\n{}", &pdf[indices[0]], &pdf[indices[1]], &pdf[indices[2]]);
    Ok(response)
}

fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot_product = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();
    dot_product / (norm_a * norm_b)
}
