use actix_web::web::Bytes;
use lazy_static::lazy_static;
use lopdf::Document;
use redis::Commands;
use regex::Regex;
use std::{sync::Mutex, error};
use uuid::Uuid;
use redis::{Client, Connection};
use serde_json::json;
use actix_web::Result;
use reqwest::{header::HeaderMap};

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
    static ref REDIS_CLIENT:Client = Client::open(std::env::var("REDIS_URI").expect("REDIS_URI NOT SET!")).expect("INVALID REDIS URI!");
    static ref REDIS_CONNECTION:Mutex<Connection> = Mutex::new(REDIS_CLIENT.get_connection().unwrap());
    static ref REQWEST_CLIENT: Mutex<reqwest::Client> = Mutex::new(reqwest::Client::new());
    static ref TEXT_SIMILARITY_ENDPOINT: String = std::env::var("SIMILARITY_ENDPOINT").expect("SIMILARITY_ENDPOINT NOT SET!");
    static ref TEXT_SIMILARITY_TOKEN: String = std::env::var("SIMILARITY_TOKEN").expect("SIMILARITY_TOKEN NOT SET!");
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
            .chunks(200)
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

pub async fn query(id: &str, question: &str) -> Result<String, Box<dyn error::Error>> {
    let mut redis_connection = REDIS_CONNECTION.lock().unwrap();
    let chunks: Vec<String> = redis_connection.lrange(id, 0, -1).unwrap();

    let client = REQWEST_CLIENT.lock().unwrap();
    let mut headers = HeaderMap::new();
    headers.insert("Authorization", TEXT_SIMILARITY_TOKEN.parse().unwrap());
    let payload = json!({
        "source_sentence": question,
        "sentences": chunks
    });
    let response = client.post(TEXT_SIMILARITY_ENDPOINT.as_str()).json(&payload).headers(headers).send().await?;
    Ok(response.text().await?)
}