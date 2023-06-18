use crate::utils::{
    data::{Query, CustomPool},
    pdfquery::{chunk, clear, query, store_jpg},
};
use actix_web::{delete, post, web, HttpRequest, Responder};
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsModel;
use std::sync::Arc;

#[post("/pdf")]
async fn upload_pdf(request: HttpRequest, pdf: web::Bytes) -> impl Responder {
    let pool = request
        .app_data::<Arc<CustomPool<SentenceEmbeddingsModel>>>()
        .expect("Pool app_data failed to load!");
    chunk(pdf, &pool)
}

#[post("/jpg")]
async fn upload_jpg(jpg: web::Bytes) -> impl Responder {
    store_jpg(jpg).await
}

#[post("/query")]
async fn query_pdf(request: HttpRequest, data: web::Json<Query>) -> impl Responder {
    let pool = request
        .app_data::<Arc<CustomPool<SentenceEmbeddingsModel>>>()
        .expect("Pool app_data failed to load!");
    query(&data.id, &data.question, &pool).await
}

#[delete("/clear")]
async fn clear_pdfs() -> impl Responder {
    let _ = clear();
    "Deleted"
}
