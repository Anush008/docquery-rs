mod utils;
use actix_web::{post, web, App, HttpRequest, HttpServer, Responder};
use rust_bert::pipelines::sentence_embeddings::{
    builder::SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::sync::{Arc, Mutex};
use tokio::task;

#[post("/pdf")]
async fn upload_pdf(request: HttpRequest, pdf: web::Bytes) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    utils::pdfquery::chunk(pdf, model)
}

#[post("/query")]
async fn query_pdf(request: HttpRequest, query: web::Json<utils::data::Query>) -> impl Responder {
    let model = request
        .app_data::<Arc<Mutex<SentenceEmbeddingsModel>>>()
        .expect("Model app_data failed to load!");
    utils::pdfquery::query(&query.id, &query.question, model).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let model = task::spawn_blocking(move || {
        Arc::new(Mutex::new(
            SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
                .create_model()
                .expect("Embedding model instantiation failed!"),
        ))
    })
    .await?;
    println!("Model loaded!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 10))
            .app_data(model.clone())
            .service(upload_pdf)
            .service(query_pdf)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
