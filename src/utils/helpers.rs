use crossbeam::channel::bounded;
use crossbeam::scope;
use lazy_static::lazy_static;
use ndarray::ArrayView1;
use openai_api_rs::v1::{
    api::Client,
    chat_completion::{self, ChatCompletionRequest},
};
use rayon::prelude::*;
use regex::Regex;
use rust_bert::pipelines::sentence_embeddings::{
    builder::SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};
use std::{env, sync::Arc};

use super::data::CustomPool;

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
                content: Some(query.to_string()),
                name: None,
                function_call: None,
            },
            chat_completion::ChatCompletionMessage {
                role: chat_completion::MessageRole::user,
                content: Some(content),
                name: None,
                function_call: None,
            },
        ],
        function_call: None,
        functions: None
    };
    let result = OPENAI_CLIENT.chat_completion(req).await?;
   result.choices[0].message.content.clone().ok_or("No response!".into())
}

pub fn embed(strings: &Vec<String>, pool: &Arc<CustomPool<SentenceEmbeddingsModel>>) -> Vec<Vec<f32>> {
    let mut embeddings_store = vec![vec![0.0; 384]; strings.len()];
    let (sender, receiver) = bounded(strings.len());

    scope(|s| {
        strings
            .into_par_iter()
            .enumerate()
            .for_each(|(index, string)| {
                let pool = pool.clone();
                let sender = sender.clone();
                s.spawn(move |_| {
                    let model: SentenceEmbeddingsModel = pool.pull();
                    let mut embedding: Vec<Vec<f32>> = model.encode(&[&string]).unwrap();
                    pool.push(model);
                    sender.send((index, embedding.pop().unwrap())).unwrap();
                });
            });
        (0..strings.len()).for_each(|_| {
            let (index, embedding) = receiver.recv().unwrap();
            embeddings_store[index] = embedding;
        });
    })
    .unwrap();

    embeddings_store
}

pub fn create_embedding_model() -> SentenceEmbeddingsModel {
    SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
        .create_model()
        .expect("Embedding model instantiation failed!")
}

