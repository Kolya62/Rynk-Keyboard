//! Builds a Rynk language model file (`<lang>.rlm`) for one language.
//!
//! Inputs (all optional except `--lang` and `--out`):
//!   --sentences  Leipzig Corpora `*-sentences.txt` (`id<TAB>sentence` per line), repeatable
//!   --subtitles  OpenSubtitles frequency list (`word count` per line, FrequencyWords)
//!   --curated    hand-curated `word:freq` list (slang, abbreviations, canonical casing)
//!   --bigrams    curated `word:next1,next2` list; `--multi-bigrams` takes `code:word:nexts`
//!   --tier       A (big vocab + trigrams) | B (medium, bigrams) | C (curated only)
//!
//! Example:
//!   dictgen --lang ru --tier A --sentences rus-sentences.txt --subtitles ru_50k.txt \
//!           --curated ru_words.txt --bigrams ru_bigrams.txt --out ru.rlm

use rynk_core::keyboard::key::{KeyAction, KeyboardMode};
use rynk_core::keyboard::layout::{LayoutBuilder, LayoutMetrics};
use rynk_core::keyboard::state::{Language, ShiftState};
use rynk_core::prediction::autocorrect::Autocorrect;
use rynk_core::prediction::lm_data::{
    ctx_id, quantize, trigram_key, LanguageModelData, NgramTable, WordEntry, SENTENCE_START,
};
use rynk_core::prediction::typos::get_quick_correction;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

const MAX_FREQ: f64 = 2500.0;
const MAX_WORD_CHARS: usize = 32;
/// Everyday dialog sentences are what people type; they count double against news and web text
const DIALOG_WEIGHT: u32 = 2;
/// Mid-sentence occurrences needed before a non-lowercase spelling becomes canonical
const MIN_CASE_VOTES: u32 = 3;
/// Trigrams must raise the probability over the bigram by this factor to be kept
const INFORMATIVE_TRIGRAM_RATIO: f64 = 1.5;
/// Frequency cap for curated words missing from the corpus
const CURATED_ONLY_MAX_FREQ: u32 = 1500;

struct Tier {
    max_words: usize,
    min_count: u32,
    bigram_min_count: u32,
    bigrams_per_ctx: usize,
    max_bigrams: usize,
    trigram_min_count: u32,
    trigrams_per_ctx: usize,
    max_trigrams: usize,
}

const TIER_A: Tier = Tier {
    max_words: 120_000,
    min_count: 3,
    bigram_min_count: 2,
    bigrams_per_ctx: 24,
    max_bigrams: 250_000,
    trigram_min_count: 3,
    trigrams_per_ctx: 10,
    max_trigrams: 200_000,
};

const TIER_B: Tier = Tier {
    max_words: 40_000,
    min_count: 3,
    bigram_min_count: 3,
    bigrams_per_ctx: 12,
    max_bigrams: 60_000,
    trigram_min_count: u32::MAX,
    trigrams_per_ctx: 0,
    max_trigrams: 0,
};

const TIER_C: Tier = Tier {
    max_words: usize::MAX,
    min_count: u32::MAX,
    bigram_min_count: u32::MAX,
    bigrams_per_ctx: 0,
    max_bigrams: 0,
    trigram_min_count: u32::MAX,
    trigrams_per_ctx: 0,
    max_trigrams: 0,
};

#[derive(Default)]
struct Args {
    lang: String,
    tier: String,
    out: String,
    /// Repeatable: news/wiki corpus plus web corpus
    sentences: Vec<String>,
    /// Repeatable: conversational sentences (Tatoeba), counted with `DIALOG_WEIGHT`
    dialog: Vec<String>,
    subtitles: Option<String>,
    curated: Option<String>,
    bigrams: Option<String>,
    multi_bigrams: Option<String>,
}

fn parse_args() -> Args {
    let mut a = Args {
        tier: "B".into(),
        ..Default::default()
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut val = || {
            it.next()
                .unwrap_or_else(|| die(&format!("missing value for {flag}")))
        };
        match flag.as_str() {
            "--lang" => a.lang = val(),
            "--tier" => a.tier = val(),
            "--out" => a.out = val(),
            "--sentences" => a.sentences.push(val()),
            "--dialog" => a.dialog.push(val()),
            "--subtitles" => a.subtitles = Some(val()),
            "--curated" => a.curated = Some(val()),
            "--bigrams" => a.bigrams = Some(val()),
            "--multi-bigrams" => a.multi_bigrams = Some(val()),
            _ => die(&format!("unknown flag {flag}")),
        }
    }
    if a.lang.is_empty() || a.out.is_empty() {
        die("--lang and --out are required");
    }
    a
}

