mod utils;
use rust_bert::pipelines::sentence_embeddings::{ builder::SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType};
use tokio::task;
use std::sync::Arc;

use actix_web::{post, web, App, HttpServer, Responder};

#[post("/pdf")]
async fn upload_pdf(pdf: web::Bytes) -> impl Responder {
    utils::docgpt::chunk(pdf)
}

#[post("/query")]
async fn query_pdf(query: web::Json<utils::data::Query>) -> impl Responder {
    utils::docgpt::query(&query.id, &query.question).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _model = task::spawn_blocking(move || { 
        SentenceEmbeddingsBuilder::remote(
            SentenceEmbeddingsModelType::AllMiniLmL12V2
        ).create_model().unwrap()

    }).await.unwrap();
    
    HttpServer::new(|| {
        App::new()
            .service(upload_pdf)
            .service(query_pdf)
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 10))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
