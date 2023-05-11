#![allow(unused_mut)]

use actix_web::web::{self, Bytes};
use lazy_static::lazy_static;
use lopdf::Document;
use regex::Regex;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::{sync::Mutex, collections::HashMap};
use uuid::Uuid;

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
    let doc = Document::load_mem(&pdf.to_vec()).unwrap();
    let pages = doc.get_pages();
    let mut chunks: Vec<String> = Vec::new();
    for page_num in 1..=pages.len() {
        let text = doc.extract_text(&[page_num.try_into().unwrap()]).unwrap();
        let text = preprocess(text);
        let mut chunk: Vec<String> = text
            .chars()
            .collect::<Vec<char>>()
            .chunks(500)
            .map(|chunk| chunk.iter().collect::<String>())
            .map(|s| format!("[{page_num}] {s}"))
            .collect();
        chunk.push(format!("Total pages: {}", pages.len()));
        chunks.append(&mut chunk);
    }
    let key = Uuid::new_v4().to_string();
    let model = model.lock().unwrap();
    let embeddings = model.encode(&chunks).unwrap();
    let mut embeddings_collection = EMBEDDINGS_COLLECTION.lock().unwrap();
    embeddings_collection.insert(key.clone(), embeddings);
    let mut pdf_collection = PDF_COLLECTION.lock().unwrap();
    pdf_collection.insert(key.clone(), chunks);
    key
}

pub async fn query(
    id: &str,
    _question: &str,
    _model: web::Data<Mutex<SentenceEmbeddingsModel>>,
) -> String {
    let embeddings_collection = EMBEDDINGS_COLLECTION.lock().unwrap();
    let pdf_collection = PDF_COLLECTION.lock().unwrap();
    let embeddings = embeddings_collection.get(id).unwrap();
    let pdf = pdf_collection.get(id).unwrap();
    dbg!(
        embeddings
    );
    
    String::from("BOILERPLATE")
}