fn die(msg: &str) -> ! {
    eprintln!("dictgen: {msg}");
    std::process::exit(2)
}

fn lines(path: &str) -> impl Iterator<Item = String> {
    let f = std::fs::File::open(path).unwrap_or_else(|e| die(&format!("{path}: {e}")));
    BufReader::new(f).lines().map_while(Result::ok)
}

/// Lowercase letters a user can type on this language's keyboard (keys + long-press alternates).
fn typeable_chars(lang: Language) -> HashSet<char> {
    let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);
    let mut set = HashSet::new();
    for shift in [ShiftState::Off, ShiftState::Shifted] {
        for key in LayoutBuilder::build_layout(KeyboardMode::Alphabet, lang, shift, &metrics) {
            if let KeyAction::Character(c) = key.action {
                set.extend(c.to_lowercase());
            }
            for &c in &key.alternate_chars {
                set.extend(c.to_lowercase());
            }
        }
    }
    set.retain(|c| c.is_alphabetic());
    set
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Script {
    Cyrillic,
    Greek,
    Armenian,
    Georgian,
    Devanagari,
    Bengali,
    Gurmukhi,
    Gujarati,
    Tamil,
    Telugu,
    Kannada,
    Malayalam,
    Sinhala,
    Thai,
    Myanmar,
    Khmer,
    Ethiopic,
    Arabic,
    Hebrew,
}

impl Script {
    fn contains(self, c: char) -> bool {
        let r = match self {
            Script::Cyrillic => 0x0400..=0x052F,
            Script::Greek => 0x0370..=0x03FF,
            Script::Armenian => 0x0530..=0x058F,
            Script::Georgian => 0x10A0..=0x10FF,
            Script::Devanagari => 0x0900..=0x097F,
            Script::Bengali => 0x0980..=0x09FF,
            Script::Gurmukhi => 0x0A00..=0x0A7F,
            Script::Gujarati => 0x0A80..=0x0AFF,
            Script::Tamil => 0x0B80..=0x0BFF,
            Script::Telugu => 0x0C00..=0x0C7F,
            Script::Kannada => 0x0C80..=0x0CFF,
            Script::Malayalam => 0x0D00..=0x0D7F,
            Script::Sinhala => 0x0D80..=0x0DFF,
            Script::Thai => 0x0E00..=0x0E7F,
            Script::Myanmar => 0x1000..=0x109F,
            Script::Khmer => 0x1780..=0x17FF,
            Script::Ethiopic => 0x1200..=0x139F,
            Script::Arabic => 0x0600..=0x06FF,
            Script::Hebrew => 0x0590..=0x05FF,
        };
        r.contains(&(c as u32))
    }
}

/// Native script of languages whose words are not written in Latin letters.
fn native_script(lang: Language) -> Option<Script> {
    use Language::*;
    Some(match lang {
        Russian | Ukrainian | Belarusian | Kazakh | Bulgarian | Macedonian | Kyrgyz | Tajik
        | Mongolian => Script::Cyrillic,
        Greek => Script::Greek,
        Armenian => Script::Armenian,
        Georgian => Script::Georgian,
        Hindi | Marathi | Nepali => Script::Devanagari,
        Bengali => Script::Bengali,
        Punjabi => Script::Gurmukhi,
        Gujarati => Script::Gujarati,
        Tamil => Script::Tamil,
        Telugu => Script::Telugu,
        Kannada => Script::Kannada,
        Malayalam => Script::Malayalam,
        Sinhala => Script::Sinhala,
        Thai => Script::Thai,
        Burmese => Script::Myanmar,
        Khmer => Script::Khmer,
        Amharic => Script::Ethiopic,
        Arabic | Persian | Urdu => Script::Arabic,
        Hebrew => Script::Hebrew,
        _ => return None,
    })
}

fn is_hangul_syllable(c: char) -> bool {
    ('\u{AC00}'..='\u{D7A3}').contains(&c)
}

struct WordFilter {
    typeable: HashSet<char>,
    korean: bool,
    /// Set when the layout cannot type the language's script yet (QWERTY fallback):
    /// words are then kept by script so the model is ready once a native layout exists
    script: Option<Script>,
}

