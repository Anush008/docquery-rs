use actix_web::web::Bytes;
use lazy_static::lazy_static;
use lopdf::Document;
use redis::Commands;
use regex::Regex;
use std::sync::{Mutex};
use uuid::Uuid;
use redis::{Client, Connection};

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
    static ref REDIS_CLIENT:Client = Client::open(env!("DOCGPT_REDIS", "REDIS URI NOT SET IN THE DOCGPT_REDIS ENV!")).unwrap();
    static ref REDIS_CONNECTION:Mutex<Connection> = Mutex::new(REDIS_CLIENT.get_connection().unwrap());
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
