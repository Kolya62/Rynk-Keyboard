use super::lexicon::{Lexicon, LexiconBase};
use super::lm_data::{LanguageModelData, NgramTable};
use crate::keyboard::state::Language;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};

pub static PROFANITY_RAW: &str = include_str!("data/profanity.txt");

pub use super::adaptive::AdaptiveDictionary;

pub type CandidateBucketMap = HashMap<(char, usize), Vec<(&'static str, u32)>>;

/// Fuzzy candidate buckets keep only the most frequent words per (first letter, length)
const BUCKET_CAP: usize = 50;
/// Next-word predictions returned per context
const CONTEXT_PREDICTIONS: usize = 6;
/// Dictionary words below this frequency may still be autocorrected (see `dominant_alternative`)
const RARE_WORD_FREQ: u32 = 400;
const MIN_DOMINANT_FREQ: u32 = 1000;
/// Frequency gap meaning "about 100× more common": the 0..=2500 scale spans ~5.7 decades of
/// probability, so 2 decades ≈ 880 points
const DOMINANCE_FREQ_GAP: u32 = 880;

/// N-gram tables of one language model
#[derive(Default)]
pub struct NgramModel {
    pub bigrams: NgramTable,
    pub trigrams: NgramTable,
}

/// Immutable data of a loaded language, shared process-wide
struct LoadedLanguage {
    lexicon: Arc<LexiconBase>,
    buckets: Arc<CandidateBucketMap>,
    ngrams: Arc<NgramModel>,
}

/// Each language model is decoded once per process: decoding is costly and lexicon strings
/// are leaked, so rebuilding on every engine creation would also leak memory.
static LOADED_LANGUAGES: Mutex<Option<HashMap<Language, Arc<LoadedLanguage>>>> = Mutex::new(None);

impl LoadedLanguage {
    fn build(model: LanguageModelData) -> Self {
        let LanguageModelData { words, bigrams, trigrams } = model;
        let lexicon = LexiconBase::from_words(words.into_iter().map(|w| (w.word, w.freq as u32)).collect());

        let mut buckets: CandidateBucketMap = HashMap::new();
        for e in lexicon.entries() {
            let c0 = e.lower.chars().next().unwrap_or('\0');
            buckets.entry((c0, e.lower.chars().count())).or_default().push((e.canonical, e.freq));
        }
        for list in buckets.values_mut() {
            list.sort_unstable_by_key(|e| std::cmp::Reverse(e.1));
            list.truncate(BUCKET_CAP);
            list.shrink_to_fit();
        }

        Self {
            lexicon: Arc::new(lexicon),
            buckets: Arc::new(buckets),
            ngrams: Arc::new(NgramModel { bigrams, trigrams }),
        }
    }
}

pub struct Dictionary {
    pub lexicons: HashMap<Language, Lexicon>,
    pub candidate_buckets: HashMap<Language, Arc<CandidateBucketMap>>,
    pub ngrams: HashMap<Language, Arc<NgramModel>>,
    pub profanity: HashSet<&'static str>,
    pub profanity_enabled: bool,
    pub user_dict: HashMap<String, u32>,
    pub adaptive_dict: AdaptiveDictionary,
    pub removed_words: HashSet<String>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

fn empty_lexicon() -> &'static Lexicon {
    static EMPTY: OnceLock<Lexicon> = OnceLock::new();
    EMPTY.get_or_init(Lexicon::default)
}

impl Dictionary {
    pub fn new() -> Self {
        let mut profanity = HashSet::with_capacity(200);
        for line in PROFANITY_RAW.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                profanity.insert(trimmed);
            }
        }

        let mut dict = Self {
            lexicons: HashMap::new(),
            candidate_buckets: HashMap::new(),
            ngrams: HashMap::new(),
            profanity,
            profanity_enabled: true,
            user_dict: HashMap::new(),
            adaptive_dict: AdaptiveDictionary::default(),
            removed_words: HashSet::new(),
        };

        // Always load Russian and English as primary base languages
        dict.ensure_language_loaded(Language::Russian);
        dict.ensure_language_loaded(Language::English);

        dict
    }

    /// Loads `lm/<code>.rlm` (decoded once per process). A missing or corrupt model leaves the
    /// language without suggestions instead of failing: typing must keep working.
    pub fn ensure_language_loaded(&mut self, lang: Language) {
        if self.lexicons.contains_key(&lang) {
            return;
        }
        let loaded = {
            let mut cache = LOADED_LANGUAGES.lock().unwrap_or_else(|e| e.into_inner());
            let cache = cache.get_or_insert_with(HashMap::new);
            cache
                .entry(lang)
                .or_insert_with(|| {
                    let model = super::model_source::load_model_bytes(lang)
                        .and_then(|bytes| LanguageModelData::decode(&bytes))
                        .unwrap_or_default();
                    Arc::new(LoadedLanguage::build(model))
                })
                .clone()
        };
        self.attach(lang, &loaded);
    }

    /// Installs a model for this dictionary only, bypassing the process cache (tests).
    pub fn install_model(&mut self, lang: Language, model: LanguageModelData) {
        self.attach(lang, &LoadedLanguage::build(model));
    }

    fn attach(&mut self, lang: Language, loaded: &LoadedLanguage) {
        self.lexicons.insert(lang, Lexicon::new(loaded.lexicon.clone()));
        self.candidate_buckets.insert(lang, loaded.buckets.clone());
        self.ngrams.insert(lang, loaded.ngrams.clone());
    }

    pub fn ensure_languages_loaded(&mut self, langs: &[Language]) {
        for &lang in langs {
            self.ensure_language_loaded(lang);
        }
    }

    pub fn set_profanity_enabled(&mut self, enabled: bool) {
        self.profanity_enabled = enabled;
    }

    pub fn get_lexicon(&self, lang: Language) -> &Lexicon {
        self.lexicons
            .get(&lang)
            .or_else(|| self.lexicons.get(&Language::Russian))
            .unwrap_or_else(|| empty_lexicon())
    }

    pub fn get_fuzzy_candidates(&self, query: &str, lang: Language) -> Vec<(&'static str, u32)> {
        let clean = query.trim().to_lowercase();
        let query_len = clean.chars().count();
        if query_len == 0 {
            return Vec::new();
        }

        let buckets_opt = self.candidate_buckets.get(&lang).or_else(|| self.candidate_buckets.get(&Language::Russian));
        let buckets = match buckets_opt {
            Some(b) => b,
            None => return Vec::new(),
        };

        let min_len = query_len.saturating_sub(2).max(1);
        let max_len = query_len + 2;

        let mut chars = clean.chars();
        let c0 = chars.next();
        let c1 = chars.next();

        let mut candidates = Vec::with_capacity(120);
        let mut seen = HashSet::with_capacity(120);

        let mut lens: Vec<usize> = (min_len..=max_len).collect();
        lens.sort_by_key(|&l| (l as isize - query_len as isize).abs());

        for &len in &lens {
            let take_c0 = if len == query_len { 50 } else { 20 };
            let take_c1 = if len == query_len { 25 } else { 10 };
            if let Some(c) = c0 {
                if let Some(list) = buckets.get(&(c, len)) {
                    for &(w, freq) in list.iter().take(take_c0) {
                        if seen.insert(w) {
                            candidates.push((w, freq));
                            if candidates.len() >= 100 {
                                break;
                            }
                        }
                    }
                }
            }
            if candidates.len() >= 100 {
                break;
            }
            if let Some(c) = c1 {
                if let Some(list) = buckets.get(&(c, len)) {
                    for &(w, freq) in list.iter().take(take_c1) {
                        if seen.insert(w) {
                            candidates.push((w, freq));
                            if candidates.len() >= 100 {
                                break;
                            }
                        }
                    }
                }
            }
            if candidates.len() >= 100 {
                break;
            }
        }

        // Layout neighbor and phonetic confusion check for c0 (if typo was on the first letter)
        if let Some(c) = c0 {
            let adj = crate::prediction::autocorrect::Autocorrect::get_adjacent_chars(c);
            for adj_c in adj.chars() {
                for &len in &lens {
                    if candidates.len() >= 120 {
                        break;
                    }
                    if let Some(list) = buckets.get(&(adj_c, len)) {
                        for &(w, freq) in list.iter().take(15) {
                            if seen.insert(w) {
                                candidates.push((w, freq));
                                if candidates.len() >= 120 {
                                    break;
                                }
                            }
                        }
                    }
                }
                if candidates.len() >= 120 {
                    break;
                }
            }
        }

        // Prefix variants check (e.g. зделал -> check 'с', unpossible -> check 'i', etc.)
        let alt_c0 = match c0 {
            Some('з') => Some('с'),
            Some('с') => Some('з'),
            Some('u') if clean.starts_with("un") => Some('i'),
            Some('i') if clean.starts_with("in") || clean.starts_with("im") => Some('u'),
            Some('d') if clean.starts_with("dis") => Some('m'),
            Some('m') if clean.starts_with("mis") => Some('d'),
            _ => None,
        };
        if let Some(alt) = alt_c0 {
            for &len in &lens {
                if candidates.len() >= 120 {
                    break;
                }
                if let Some(list) = buckets.get(&(alt, len)) {
                    for &(w, freq) in list.iter().take(20) {
                        if seen.insert(w) {
                            candidates.push((w, freq));
                            if candidates.len() >= 120 {
                                break;
                            }
                        }
                    }
                }
            }
        }

        candidates
    }

    pub fn contains_word(&self, word: &str, _is_ru: bool) -> bool {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() || self.removed_words.contains(&clean) {
            return false;
        }
        if self.user_dict.contains_key(&clean)
            || self.adaptive_dict.learned_words.contains_key(&clean)
        {
            return true;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return true;
        }
        self.lexicons.values().any(|l| l.contains(&clean))
    }

    pub fn contains_word_for_lang(&self, word: &str, lang: Language) -> bool {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() || self.removed_words.contains(&clean) {
            return false;
        }
        if self.user_dict.contains_key(&clean)
            || self.adaptive_dict.learned_words.contains_key(&clean)
        {
            return true;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return true;
        }
        self.get_lexicon(lang).contains(&clean)
    }

    pub fn get_word_frequency(&self, word: &str, is_ru: bool) -> u32 {
        let lang = if is_ru {
            Language::Russian
        } else {
            Language::English
        };
        self.get_word_frequency_for_lang(word, lang)
    }

    pub fn get_word_frequency_for_lang(&self, word: &str, lang: Language) -> u32 {
        let clean = word.trim().to_lowercase();
        if self.removed_words.contains(&clean) {
            return 0;
        }
        if let Some(&f) = self.user_dict.get(&clean) {
            return f;
        }
        if let Some(&f) = self.adaptive_dict.learned_words.get(&clean) {
            return f;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return 1000;
        }
        if let Some(f) = self.get_lexicon(lang).get_frequency(&clean) {
            return f;
        }
        // Fallback check in Russian or English lexicon
        [Language::Russian, Language::English]
            .iter()
            .find_map(|l| self.lexicons.get(l).and_then(|lex| lex.get_frequency(&clean)))
            .unwrap_or(0)
    }

    /// Adds a user word from the settings screen, where only Russian/other is known.
    pub fn add_user_word(&mut self, word: &str, is_ru: bool) {
        let lang = if is_ru { Language::Russian } else { Language::English };
        self.add_user_word_in(word, lang);
    }

    pub fn add_user_word_in(&mut self, word: &str, lang: Language) {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() {
            return;
        }
        self.removed_words.remove(&clean);
        self.user_dict.insert(clean.clone(), 2500);
        self.ensure_language_loaded(lang);
        if let Some(lex) = self.lexicons.get_mut(&lang) {
            lex.insert(&clean, 2500);
        }
    }

    pub fn remove_user_word(&mut self, word: &str) {
        let clean = word.trim().to_lowercase();
        self.user_dict.remove(&clean);
        self.adaptive_dict.remove_word(&clean);
        if self.removed_words.len() >= 1000 {
            if let Some(first) = self.removed_words.iter().next().cloned() {
                self.removed_words.remove(&first);
            }
        }
        self.removed_words.insert(clean);
    }

    pub fn get_user_words(&self) -> Vec<String> {
        let mut words: Vec<String> = self.user_dict.keys().cloned().collect();
        words.sort();
        words
    }

    pub fn learn_word(&mut self, word: &str, lang: Language) {
        let trimmed = word.trim().to_lowercase();
        if trimmed.chars().count() < 2 || self.removed_words.contains(&trimmed) {
            return;
        }

        self.adaptive_dict.learn_word_in(&trimmed, Some(lang));
        if let Some(&freq) = self.adaptive_dict.learned_words.get(&trimmed) {
            self.ensure_language_loaded(lang);
            if let Some(lex) = self.lexicons.get_mut(&lang) {
                lex.insert(&trimmed, freq);
            }
        }
    }

    pub fn learn_bigram(&mut self, w1: &str, w2: &str) {
        self.adaptive_dict.learn_bigram(w1, w2);
    }

    /// Puts restored learned words back into the word lists of the languages they were typed in
    /// (words from older files without a language go to the given fallback).
    pub fn apply_learned_words(&mut self, fallback: Language) {
        let learned: Vec<(String, u32, Language)> = self
            .adaptive_dict
            .learned_words
            .iter()
            .map(|(w, &f)| (w.clone(), f, *self.adaptive_dict.word_languages.get(w).unwrap_or(&fallback)))
            .collect();
        for (w, f, lang) in learned {
            if self.removed_words.contains(&w) {
                continue;
            }
            self.ensure_language_loaded(lang);
            if let Some(lex) = self.lexicons.get_mut(&lang) {
                lex.insert(&w, f);
            }
        }
    }

    /// Next-word predictions after `last_word`: model and learned pairs, then preposition-based
    /// guesses, then the Russian/English model as a fallback.
    pub fn get_context_predictions(&self, last_word: &str, lang: Language) -> Vec<String> {
        let k = last_word.trim().to_lowercase();
        if k.is_empty() {
            return Vec::new();
        }
        let ctx = super::lm::WordContext::after(Some(&k));
        let mut res = self.predict_next_words(lang, &ctx, CONTEXT_PREDICTIONS);

        let prep_preds = crate::prediction::morphology::Morphology::get_preposition_context_predictions(&k, lang);
        for &pw in prep_preds {
            let s = pw.to_string();
            if !res.contains(&s) && !self.removed_words.contains(&s) {
                res.push(s);
            }
        }

        if res.is_empty() {
            let fallback = if lang == Language::Russian { Language::English } else { Language::Russian };
            res = self.predict_next_words(fallback, &ctx, CONTEXT_PREDICTIONS);
        }
        res
    }

    /// A rare dictionary word (often a misspelling that slipped into a corpus, e.g. "спасиб")
    /// should not block autocorrection when a near-identical, far more frequent word exists.
    /// Returns that word. User and learned words are never overridden.
    pub fn dominant_alternative(&self, word: &str, lang: Language) -> Option<(String, u32)> {
        let clean = word.trim().to_lowercase();
        if self.user_dict.contains_key(&clean) || self.adaptive_dict.learned_words.contains_key(&clean) {
            return None;
        }
        let lexicon = self.get_lexicon(lang);
        let freq = lexicon.get_frequency(&clean)?;
        if freq >= RARE_WORD_FREQ {
            return None;
        }
        let min_freq = MIN_DOMINANT_FREQ.max(freq + DOMINANCE_FREQ_GAP);
        let mut candidates = lexicon.find_completions(&clean, 8);
        candidates.extend(self.get_fuzzy_candidates(&clean, lang).into_iter().map(|(w, f)| (w.to_string(), f)));
        candidates
            .into_iter()
            .filter(|(w, f)| {
                let lower = w.to_lowercase();
                *f >= min_freq
                    && lower != clean
                    && !self.removed_words.contains(&lower)
                    && super::autocorrect::Autocorrect::weighted_edit_distance(&clean, &lower) <= 1.2
            })
            .max_by_key(|(_, f)| *f)
    }
}