/// Share of corpus letters (by occurrence) a layout can type below which the layout is
/// considered incomplete and words are filtered by script instead
const MIN_LAYOUT_COVERAGE: f64 = 0.98;

impl WordFilter {
    #[cfg(test)]
    fn for_language(lang: Language) -> Self {
        Self::with_coverage(lang, None)
    }

    /// `letter_counts`: letter frequencies of a corpus sample, used to detect layouts that
    /// miss common letters of the language
    fn with_coverage(lang: Language, letter_counts: Option<&HashMap<char, u64>>) -> Self {
        let typeable = typeable_chars(lang);
        let script = native_script(lang).filter(|s| {
            let lacks_script = !typeable.iter().any(|&c| s.contains(c));
            let coverage = letter_counts.map(|counts| {
                let total: u64 = counts
                    .iter()
                    .filter(|(c, _)| s.contains(**c))
                    .map(|(_, n)| n)
                    .sum();
                let typed: u64 = counts
                    .iter()
                    .filter(|(c, _)| s.contains(**c) && typeable.contains(c))
                    .map(|(_, n)| n)
                    .sum();
                if total == 0 {
                    1.0
                } else {
                    typed as f64 / total as f64
                }
            });
            if lacks_script {
                eprintln!(
                    "{}: layout lacks {:?} letters, filtering by script",
                    lang.code(),
                    s
                );
            } else if let Some(c) = coverage.filter(|&c| c < MIN_LAYOUT_COVERAGE) {
                eprintln!(
                    "{}: layout covers {:.1}% of letters, filtering by script",
                    lang.code(),
                    c * 100.0
                );
            }
            lacks_script || coverage.is_some_and(|c| c < MIN_LAYOUT_COVERAGE)
        });
        Self {
            typeable,
            korean: lang == Language::Korean,
            script,
        }
    }
}

/// Letter frequencies of the first lines of a corpus.
fn sample_letters(path: &str, lines_to_read: usize) -> HashMap<char, u64> {
    let mut counts = HashMap::new();
    for line in lines(path).take(lines_to_read) {
        let text = line.rsplit_once('\t').map(|(_, t)| t).unwrap_or(&line);
        for c in text
            .chars()
            .flat_map(char::to_lowercase)
            .filter(|c| c.is_alphabetic())
        {
            *counts.entry(c).or_insert(0) += 1;
        }
    }
    counts
}

/// Serbian Cyrillic to Latin (gajica); the Serbian layout is Latin while most corpora are Cyrillic.
fn serbian_to_latin(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        let lower = c.to_lowercase().next().unwrap_or(c);
        let lat = match lower {
            'а' => "a",
            'б' => "b",
            'в' => "v",
            'г' => "g",
            'д' => "d",
            'ђ' => "đ",
            'е' => "e",
            'ж' => "ž",
            'з' => "z",
            'и' => "i",
            'ј' => "j",
            'к' => "k",
            'л' => "l",
            'љ' => "lj",
            'м' => "m",
            'н' => "n",
            'њ' => "nj",
            'о' => "o",
            'п' => "p",
            'р' => "r",
            'с' => "s",
            'т' => "t",
            'ћ' => "ć",
            'у' => "u",
            'ф' => "f",
            'х' => "h",
            'ц' => "c",
            'ч' => "č",
            'џ' => "dž",
            'ш' => "š",
            _ => {
                out.push(c);
                continue;
            }
        };
        if c.is_uppercase() {
            let mut chars = lat.chars();
            out.extend(chars.next().into_iter().flat_map(char::to_uppercase));
            out.push_str(chars.as_str());
        } else {
            out.push_str(lat);
        }
    }
    out
}

