mod utils;
mod routes;
use actix_web::{web, App, HttpServer};
use rust_bert::pipelines::sentence_embeddings::{
    builder::SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use std::sync::{Arc, Mutex};
use tokio::task;


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
            .service(routes::upload_pdf)
            .service(routes::query_pdf)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
