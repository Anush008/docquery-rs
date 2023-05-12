use ndarray::ArrayView1;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_NL: Regex = Regex::new(r"\n").expect("Invalid regex!");
    static ref RE_SPACE: Regex = Regex::new(r"\s+").expect("Invalid regex!");
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