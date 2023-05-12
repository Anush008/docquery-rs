use actix_web::{post, web, HttpRequest, Responder};
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::sync::{Arc, Mutex};
use crate::utils::{ pdfquery::{ chunk, query}, data::Query };

#[post("/pdf")]
async fn upload_pdf(request: HttpRequest, pdf: web::Bytes) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    chunk(pdf, model)
}

#[post("/query")]
async fn query_pdf(request: HttpRequest, data: web::Json<Query>) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    query(&data.id, &data.question, model).await
}