use misaki_rs::{G2P, Language};
fn main() {
    let g2p = G2P::new(Language::EnglishUS);
    let (phonemes, tokens) = g2p.g2p("Hello Kokoro").unwrap();
    println!("Phonemes: {}", phonemes);
    println!("Tokens: {:?}", tokens);
}
