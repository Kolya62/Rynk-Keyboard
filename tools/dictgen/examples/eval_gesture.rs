//! Glide typing evaluation: swipes the words of real sentences with a noisy finger path and
//! measures how often the decoder's first (and top three) guesses are right.
//!
//! `cargo run --release -p dictgen --example eval_gesture -- en tools/dictgen/.cache/tatoeba/eng-sentences.txt`

use rynk_core::keyboard::key::KeyAction;
use rynk_core::keyboard::state::Language;
use rynk_core::keyboard::KeyboardEngine;
use rynk_core::prediction::gesture::resample;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

const SENTENCES: usize = 300;

struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }
    fn gauss(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-7);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let lang = Language::from_code(&args.next().expect("language code")).expect("known language");
    let path = args.next().expect("sentences file");
    let sentences: Vec<String> = BufReader::new(std::fs::File::open(&path).expect("open"))
        .lines()
        .map_while(Result::ok)
        .map(|l| l.rsplit_once('\t').map(|(_, t)| t.to_string()).unwrap_or(l))
        .filter(|s| s.split_whitespace().count() >= 3)
        .take(SENTENCES)
        .collect();

    let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
    engine.set_enabled_languages(vec![lang]);
    engine.set_language(lang);
    engine.set_editor_context("", false);
    let keys: HashMap<char, (f32, f32)> = engine
        .keys
        .iter()
        .filter_map(|k| match k.action {
            KeyAction::Character(c) if c.is_alphabetic() => Some((c.to_lowercase().next()?, k.center())),
            _ => None,
        })
        .collect();
    let key_w = engine.key_geometry.key_width;
    let mut rng = Rng(0x2545_F491_4F6C_DD1D);

    for sigma in [0.15f32, 0.3] {
        let (mut words, mut top1, mut top3) = (0, 0, 0);
        let mut times = Vec::new();
        for sentence in &sentences {
            let mut prefix = String::new();
            for raw in sentence.split_whitespace() {
                let target = raw.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                engine.set_editor_context(&prefix, false);
                prefix.push_str(raw);
                prefix.push(' ');
                if target.chars().count() < 2 || !target.chars().all(|c| keys.contains_key(&c)) {
                    continue;
                }
                // Anchors near each key, then a smooth path through them
                let anchors: Vec<(f32, f32)> = target
                    .chars()
                    .map(|c| {
                        let (x, y) = keys[&c];
                        (x + rng.gauss() * sigma * key_w, y + rng.gauss() * sigma * key_w)
                    })
                    .collect();
                let path = resample(&anchors, 25 * anchors.len());
                let t = std::time::Instant::now();
                let got = engine.prediction.dictionary.decode_gesture(
                    lang,
                    &engine.state.word_context(),
                    &path,
                    &engine.key_geometry,
                    3,
                );
                times.push(t.elapsed().as_micros() as u64);
                words += 1;
                let hits: Vec<String> = got.iter().map(|c| c.word.to_lowercase()).collect();
                top1 += (hits.first() == Some(&target)) as usize;
                top3 += hits.contains(&target) as usize;
            }
        }
        times.sort_unstable();
        let pct = |n: usize| 100.0 * n as f64 / words.max(1) as f64;
        println!(
            "noise {sigma:.2} key: words={words} top-1 {:.1}%  top-3 {:.1}%  decode p50 {}µs p95 {}µs",
            pct(top1),
            pct(top3),
            times[times.len() / 2],
            times[times.len() * 95 / 100]
        );
    }
}