impl WordFilter {
    /// Lowercased form if the token is a word the keyboard can produce, else None.
    fn accept(&self, token: &str) -> Option<String> {
        // Words glued together in the source ("МоскваБольше") are not words; genuine mixed-case
        // spellings (iPhone, macOS) come from the curated lists
        if token.chars().zip(token.chars().skip(1)).any(|(a, b)| a.is_lowercase() && b.is_uppercase()) {
            return None;
        }
        // The keyboard types ASCII apostrophes: "don’t" and "don't" are the same word
        let lower = token.to_lowercase().replace(['’', 'ʼ'], "'");
        let n = lower.chars().count();
        if n == 0 || n > MAX_WORD_CHARS {
            return None;
        }
        let mut letters = 0;
        for (i, c) in lower.chars().enumerate() {
            if c.is_alphabetic() {
                let ok = match self.script {
                    Some(script) => script.contains(c),
                    None => {
                        self.typeable.contains(&c)
                            || self.typeable.contains(&Autocorrect::strip_diacritics(c))
                            || (self.korean && is_hangul_syllable(c))
                    }
                };
                if !ok {
                    return None;
                }
                letters += 1;
            } else if (c == '\'' || c == '-') && i > 0 && i + 1 < n {
                // inner apostrophe / hyphen: "don't", "по-моему"
            } else if is_combining_mark(c) {
                // Indic vowel signs, viramas etc. are part of the word
            } else {
                return None;
            }
        }
        (letters > 0).then_some(lower)
    }
}

fn is_combining_mark(c: char) -> bool {
    // General category M* without pulling in a tables crate: combining blocks + Indic/SEA signs
    matches!(c as u32,
        0x0300..=0x036F | 0x0483..=0x0489 | 0x0591..=0x05C7 | 0x0610..=0x061A | 0x064B..=0x065F
        | 0x0670 | 0x06D6..=0x06ED | 0x0900..=0x0903 | 0x093A..=0x094F | 0x0951..=0x0957
        | 0x0962..=0x0963 | 0x0981..=0x0983 | 0x09BC..=0x09D7 | 0x09E2..=0x09E3
        | 0x0A01..=0x0A03 | 0x0A3C..=0x0A51 | 0x0A70..=0x0A71 | 0x0A75 | 0x0A81..=0x0A83
        | 0x0ABC..=0x0ACD | 0x0AE2..=0x0AE3 | 0x0B01..=0x0B03 | 0x0B3C..=0x0B57
        | 0x0B82 | 0x0BBE..=0x0BCD | 0x0BD7 | 0x0C00..=0x0C04 | 0x0C3C..=0x0C56
        | 0x0C62..=0x0C63 | 0x0C81..=0x0C83 | 0x0CBC..=0x0CD6 | 0x0CE2..=0x0CE3
        | 0x0D00..=0x0D03 | 0x0D3B..=0x0D57 | 0x0D62..=0x0D63 | 0x0D81..=0x0D83
        | 0x0DCA..=0x0DDF | 0x0DF2..=0x0DF3 | 0x0E31 | 0x0E34..=0x0E3A | 0x0E47..=0x0E4E
        | 0x102B..=0x103E | 0x17B4..=0x17D3 | 0x200C..=0x200D)
}

#[derive(Default)]
struct CorpusStats {
    unigrams: HashMap<String, u32>,
    /// Spelling votes from non-sentence-initial positions
    spellings: HashMap<String, HashMap<String, u32>>,
}

/// Streams a corpus sentence by sentence as `(accepted lowercase word, original token)` pairs;
/// rejected tokens (numbers, foreign words) are `None` and break n-gram chains. Corpora are
/// read twice (vocabulary, then n-grams) instead of being held in memory.
/// Tatoeba writes most examples about the same placeholder people; as data they would make
/// "Tom" the most likely first word of a sentence in every language
fn is_tatoeba_placeholder(token: &str) -> bool {
    const NAMES: &[&str] = &[
        "tom", "toms", "toma", "tomu", "tomem", "tomowi", "tomovi", "tomas", "том", "тома", "тому",
        "томом", "томе", "mary", "marie", "maria", "mari", "мэри", "мері", "john", "джон", "jim",
    ];
    token.chars().next().is_some_and(char::is_uppercase) && NAMES.contains(&token.to_lowercase().as_str())
}

fn for_each_sentence(
    path: &str,
    dialog: bool,
    filter: &WordFilter,
    lang: Language,
    mut f: impl FnMut(&[(Option<String>, &str)]),
) {
    for line in lines(path) {
        // Leipzig: "id<TAB>text", Tatoeba: "id<TAB>lang<TAB>text"
        let text = line.rsplit_once('\t').map(|(_, t)| t).unwrap_or(&line);
        let mut text: String = text.nfc().collect();
        if lang == Language::Serbian {
            text = serbian_to_latin(&text);
        }
        let tokens: Vec<(Option<String>, &str)> = text
            .unicode_words()
            .map(|t| {
                let accepted = if dialog && is_tatoeba_placeholder(t) { None } else { filter.accept(t) };
                (accepted, t)
            })
            .collect();
        f(&tokens);
    }
}

