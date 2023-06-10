use lazy_static::lazy_static;
use ndarray::ArrayView1;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{self, ChatCompletionRequest},
};
use regex::Regex;
use std::env;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").expect("Invalid regex!");
    static ref RE_SPACE: Regex = Regex::new(r"\s+").expect("Invalid regex!");
    static ref OPENAI_CLIENT: Client =
        Client::new(env::var("OPENAI_API_KEY").expect("OpenAI client instantiation failed!"));
}

pub fn cosine_similarity(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let dot_product = a.dot(&b);
    let norm_a = a.dot(&a).sqrt();
    let norm_b = b.dot(&b).sqrt();
    dot_product / (norm_a * norm_b)
}

pub fn preprocess_text(text: String) -> String {
    RE_SPACE
        .replace_all(&RE_NL.replace_all(&text, ""), " ")
        .to_string()
}

pub async fn ask_gpt(content: String) -> Result<String, Box<dyn std::error::Error>> {
    let query = concat!("You are PDFQuery. An AI assistant for PDFs that can answer user-queries about user-uploaded PDFs.",
    "You generate a comprehensive, elaborate answer to the user-query using the given chunks of the PDF content obtained from semantic search. ",
    "You cite each reference using [ Page Number] notation (every PDF content has this number at the beginning). ",
    "Citation should be done at the end of each sentence. Only include information found in the PDF content. ",
    "You ignore any outlier PDF content which is unrelated to the query.");

    let req = ChatCompletionRequest {
        model: chat_completion::GPT3_5_TURBO.to_string(),
        messages: vec![
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::system,
                content: query.to_string(),
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content,
            },
        ],
    };
    let result = OPENAI_CLIENT.chat_completion(req).await?;
    Ok(result.choices[0].message.content.clone())
}
