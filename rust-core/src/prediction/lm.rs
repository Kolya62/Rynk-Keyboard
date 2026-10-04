//! Context model: probability of a word given up to two previous words (stupid backoff over the
//! corpus n-grams of `lm_data`), merged with pairs learned from the user.

use super::dictionary::Dictionary;
use super::lm_data::{ctx_id, dequantize_log10, trigram_key, SENTENCE_START};
use crate::keyboard::state::Language;
use std::collections::HashMap;

/// log10(0.4): stupid backoff weight per order dropped
const BACKOFF_LOG10: f32 = -0.398;
/// Learned pairs are the user's own habits; they outrank corpus statistics of equal probability
const PERSONAL_BOOST_LOG10: f32 = 0.3;
/// Below this many observations a learned pair is weighted down (one-off phrases)
const PERSONAL_MIN_COUNT: u32 = 2;
/// Pseudo-observations added to a word's learned continuations so a single pair does not
/// claim certainty
const PERSONAL_PRIOR: f32 = 2.0;

/// log10 score of a learned pair seen `count` times out of `total` learned continuations.
fn personal_score(count: u32, total: u32) -> f32 {
    let weight = if count >= PERSONAL_MIN_COUNT { PERSONAL_BOOST_LOG10 } else { BACKOFF_LOG10 };
    (count as f32 / (total as f32 + PERSONAL_PRIOR)).log10() + weight
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextToken<'a> {
    SentenceStart,
    Word(&'a str),
}

/// The words before the one being typed (nearest first in `prev1`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WordContext<'a> {
    pub prev1: Option<ContextToken<'a>>,
    pub prev2: Option<ContextToken<'a>>,
}

impl<'a> WordContext<'a> {
    /// Context of only the previous word (no sentence information)
    pub fn after(word: Option<&'a str>) -> Self {
        Self {
            prev1: word.filter(|w| !w.is_empty()).map(ContextToken::Word),
            prev2: None,
        }
    }

    pub fn prev_word(&self) -> Option<&'a str> {
        match self.prev1 {
            Some(ContextToken::Word(w)) => Some(w),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.prev1.is_none()
    }
}

/// Unigram log10 probability approximated from the 0..=2500 ranking frequency, which the model
/// builder derives from log-scaled corpus frequency (roughly 1e-7 .. 5e-2).
pub fn unigram_log10(freq: u32) -> f32 {
    -7.0 + 5.7 * (freq.min(2500) as f32 / 2500.0)
}

impl Dictionary {
    fn context_key(&self, lang: Language, token: ContextToken) -> Option<u32> {
        match token {
            ContextToken::SentenceStart => Some(SENTENCE_START),
            ContextToken::Word(w) => self.lexicons.get(&lang)?.model_id(w).map(ctx_id),
        }
    }

    /// Context keys `(prev2, prev1)` known to the model
    fn context_keys(&self, lang: Language, ctx: &WordContext) -> (Option<u32>, Option<u32>) {
        let k1 = ctx.prev1.and_then(|t| self.context_key(lang, t));
        let k2 = k1.and(ctx.prev2).and_then(|t| self.context_key(lang, t));
        (k2, k1)
    }

    /// log10 P(word | context), stupid backoff: trigram, else 0.4 × bigram, else 0.16 × unigram.
    /// Learned pairs take precedence when they are more likely.
    pub fn context_log10(&self, lang: Language, ctx: &WordContext, word: &str) -> f32 {
        let unigram = unigram_log10(self.get_word_frequency_for_lang(word, lang));
        let mut best = self.baseline_log10(ctx, unigram);

        if let (Some(lex), Some(ngrams)) = (self.lexicons.get(&lang), self.ngrams.get(&lang)) {
            if let Some(id) = lex.model_id(word) {
                let (k2, k1) = self.context_keys(lang, ctx);
                let trigram = k2.zip(k1).and_then(|(a, b)| ngrams.trigrams.q(trigram_key(a, b), id));
                let bigram = k1.and_then(|k| ngrams.bigrams.q(k as u64, id));
                if let Some(q) = trigram {
                    best = best.max(dequantize_log10(q));
                } else if let Some(q) = bigram {
                    let backoff = if ctx.prev2.is_some() { BACKOFF_LOG10 } else { 0.0 };
                    best = best.max(backoff + dequantize_log10(q));
                }
            }
        }

        if let Some(prev) = ctx.prev_word() {
            if let Some(p) = self.personal_log10(prev, word) {
                best = best.max(p);
            }
        }
        best
    }

    /// Score a word gets when the context says nothing about it.
    fn baseline_log10(&self, ctx: &WordContext, unigram: f32) -> f32 {
        let orders = match (ctx.prev1.is_some(), ctx.prev2.is_some()) {
            (false, _) => 0.0,
            (true, false) => 1.0,
            (true, true) => 2.0,
        };
        unigram + orders * BACKOFF_LOG10
    }

    /// How much the context raises a word above its context-free score, in decades (>= 0).
    pub fn context_gain(&self, lang: Language, ctx: &WordContext, word: &str) -> f32 {
        if ctx.is_empty() {
            return 0.0;
        }
        let unigram = unigram_log10(self.get_word_frequency_for_lang(word, lang));
        (self.context_log10(lang, ctx, word) - self.baseline_log10(ctx, unigram)).max(0.0)
    }

    fn personal_log10(&self, prev: &str, word: &str) -> Option<f32> {
        let count = self.adaptive_dict.bigram_count(prev, word)?;
        Some(personal_score(count, self.adaptive_dict.continuation_total(prev)))
    }