/// `corpora`: (path, count weight)
fn read_corpus(corpora: &[(String, u32)], filter: &WordFilter, lang: Language) -> CorpusStats {
    let mut stats = CorpusStats::default();
    for (path, weight) in corpora {
        for_each_sentence(path, *weight == DIALOG_WEIGHT, filter, lang, |tokens| {
            for (pos, (accepted, token)) in tokens.iter().enumerate() {
                let Some(lower) = accepted else { continue };
                *stats.unigrams.entry(lower.clone()).or_insert(0) += weight;
                if pos > 0 {
                    *stats
                        .spellings
                        .entry(lower.clone())
                        .or_default()
                        .entry(token.replace(['’', 'ʼ'], "'"))
                        .or_insert(0) += 1;
                }
            }
        });
    }
    stats
}

/// Most common spelling seen mid-sentence; lowercase unless capitalization clearly dominates.
/// `plurality`: take the most common spelling outright (German, where every noun is
/// capitalized and lowercase variants are informal misspellings).
fn canonical_spelling(lower: &str, votes: Option<&HashMap<String, u32>>, plurality: bool) -> String {
    let Some(votes) = votes else {
        return lower.to_string();
    };
    let lower_votes = votes.get(lower).copied().unwrap_or(0);
    let Some((best, &best_votes)) = votes.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
    else {
        return lower.to_string();
    };
    // A capitalized spelling needs real evidence: one "Уе)) Спасиб" must not capitalize a word
    if best == lower || best_votes < MIN_CASE_VOTES || (!plurality && lower_votes * 10 >= best_votes * 3) {
        lower.to_string()
    } else {
        best.clone()
    }
}

