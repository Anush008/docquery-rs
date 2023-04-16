use actix_web::web::Bytes;
use lazy_static::lazy_static;
use lopdf::Document;
use regex::Regex;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").unwrap();
    static ref RE_SPACE: Regex = Regex::new(r"\s+").unwrap();
}

fn preprocess(text: String) -> String {
    RE_SPACE
        .replace_all(&RE_NL.replace_all(&text, ""), " ")
        .to_string()
}

pub fn chunk(pdf: Bytes) -> Vec<String> {
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
    chunks
}
