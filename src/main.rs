use actix_web::{post, web, App, HttpServer, Responder};
mod utils;


#[post("/pdf")]
async fn upload_pdf(pdf: web::Bytes) -> impl Responder {
    utils::docgpt::chunk(pdf)
}

#[post("/query")]
async fn query_pdf(query: web::Json<utils::data::Query>) -> impl Responder {
    utils::docgpt::query(&query.id, &query.question).await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
