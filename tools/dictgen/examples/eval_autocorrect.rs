//! Autocorrect evaluation: types real sentences with simulated finger touches and measures what
//! the keyboard commits.
//!
//! `cargo run --release -p dictgen --example eval_autocorrect -- ru tools/dictgen/.cache/tatoeba/rus-sentences.txt [mild|normal|aggressive]`
//!
//! Each word is typed as touches around its key centers (Gaussian noise, so some touches land on
//! neighboring keys), plus occasional omitted / swapped / doubled letters, then space is pressed.
//! The text before the word is given to the engine as editor context, like on a device.

use rynk_core::keyboard::key::{KeyAction, KeyboardMode};
use rynk_core::keyboard::state::{KeyboardOutputEvent, Language};
use rynk_core::keyboard::touch::TouchAction;
use rynk_core::keyboard::KeyboardEngine;
use rynk_core::prediction::correction::AutocorrectStrength;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

const SENTENCES: usize = 400;

/// Deterministic xorshift RNG with Gaussian samples (Box-Muller)
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

struct Scenario {
    name: &'static str,
    /// Touch noise as a fraction of key width/height
    sigma: f32,
    /// Per-word probability of an omitted, swapped or doubled letter
    edit_rate: f32,
}

#[derive(Default)]
struct Stats {
    /// Words where a touch hit space or punctuation mid-word (a different problem than
    /// spelling correction; reported separately and excluded from the other numbers)
    split_by_space: usize,
    words: usize,
    typed_ok: usize,
    output_ok: usize,
    fixed: usize,
    broken: usize,
    /// Suggestion update time after each keystroke, µs
    keystroke_us: Vec<u64>,
}

fn main() {
    let mut args = std::env::args().skip(1);
    let lang = Language::from_code(&args.next().expect("language code")).expect("known language");
    let path = args.next().expect("sentences file");
    let strength = match args.next().as_deref() {
        Some("mild") => AutocorrectStrength::MILD,
        Some("aggressive") => AutocorrectStrength::AGGRESSIVE,
        _ => AutocorrectStrength::NORMAL,
    };
    let sentences: Vec<String> = BufReader::new(std::fs::File::open(&path).expect("open sentences"))
        .lines()
        .map_while(Result::ok)
        .map(|l| l.rsplit_once('\t').map(|(_, t)| t.to_string()).unwrap_or(l))
        .filter(|s| s.split_whitespace().count() >= 3)
        .take(SENTENCES)
        .collect();

    for scenario in [
        Scenario { name: "precise typing", sigma: 0.12, edit_rate: 0.0 },
        Scenario { name: "sloppy typing", sigma: 0.32, edit_rate: 0.06 },
    ] {
        let mut stats = run(lang, &sentences, &scenario, strength);
        stats.keystroke_us.sort_unstable();
        let q = |p: f64| stats.keystroke_us.get((stats.keystroke_us.len() as f64 * p) as usize).copied().unwrap_or(0);
        let pct = |n: usize, d: usize| 100.0 * n as f64 / d.max(1) as f64;
        println!(
            "{:<15} words={:<5} (+{} hit space) typed correctly {:>5.1}%  committed correctly {:>5.1}%  fixed {:>5.1}% of typos  broke {:>4.2}% of correct words",
            scenario.name,
            stats.words,
            stats.split_by_space,
            pct(stats.typed_ok, stats.words),
            pct(stats.output_ok, stats.words),
            pct(stats.fixed, stats.words - stats.typed_ok),
            pct(stats.broken, stats.typed_ok),
        );
        println!("{:<15} suggestions per keystroke: p50 {}µs  p95 {}µs  max {}µs", "", q(0.5), q(0.95), q(0.999));
    }
}

