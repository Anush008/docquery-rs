use crate::utils::{
    data::Query,
    pdfquery::{chunk, query, clear, store_jpg},
};
use actix_web::{post, delete, web, HttpRequest, Responder};
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::sync::{Arc, Mutex};

#[post("/pdf")]
async fn upload_pdf(request: HttpRequest, pdf: web::Bytes) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    chunk(pdf, model)
}

#[post("/jpg")]
async fn upload_jpg(jpg: web::Bytes) -> impl Responder {
    store_jpg(jpg).await
}


#[post("/query")]
async fn query_pdf(request: HttpRequest, data: web::Json<Query>) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    query(&data.id, &data.question, model).await
}

#[delete("/clear")]
async fn clear_pdfs() -> impl Responder {
    let _ = clear();
    "Deleted"
}