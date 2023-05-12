#![allow(unused_mut)]

use actix_web::web::{self, Bytes};
use lazy_static::lazy_static;
use lopdf::Document;
use regex::Regex;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::{sync::Mutex, collections::HashMap};
use uuid::Uuid;
use ndarray::ArrayView1;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
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

pub fn chunk(pdf: Bytes, model: web::Data<Mutex<SentenceEmbeddingsModel>>) -> String {
    let model = model.lock().unwrap();
    let doc = Document::load_mem(&pdf.to_vec()).unwrap();
    let mut embeddings: Vec<Vec<f32>> = Vec::new();
    let mut chunks: Vec<String> = Vec::new();
    let pages = doc.get_pages();
    chunks.push(format!("[0] Total pages in the PDF - {}", pages.len()));
    embeddings.append(&mut model.encode(&[&chunks.last().unwrap()]).unwrap());
    for page_num in 1..=pages.len() {
        let text = doc.extract_text(&[page_num.try_into().unwrap()]).unwrap();
        let text = preprocess(text);
        let mut chunk: Vec<String> = text
            .chars()
            .collect::<Vec<char>>()
            .chunks(150)
            .map(|chunk| chunk.iter().collect::<String>())
            .map(|s: String| format!("[{page_num}] {s}"))
            .map(|s: String| {
                let mut embedding = model.encode(&[&s]).unwrap();            
                embeddings.append(&mut embedding);
                s
            })
            .collect();
        chunks.append(&mut chunk);
    }
    let key = Uuid::new_v4().to_string();
    let mut embeddings_collection = EMBEDDINGS_COLLECTION.lock().unwrap();
    embeddings_collection.insert(key.clone(), embeddings);
    let mut pdf_collection = PDF_COLLECTION.lock().unwrap();
    pdf_collection.insert(key.clone(), chunks);
    key
}

pub fn query(
    id: &str,
    question: &str,
    model: web::Data<Mutex<SentenceEmbeddingsModel>>,
) -> String {
    let embeddings_collection = EMBEDDINGS_COLLECTION.lock().unwrap();
    let pdf_collection = PDF_COLLECTION.lock().unwrap();
    let embeddings = embeddings_collection.get(id).unwrap();
    let pdf = pdf_collection.get(id).unwrap();
    let model = model.lock().unwrap();
    let question_embedding = model.encode(&[question]).unwrap();
    let similarities: Vec<f32> = embeddings
    .iter()
    .map(|embedding| cosine_similarity(ArrayView1::from(&question_embedding[0]), ArrayView1::from(embedding)))
    .collect();
    let max_index = similarities.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(index, _)| index).unwrap();
    let similar_sentence = &pdf[max_index];
    similar_sentence.to_string()
}


fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot_product = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();
    dot_product / (norm_a * norm_b)
}