fn run(lang: Language, sentences: &[String], scenario: &Scenario, strength: AutocorrectStrength) -> Stats {
    let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
    engine.autocorrect_strength = strength;
    engine.set_enabled_languages(vec![lang]);
    engine.set_language(lang);
    engine.prediction.dictionary.adaptive_dict.enabled = false;
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    let mut stats = Stats::default();
    let mut time = 1_000u64;

    for sentence in sentences {
        let mut prefix = String::new();
        for raw in sentence.split_whitespace() {
            let target: String = raw.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            engine.set_editor_context(&prefix, false);
            let keys = key_centers(&engine);
            if target.chars().count() >= 2 && target.chars().all(|c| keys.contains_key(&c)) {
                let typed_target = perturb(&target, scenario.edit_rate, &mut rng);
                let _ = engine.state.drain_events();
                let mut typed = String::new();
                for c in typed_target.chars() {
                    let (cx, cy, w, h) = keys[&c];
                    let x = cx + rng.gauss() * scenario.sigma * w;
                    let y = cy + rng.gauss() * scenario.sigma * h;
                    engine.on_touch(TouchAction::Down, 0, x, y, time);
                    time += 60;
                    // Key release plus the suggestion refresh the device does after it
                    let t = std::time::Instant::now();
                    engine.on_touch(TouchAction::Up, 0, x, y, time);
                    engine.update_suggestions();
                    stats.keystroke_us.push(t.elapsed().as_micros() as u64);
                    time += 90;
                }
                typed.push_str(&apply_events(&engine.state.drain_events(), ""));
                let trace = if std::env::var_os("EVAL_TRACE").is_some() {
                    let obs = rynk_core::prediction::correction::observations(
                        &engine.state.composing_text,
                        &engine.state.composing_touches,
                    );
                    let ranked = engine.prediction.dictionary.rank_candidates(
                        lang,
                        &engine.state.word_context(),
                        &obs,
                        Some(&engine.key_geometry),
                    );
                    let touches = obs.iter().filter(|o| o.point.is_some()).count();
                    format!(
                        "user {} learned {} overlay {} touches {touches}/{} top: {:?}",
                        engine.prediction.dictionary.user_dict.len(),
                        engine.prediction.dictionary.adaptive_dict.learned_words.len(),
                        engine.prediction.dictionary.get_lexicon(lang).overlay_entries().len(),
                        obs.len(),
                        ranked.iter().take(3).map(|r| (r.word.as_str(), r.cost, r.score)).collect::<Vec<_>>()
                    )
                } else {
                    String::new()
                };
                engine.execute_key_action(KeyAction::Space);
                time += 400;
                let output = apply_events(&engine.state.drain_events(), &typed).trim().to_lowercase();

                if typed.chars().any(|c| !c.is_alphabetic()) {
                    stats.split_by_space += 1;
                    continue;
                }
                stats.words += 1;
                // Russian е/ё are interchangeable when typing
                let norm = |s: &str| s.to_lowercase().replace('ё', "е");
                let typed_ok = norm(&typed) == norm(&target);
                let output_ok = norm(&output) == norm(&target);
                stats.typed_ok += typed_ok as usize;
                stats.output_ok += output_ok as usize;
                stats.fixed += (!typed_ok && output_ok) as usize;
                if !output_ok && std::env::var_os("EVAL_VERBOSE").is_some() {
                    let valid = engine.prediction.dictionary.is_valid_input(&typed, lang);
                    eprintln!("  {target:>14} typed {typed:<14} -> {output:<14}{} {trace}", if valid { " (typed is a word)" } else { "" });
                }
                stats.broken += (typed_ok && !output_ok) as usize;
            }
            prefix.push_str(raw);
            prefix.push(' ');
        }
    }
    stats
}

/// Lowercase letter -> (center x, center y, key width, key height) on the current layout
fn key_centers(engine: &KeyboardEngine) -> HashMap<char, (f32, f32, f32, f32)> {
    assert_eq!(engine.state.mode, KeyboardMode::Alphabet);
    let mut map = HashMap::new();
    for key in &engine.keys {
        if let KeyAction::Character(c) = key.action {
            for l in c.to_lowercase() {
                map.insert(l, (key.x + key.width / 2.0, key.y + key.height / 2.0, key.width, key.height));
            }
        }
    }
    map
}

/// Occasionally omits, swaps or doubles a letter
fn perturb(word: &str, rate: f32, rng: &mut Rng) -> String {
    let mut chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 || rng.next_f32() >= rate {
        return word.to_string();
    }
    let i = 1 + (rng.next_f32() * (chars.len() - 2) as f32) as usize;
    match (rng.next_f32() * 3.0) as u32 {
        0 => {
            chars.remove(i);
        }
        1 => chars.swap(i, i - 1),
        _ => chars.insert(i, chars[i]),
    }
    chars.into_iter().collect()
}

/// Applies commit/delete events to `text`
fn apply_events(events: &[KeyboardOutputEvent], text: &str) -> String {
    let mut out = text.to_string();
    for e in events {
        match e {
            KeyboardOutputEvent::CommitText(t) => out.push_str(t),
            KeyboardOutputEvent::DeleteSurroundingText { before, .. } => {
                let mut units = *before as usize;
                while units > 0 {
                    match out.pop() {
                        Some(c) => units = units.saturating_sub(c.len_utf16()),
                        None => break,
                    }
                }
            }
            _ => {}
        }
    }
    out
}
