//! Measures model loading and suggestion latency: `cargo run --release -p dictgen --example load_bench`
use rynk_core::keyboard::state::Language;
use rynk_core::prediction::PredictionService;
use std::time::Instant;

fn rss_kb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS"))
                .map(|l| l.to_string())
        })
        .and_then(|l| l.split_whitespace().nth(1).and_then(|v| v.parse().ok()))
        .unwrap_or(0)
}

fn main() {
    let before = rss_kb();
    let t = Instant::now();
    let mut service = PredictionService::new(); // loads ru + en
    println!(
        "ru+en load: {:?}, RSS +{} MB",
        t.elapsed(),
        (rss_kb() - before) / 1024
    );

    let t = Instant::now();
    service.dictionary.ensure_language_loaded(Language::German);
    println!(
        "de load: {:?}, RSS total +{} MB",
        t.elapsed(),
        (rss_kb() - before) / 1024
    );

    let t = Instant::now();
    let _second = PredictionService::new();
    println!("second engine (cached models): {:?}", t.elapsed());

    for (input, last) in [
        ("прив", None),
        ("спасиб", None),
        ("", Some("доброе")),
        ("thnk", None),
        ("", Some("thank")),
    ] {
        let t = Instant::now();
        let lang = if input.is_ascii() && last.is_none_or(|l| l.is_ascii()) {
            Language::English
        } else {
            Language::Russian
        };
        let s = service.get_suggestions_for_lang(input, last, lang);
        println!("{:>8?} {input:?} after {last:?}: {s:?}", t.elapsed());
    }
}
