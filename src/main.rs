use actix_web::{web, App, HttpServer, Responder, post};
use lopdf::Document;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
}

fn preprocess(text: String) -> String {
    RE_SPACE
        .replace_all(&RE_NL.replace_all(&text, " "), " ")
        .to_string()
}

#[post("/pdf")]
async fn extract_pdf(pdf: web::Bytes) -> impl Responder {
    //TODO: Move to utils
    let doc = Document::load_mem(&pdf.to_vec()).unwrap();
    let pages  = doc.get_pages();
    let mut chunks: Vec<String> = Vec::new();
    for page_num in 1..=pages.len() {
        let text = doc.extract_text(&[page_num.try_into().unwrap()]).unwrap();
        let text = preprocess(text);
        let mut chunk:Vec<String> = text.chars().collect::<Vec<char>>().chunks(200)
        .map(|chunk| chunk.iter().collect::<String>()).map(|s| format!("[{page_num}]{s}")).collect();
        chunks.append(&mut chunk);
    };
    chunks.len().to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(extract_pdf)
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 50))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

