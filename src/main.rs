mod routes;
mod utils;
use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::sync::{Arc, Mutex};
use tokio::task;

use crate::utils::helpers::create_embedding_model;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = task::spawn_blocking(move || {
        Arc::new(Mutex::new(create_embedding_model()))
    })
    .await?;
    println!("Pool created!");

    HttpServer::new(move || {
        App::new()
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 10))
            .app_data(pool.clone())
            .route("/", web::get().to(|| HttpResponse::Ok()))
            .service(routes::upload_pdf)
            .service(routes::query_pdf)
            .service(routes::clear_pdfs)
            .service(routes::upload_jpg)
            .service(fs::Files::new("/images", "./images"))
    })
    .bind(("0.0.0.0", 80))?
    .run()
    .await
}
