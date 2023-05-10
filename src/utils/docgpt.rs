use actix_web::web;
use actix_web::web::Bytes;
use lazy_static::lazy_static;
use lopdf::Document;
use redis::Commands;
use redis::{Client, Connection};
use regex::Regex;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::{sync::Mutex, time::Instant};
use uuid::Uuid;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
    static ref REDIS_CLIENT: Client =
        Client::open(std::env::var("REDIS_URI").expect("REDIS_URI NOT SET!"))
            .expect("INVALID REDIS URI!");
    static ref REDIS_CONNECTION: Mutex<Connection> =
        Mutex::new(REDIS_CLIENT.get_connection().unwrap());
}

fn preprocess(text: String) -> String {
    RE_SPACE
        .replace_all(&RE_NL.replace_all(&text, ""), " ")
        .to_string()
}

pub fn chunk(pdf: Bytes) -> String {
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
        chunks.append(&mut chunk);
    }
    let mut redis_connection = REDIS_CONNECTION.lock().unwrap();
    let key = Uuid::new_v4().to_string();
    let _: () = redis_connection.rpush(key.as_str(), chunks).unwrap();
    key
}

pub async fn query(
    id: &str,
    _question: &str,
    model: web::Data<Mutex<SentenceEmbeddingsModel>>,
) -> String {
    let mut redis_connection = REDIS_CONNECTION.lock().unwrap();
    let chunks: Vec<String> = redis_connection.lrange(id, 0, -1).unwrap();
    let model = model.lock().unwrap();
    let now = Instant::now();
    model.encode(&chunks).unwrap();
    format!("Time taken: {:?}", now.elapsed())
}