fn main() {
    let args = parse_args();
    let lang = Language::from_code(&args.lang)
        .unwrap_or_else(|| die(&format!("unknown language {}", args.lang)));
    let tier = match args.tier.as_str() {
        "A" => &TIER_A,
        "B" => &TIER_B,
        "C" => &TIER_C,
        t => die(&format!("unknown tier {t}")),
    };
    let sample = args.sentences.first().or(args.dialog.first()).map(|p| sample_letters(p, 20_000));
    let filter = WordFilter::with_coverage(lang, sample.as_ref());

    // 1. Corpus statistics (all corpora pooled)
    let corpora: Vec<(String, u32)> = args
        .sentences
        .iter()
        .map(|p| (p.clone(), 1))
        .chain(args.dialog.iter().map(|p| (p.clone(), DIALOG_WEIGHT)))
        .collect();
    let corpus = read_corpus(&corpora, &filter, lang);
    let corpus_total: u64 = corpus.unigrams.values().map(|&c| c as u64).sum();

    let mut subtitles: HashMap<String, u32> = HashMap::new();
    if let Some(p) = &args.subtitles {
        for line in lines(p) {
            let mut parts = line.split_whitespace();
            if let (Some(w), Some(c)) = (
                parts.next(),
                parts.next().and_then(|c| c.parse::<u32>().ok()),
            ) {
                let w: String = w.nfc().collect();
                if let Some(lower) = filter.accept(&w) {
                    *subtitles.entry(lower).or_insert(0) += c;
                }
            }
        }
    }
    let subs_total: u64 = subtitles.values().map(|&c| c as u64).sum();

    // Curated entries keep their spelling and act as a frequency floor
    let mut curated: Vec<(String, u32)> = Vec::new();
    if let Some(p) = &args.curated {
        for line in lines(p) {
            if let Some((w, f)) = line.rsplit_once(':') {
                if let Ok(f) = f.trim().parse::<u32>() {
                    let w = w.trim();
                    if !w.is_empty() {
                        curated.push((w.nfc().collect(), f.min(MAX_FREQ as u32)));
                    }
                }
            }
        }
    }

    // 2. Vocabulary scored by relative frequency, mixing written (news/wiki) and spoken (subtitles)
    let (w_corpus, w_subs) = match (corpus_total > 0, subs_total > 0) {
        (true, true) => (0.6, 0.4),
        (true, false) => (1.0, 0.0),
        (false, true) => (0.0, 1.0),
        (false, false) => (0.0, 0.0),
    };
    let mut scores: HashMap<String, (f64, u32)> = HashMap::new();
    for (w, &c) in &corpus.unigrams {
        if c >= tier.min_count {
            let e = scores.entry(w.clone()).or_insert((0.0, 0));
            e.0 += w_corpus * c as f64 / corpus_total as f64;
            e.1 += c;
        }
    }
    for (w, &c) in &subtitles {
        if c >= tier.min_count.min(2) {
            let e = scores.entry(w.clone()).or_insert((0.0, 0));
            e.0 += w_subs * c as f64 / subs_total as f64;
            e.1 = e.1.max(1);
        }
    }
    // Known misspellings ("пожалуста", "вобще") occur in subtitles and news too; keeping them
    // would make the keyboard treat them as valid words and never correct them
    scores.retain(|w, _| get_quick_correction(w).is_none_or(|fix| fix.to_lowercase() == *w));
    let mut vocab: Vec<(String, f64, u32)> =
        scores.into_iter().map(|(w, (s, c))| (w, s, c)).collect();
    vocab.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    vocab.truncate(tier.max_words);

    let (max_score, min_score) = match (vocab.first(), vocab.last()) {
        (Some(a), Some(b)) => (a.1, b.1),
        _ => (1.0, 1.0),
    };
    let span = (max_score / min_score).ln().max(1e-9);
    let scale = |s: f64| -> u16 {
        if max_score <= min_score {
            return 1000;
        }
        (MAX_FREQ * (s / min_score).ln() / span)
            .round()
            .clamp(1.0, MAX_FREQ) as u16
    };

    let mut words: Vec<WordEntry> = Vec::with_capacity(vocab.len() + curated.len());
    let mut index: HashMap<String, usize> = HashMap::with_capacity(vocab.len() + curated.len());
    for (lower, score, count) in vocab {
        let spelling = canonical_spelling(&lower, corpus.spellings.get(&lower), lang == Language::German);
        index.insert(lower, words.len());
        words.push(WordEntry {
            word: spelling,
            count,
            freq: scale(score),
        });
    }
    // Curated lists only have coarse frequency buckets: corpus statistics win for words the
    // corpus knows; curated-only words (slang, new terms) are capped below the core vocabulary
    let has_corpus = !words.is_empty();
    for (w, f) in curated {
        let lower = w.to_lowercase();
        match index.get(&lower) {
            Some(&i) => {
                // Curated casing wins (macOS, СПб, ООО)
                if w != lower {
                    words[i].word = w;
                }
            }
            None => {
                let freq = if has_corpus {
                    f.min(CURATED_ONLY_MAX_FREQ)
                } else {
                    f
                };
                index.insert(lower, words.len());
                words.push(WordEntry {
                    word: w,
                    count: 0,
                    freq: freq as u16,
                });
            }
        }
    }

    // Ids ordered by descending frequency keep varints short for common words
    let mut order: Vec<usize> = (0..words.len()).collect();
    order.sort_by(|&a, &b| {
        words[b]
            .freq
            .cmp(&words[a].freq)
            .then(words[b].count.cmp(&words[a].count))
    });
    let mut remap = vec![0u32; words.len()];
    for (new_id, &old) in order.iter().enumerate() {
        remap[old] = new_id as u32;
    }
    let words: Vec<WordEntry> = order.iter().map(|&i| words[i].clone()).collect();
    let id_of = |w: &str| -> Option<u32> { index.get(w).map(|&i| remap[i]) };

    // 3. N-gram counts within sentences; unknown tokens break the chain
    let mut bi: HashMap<(u32, u32), u32> = HashMap::new();
    let mut tri: HashMap<(u32, u32, u32), u32> = HashMap::new();
    let want_tri = tier.trigrams_per_ctx > 0;
    if tier.bigrams_per_ctx > 0 {
        for (path, weight) in &corpora {
            let weight = *weight;
            for_each_sentence(path, weight == DIALOG_WEIGHT, &filter, lang, |tokens| {
                let (mut c1, mut c2): (Option<u32>, Option<u32>) = (None, Some(SENTENCE_START));
                for (accepted, _) in tokens {
                    let id = accepted.as_deref().and_then(id_of);
                    match id {
                        Some(w) => {
                            if let Some(p) = c2 {
                                *bi.entry((p, w)).or_insert(0) += weight;
                                if let (true, Some(pp)) = (want_tri, c1) {
                                    *tri.entry((pp, p, w)).or_insert(0) += weight;
                                }
                            }
                            c1 = c2;
                            c2 = Some(ctx_id(w));
                        }
                        None => {
                            c1 = None;
                            c2 = None;
                        }
                    }
                }
            });
        }
    }

    // A trigram is worth its space only if the second context word changes the prediction;
    // most do not ("... в итоге" follows "в" whatever came before)
    let mut bi_totals: HashMap<u32, u64> = HashMap::new();
    for (&(ctx, _), &n) in &bi {
        *bi_totals.entry(ctx).or_insert(0) += n as u64;
    }
    let mut tri_totals: HashMap<(u32, u32), u64> = HashMap::new();
    for (&(a, b, _), &n) in &tri {
        *tri_totals.entry((a, b)).or_insert(0) += n as u64;
    }
    tri.retain(|&(a, b, w), &mut n| {
        let p3 = n as f64 / tri_totals[&(a, b)] as f64;
        let p2 = bi.get(&(b, w)).map(|&m| m as f64 / bi_totals[&b] as f64).unwrap_or(0.0);
        p3 >= INFORMATIVE_TRIGRAM_RATIO * p2
    });
    let trigrams = prune(
        tri.into_iter()
            .map(|((a, b, w), n)| (trigram_key(a, b), w, n))
            .collect(),
        tier.trigram_min_count,
        tier.trigrams_per_ctx,
        tier.max_trigrams,
    );
    let mut bigrams = prune(
        bi.into_iter().map(|((c, w), n)| (c as u64, w, n)).collect(),
        tier.bigram_min_count,
        tier.bigrams_per_ctx,
        tier.max_bigrams,
    );

    // 4. Curated bigrams are hand-picked, so they get a solid probability if missing
    let mut curated_pairs: Vec<(String, String)> = Vec::new();
    if let Some(p) = &args.bigrams {
        for line in lines(p) {
            if let Some((w, nexts)) = line.split_once(':') {
                for n in nexts.split(',') {
                    curated_pairs.push((w.trim().to_lowercase(), n.trim().to_lowercase()));
                }
            }
        }
    }
    if let Some(p) = &args.multi_bigrams {
        for line in lines(p) {
            let mut parts = line.splitn(3, ':');
            if let (Some(code), Some(w), Some(nexts)) = (parts.next(), parts.next(), parts.next()) {
                if code.trim() == args.lang {
                    for n in nexts.split(',') {
                        curated_pairs.push((w.trim().to_lowercase(), n.trim().to_lowercase()));
                    }
                }
            }
        }
    }
    let present: HashSet<(u64, u32)> = bigrams.iter().map(|&(k, w, _)| (k, w)).collect();
    for (a, b) in curated_pairs {
        if let (Some(ia), Some(ib)) = (id_of(&a), id_of(&b)) {
            let key = ctx_id(ia) as u64;
            if !present.contains(&(key, ib)) {
                bigrams.push((key, ib, quantize(0.15)));
            }
        }
    }

    let model = LanguageModelData {
        words,
        bigrams: NgramTable::from_triples(bigrams),
        trigrams: NgramTable::from_triples(trigrams),
    };
    let bytes = model.encode();
    std::fs::write(&args.out, &bytes).unwrap_or_else(|e| die(&format!("{}: {e}", args.out)));
    eprintln!(
        "{}: {} words, {} bigrams, {} trigrams, {:.2} MB",
        args.lang,
        model.words.len(),
        model.bigrams.len(),
        model.trigrams.len(),
        bytes.len() as f64 / 1e6
    );
}

