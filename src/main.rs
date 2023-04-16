use actix_web::{web, App, HttpServer, Responder, post};
use lopdf::Document;

#[post("/pdf")]
async fn extract_pdf(pdf: web::Bytes) -> impl Responder {
    let doc = Document::load_mem(&pdf.to_vec()).unwrap();
    let pages  = doc.get_pages();
    let length = pages.len();

    length.to_string()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(extract_pdf)
            .app_data(web::PayloadConfig::default().limit(1024 * 1024 * 50))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