    /// Most likely next words for `ctx`, best first, as canonical spellings.
    pub fn predict_next_words(&self, lang: Language, ctx: &WordContext, limit: usize) -> Vec<String> {
        if ctx.is_empty() || limit == 0 {
            return Vec::new();
        }
        // lowercase -> (spelling, score)
        let mut scores: HashMap<String, (String, f32)> = HashMap::new();
        let mut offer = |word: &str, score: f32| {
            let lower = word.to_lowercase();
            let slot = scores.entry(lower).or_insert_with(|| (word.to_string(), f32::MIN));
            if score > slot.1 {
                slot.1 = score;
            }
        };

        if let (Some(lex), Some(ngrams)) = (self.lexicons.get(&lang), self.ngrams.get(&lang)) {
            let (k2, k1) = self.context_keys(lang, ctx);
            if let (Some(a), Some(b)) = (k2, k1) {
                for &(id, q) in ngrams.trigrams.get(trigram_key(a, b)) {
                    if let Some(w) = lex.word_by_model_id(id) {
                        offer(w, dequantize_log10(q));
                    }
                }
            }
            if let Some(b) = k1 {
                let backoff = if ctx.prev2.is_some() { BACKOFF_LOG10 } else { 0.0 };
                for &(id, q) in ngrams.bigrams.get(b as u64) {
                    if let Some(w) = lex.word_by_model_id(id) {
                        offer(w, backoff + dequantize_log10(q));
                    }
                }
            }
        }

        if let Some(prev) = ctx.prev_word() {
            let total = self.adaptive_dict.continuation_total(prev);
            for (w, _) in self.adaptive_dict.next_words(prev) {
                let count = self.adaptive_dict.bigram_count(prev, w).unwrap_or(0);
                // Keep the dictionary's canonical spelling ("Москва") for learned lowercase words
                let spelling = self
                    .lexicons
                    .get(&lang)
                    .and_then(|l| l.model_id(w).and_then(|id| l.word_by_model_id(id)))
                    .unwrap_or(w);
                offer(spelling, personal_score(count, total));
            }
        }

        let mut ranked: Vec<(String, f32)> = scores
            .into_iter()
            .filter(|(lower, _)| {
                !self.removed_words.contains(lower)
                    && (self.profanity_enabled || !self.profanity.contains(lower.as_str()))
            })
            .map(|(_, v)| v)
            .collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        ranked.into_iter().take(limit).map(|(w, _)| w).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction::lm_data::{quantize, LanguageModelData, NgramTable, WordEntry};

    /// ids: 0 я, 1 иду, 2 в, 3 магазин, 4 кино, 5 школу
    fn dict() -> Dictionary {
        let words = ["я", "иду", "в", "магазин", "кино", "школу"]
            .iter()
            .map(|w| WordEntry { word: w.to_string(), count: 10, freq: 1500 })
            .collect();
        let bigrams = NgramTable::from_triples(vec![
            (ctx_id(2) as u64, 4, quantize(0.3)),
            (ctx_id(2) as u64, 3, quantize(0.2)),
            (SENTENCE_START as u64, 0, quantize(0.4)),
        ]);
        let trigrams = NgramTable::from_triples(vec![(trigram_key(ctx_id(1), ctx_id(2)), 3, quantize(0.6))]);
        let mut d = Dictionary::new();
        d.install_model(Language::Ukrainian, LanguageModelData { words, bigrams, trigrams });
        d
    }

    const L: Language = Language::Ukrainian;

    #[test]
    fn trigram_context_beats_bigram_context() {
        let d = dict();
        let bigram_only = WordContext::after(Some("в"));
        assert_eq!(d.predict_next_words(L, &bigram_only, 2), vec!["кино", "магазин"]);

        let trigram = WordContext {
            prev1: Some(ContextToken::Word("в")),
            prev2: Some(ContextToken::Word("иду")),
        };
        assert_eq!(d.predict_next_words(L, &trigram, 1), vec!["магазин"]);
        assert!(d.context_log10(L, &trigram, "магазин") > d.context_log10(L, &trigram, "кино"));
    }

    #[test]
    fn sentence_start_predicts_first_words() {
        let d = dict();
        let start = WordContext { prev1: Some(ContextToken::SentenceStart), prev2: None };
        assert_eq!(d.predict_next_words(L, &start, 1), vec!["я"]);
    }

    #[test]
    fn context_gain_is_zero_without_evidence() {
        let d = dict();
        let ctx = WordContext::after(Some("в"));
        assert!(d.context_gain(L, &ctx, "кино") > 0.5);
        assert_eq!(d.context_gain(L, &ctx, "школу"), 0.0);
        assert_eq!(d.context_gain(L, &WordContext::default(), "кино"), 0.0);
    }

    #[test]
    fn learned_pairs_outrank_corpus_after_repetition() {
        let mut d = dict();
        let ctx = WordContext::after(Some("в"));
        d.adaptive_dict.learn_bigram("в", "школу");
        assert_ne!(d.predict_next_words(L, &ctx, 1), vec!["школу"], "a single use is not a habit");
        d.adaptive_dict.learn_bigram("в", "школу");
        assert_eq!(d.predict_next_words(L, &ctx, 1), vec!["школу"]);
    }

    #[test]
    fn removed_words_are_never_predicted() {
        let mut d = dict();
        d.remove_user_word("кино");
        assert!(!d.predict_next_words(L, &WordContext::after(Some("в")), 5).contains(&"кино".to_string()));
    }
}