/// Keeps n-grams with `count >= min_count`, the `per_ctx` most likely per context and at most
/// `max_total` overall (by count), and converts counts to quantized conditional probabilities.
fn prune(
    counts: Vec<(u64, u32, u32)>,
    min_count: u32,
    per_ctx: usize,
    max_total: usize,
) -> Vec<(u64, u32, u8)> {
    if per_ctx == 0 || max_total == 0 {
        return Vec::new();
    }
    let mut ctx_totals: HashMap<u64, u64> = HashMap::new();
    for &(k, _, n) in &counts {
        *ctx_totals.entry(k).or_insert(0) += n as u64;
    }
    let mut by_ctx: HashMap<u64, Vec<(u32, u32)>> = HashMap::new();
    for (k, w, n) in counts {
        if n >= min_count {
            by_ctx.entry(k).or_default().push((w, n));
        }
    }
    let mut kept: Vec<(u64, u32, u32)> = Vec::new();
    for (k, mut list) in by_ctx {
        list.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        list.truncate(per_ctx);
        kept.extend(list.into_iter().map(|(w, n)| (k, w, n)));
    }
    kept.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
    kept.truncate(max_total);
    kept.into_iter()
        .map(|(k, w, n)| (k, w, quantize(n as f64 / ctx_totals[&k] as f64)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn votes(pairs: &[(&str, u32)]) -> HashMap<String, u32> {
        pairs.iter().map(|&(w, n)| (w.to_string(), n)).collect()
    }

    #[test]
    fn spelling_keeps_proper_nouns_and_lowercases_common_words() {
        assert_eq!(
            canonical_spelling("москва", Some(&votes(&[("Москва", 90), ("москва", 2)])), false),
            "Москва"
        );
        // Lowercase is common enough mid-sentence: not a proper noun
        assert_eq!(
            canonical_spelling("дом", Some(&votes(&[("Дом", 10), ("дом", 5)])), false),
            "дом"
        );
        assert_eq!(canonical_spelling("дом", None, false), "дом");
        assert_eq!(canonical_spelling("dank", Some(&votes(&[("Dank", 10), ("dank", 5)])), true), "Dank");
        assert_eq!(canonical_spelling("спасиб", Some(&votes(&[("Спасиб", 1)])), false), "спасиб");
    }

    #[test]
    fn filter_accepts_only_typeable_words() {
        let ru = WordFilter::for_language(Language::Russian);
        assert_eq!(ru.accept("Привет").as_deref(), Some("привет"));
        assert_eq!(ru.accept("по-моему").as_deref(), Some("по-моему"));
        assert_eq!(
            ru.accept("hello"),
            None,
            "Latin is not on the Russian layout"
        );
        assert_eq!(ru.accept("2024"), None);
        assert_eq!(ru.accept("-нибудь"), None, "leading hyphen");
        let en = WordFilter::for_language(Language::English);
        assert_eq!(en.accept("Don’t").as_deref(), Some("don't"));

        // No native Greek layout yet: Greek words are still collected by script
        let el = WordFilter::for_language(Language::Greek);
        assert_eq!(el.accept("Καλημέρα").as_deref(), Some("καλημέρα"));
        assert_eq!(
            el.accept("hello"),
            None,
            "Latin words do not pollute a Greek model"
        );
    }

    #[test]
    fn tatoeba_placeholder_names_are_skipped() {
        assert!(is_tatoeba_placeholder("Tom"));
        assert!(is_tatoeba_placeholder("Тому"));
        assert!(!is_tatoeba_placeholder("том"), "the pronoun stays");
        assert!(!is_tatoeba_placeholder("Tomorrow"));
    }

    #[test]
    fn serbian_cyrillic_is_transliterated() {
        assert_eq!(
            serbian_to_latin("Љубав и Џон, ћерка"),
            "Ljubav i Džon, ćerka"
        );
    }

    #[test]
    fn incomplete_layouts_fall_back_to_script_filter() {
        // A corpus dominated by a letter the layout cannot type
        let counts: HashMap<char, u64> = [('ж', 50u64), ('а', 50)].into_iter().collect();
        let complete = WordFilter::with_coverage(Language::Russian, Some(&counts));
        assert!(complete.script.is_none());
        let counts: HashMap<char, u64> = [('ґ', 50u64), ('а', 50)].into_iter().collect();
        let incomplete = WordFilter::with_coverage(Language::Russian, Some(&counts));
        assert!(incomplete.script.is_some());

        let de = WordFilter::for_language(Language::German);
        assert_eq!(de.accept("Straße").as_deref(), Some("straße"));
    }

    #[test]
    fn prune_converts_counts_to_conditional_probabilities() {
        let kept = prune(
            vec![(1, 10, 6), (1, 11, 3), (1, 12, 1), (2, 10, 5)],
            2,
            1,
            10,
        );
        assert!(kept.contains(&(1, 10, quantize(0.6))));
        assert!(
            !kept.iter().any(|&(k, w, _)| k == 1 && w == 11),
            "per-context cap"
        );
        assert!(!kept.iter().any(|&(_, w, _)| w == 12), "min count");
        assert!(kept.contains(&(2, 10, quantize(1.0))));
    }
}
