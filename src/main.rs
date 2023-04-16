use actix_web::{post, web, App, HttpServer, Responder};
mod utils;

#[post("/pdf")]
async fn upload_pdf(pdf: web::Bytes) -> impl Responder {
    let chunks = utils::chunker::chunk(pdf);
    chunks.len().to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(upload_pdf)
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 10))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
