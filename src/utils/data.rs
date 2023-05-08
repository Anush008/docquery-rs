#[derive(serde::Deserialize)]
pub struct Query {
    pub id: String,
    pub question: String
}