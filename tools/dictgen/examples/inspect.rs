//! Prints the head of a model file: `cargo run -p dictgen --example inspect -- ru.rlm [word]`
use rynk_core::prediction::lm_data::{ctx_id, dequantize_log10, LanguageModelData};

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("model path");
    let model = LanguageModelData::decode(&std::fs::read(&path).expect("read")).expect("valid model");
    println!("{} words, {} bigrams, {} trigrams", model.words.len(), model.bigrams.len(), model.trigrams.len());
    let top: Vec<String> = model.words.iter().take(40).map(|w| format!("{}:{}", w.word, w.freq)).collect();
    println!("top: {}", top.join(" "));
    let index = model.lowercase_index();
    for word in args {
        let Some(&id) = index.get(&word.to_lowercase()) else {
            println!("{word}: not in vocabulary");
            continue;
        };
        let e = &model.words[id as usize];
        let nexts: Vec<String> = model
            .bigrams
            .get(ctx_id(id) as u64)
            .iter()
            .take(10)
            .map(|&(n, q)| format!("{}({:.2})", model.words[n as usize].word, 10f32.powf(dequantize_log10(q))))
            .collect();
        println!("{} (id {id}, count {}, freq {}) -> {}", e.word, e.count, e.freq, nexts.join(" "));
    }
}
