mod utils;
use rust_bert::pipelines::sentence_embeddings::{
    builder::SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::sync::Mutex;
use tokio::task;
use actix_web::{post, web, App, HttpServer, Responder};

#[post("/pdf")]
async fn upload_pdf(
    pdf: web::Bytes,
    model: web::Data<Mutex<SentenceEmbeddingsModel>>,
) -> impl Responder {
    utils::docgpt::chunk(pdf, model)
}

#[post("/query")]
async fn query_pdf(
    query: web::Json<utils::data::Query>,
    model: web::Data<Mutex<SentenceEmbeddingsModel>>,
) -> impl Responder {
    utils::docgpt::query(&query.id, &query.question, model).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let model = task::spawn_blocking(move || {
        SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()
            .unwrap()
    })
    .await?;
    println!("Model loaded!");
    let model = web::Data::new(Mutex::new(model));

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
