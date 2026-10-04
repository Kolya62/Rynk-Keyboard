//! Ranking of spelling candidates and the autocorrect decision.
//!
//! score(word) = -cost(typed | word) + LM_WEIGHT × log10 P(word | context)
//! where the cost comes from the touch-aware `Decoder` and the probability from the context
//! model. A word that is not in the dictionary competes with a fixed out-of-vocabulary prior.

use super::decoder::{Candidate, Decoder, KeyGeometry, Observation};
use super::dictionary::Dictionary;
use super::lm::{ContextToken, WordContext};
use crate::keyboard::state::Language;

const LM_WEIGHT: f32 = 1.0;
/// log10 prior of "the user meant exactly this unknown word"
const OOV_LOG10: f32 = -7.0;
/// Extra cost of a missing space between two words
const SPLIT_COST: f32 = 1.8;
const MAX_CANDIDATES: usize = 40;
/// Longer input is not a word (held key, pasted text): no decoding
const MAX_WORD_CHARS: usize = 32;
/// log10 boosts for the user's own words: added to the dictionary, or learned from typing
const USER_WORD_BONUS: f32 = 2.0;
const LEARNED_WORD_BONUS: f32 = 1.2;
/// log10 boost for a word that agrees with the preceding preposition
const GRAMMAR_BONUS: f32 = 0.5;

/// Corrections accepted on space: how far the best candidate must beat the typed word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutocorrectStrength {
    /// Maximum decoder cost (per word length) for an automatic replacement
    pub cost_base: f32,
    pub cost_per_char: f32,
    pub cost_cap: f32,
    /// Fix a missing space between two words ("приветкак" -> "привет как")
    pub split_words: bool,
}

impl AutocorrectStrength {
    pub const MILD: Self = Self { cost_base: 0.4, cost_per_char: 0.35, cost_cap: 2.5, split_words: true };
    pub const NORMAL: Self = Self { cost_base: 0.8, cost_per_char: 0.5, cost_cap: 4.0, split_words: true };
    pub const AGGRESSIVE: Self = Self { cost_base: 1.2, cost_per_char: 0.65, cost_cap: 5.0, split_words: true };

    /// Settings level: 1 mild, 2 normal, 3 aggressive (0, autocorrect off, is handled by the engine)
    pub fn from_level(level: i32) -> Self {
        match level {
            1 => Self::MILD,
            3 => Self::AGGRESSIVE,
            _ => Self::NORMAL,
        }
    }

    fn max_cost(&self, chars: usize) -> f32 {
        (self.cost_base + self.cost_per_char * chars as f32).min(self.cost_cap)
    }
}

impl Default for AutocorrectStrength {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ranked {
    /// Canonical spelling; two words separated by a space for a missing-space fix
    pub word: String,
    pub score: f32,
    pub cost: f32,
    pub completed_chars: usize,
}

/// Observations for the typed text; `touches` belong to its last letters (letters adopted from
/// the editor have no touch). Touches that do not fit are ignored.
pub fn observations(text: &str, touches: &[Option<(f32, f32)>]) -> Vec<Observation> {
    let chars: Vec<char> = text.chars().flat_map(char::to_lowercase).collect();
    let usable = touches.len() <= chars.len() && text.chars().count() == chars.len();
    let pad = chars.len() - touches.len().min(chars.len());
    chars
        .iter()
        .enumerate()
        .map(|(i, &ch)| Observation {
            ch,
            point: if usable && i >= pad { touches[i - pad] } else { None },
        })
        .collect()
}

impl Dictionary {
    /// The typed word counts as intended: known, and not a rare twin of a common word.
    pub fn is_valid_input(&self, word: &str, lang: Language) -> bool {
        self.contains_word_for_lang(word, lang) && self.dominant_alternative(word, lang).is_none()
    }

    /// Whether the typed word is what the user meant, given how it was typed: a known word,
    /// unless it is a rare one and the touches fit a far more common word about as well
    /// ("ging" typed for "going").
    pub fn is_intended_input(&self, typed: &str, lang: Language, ranked: &[Ranked]) -> bool {
        if !self.is_valid_input(typed, lang) {
            return false;
        }
        let lower = typed.to_lowercase();
        if self.user_dict.contains_key(&lower) || self.adaptive_dict.learned_words.contains_key(&lower) {
            return true;
        }
        let freq = self.get_word_frequency_for_lang(&lower, lang);
        if freq >= super::dictionary::RARE_WORD_FREQ {
            return true;
        }
        let min_freq = super::dictionary::MIN_DOMINANT_FREQ.max(freq + super::dictionary::DOMINANCE_FREQ_GAP);
        !ranked.iter().any(|r| {
            r.completed_chars <= 1
                && r.cost <= super::dictionary::DOMINANCE_MAX_COST
                && r.word.to_lowercase() != lower
                && self.get_word_frequency_for_lang(&r.word, lang) >= min_freq
        })
    }

    fn decode_candidates(
        &self,
        lang: Language,
        obs: &[Observation],
        geometry: Option<&KeyGeometry>,
        completions: bool,
    ) -> Vec<Candidate> {
        let n = obs.len();
        if n == 0 || n > MAX_WORD_CHARS {
            return Vec::new();
        }
        // Covers the most permissive autocorrect strength; chips beyond it are not useful
        let max_cost = (1.2 + 0.65 * n as f32).min(5.0);
        let lexicon = self.get_lexicon(lang);
        let mut candidates = Decoder::new(lexicon, obs, geometry).decode(max_cost, MAX_CANDIDATES, completions);
        if !lexicon.overlay_entries().is_empty() {
            // User and learned words: a boost for a base word or a word of its own
            for o in Decoder::for_overlay(lexicon, obs, geometry).decode(max_cost, MAX_CANDIDATES, completions) {
                match candidates.iter_mut().find(|c| c.word.to_lowercase() == o.word.to_lowercase()) {
                    Some(c) => {
                        c.freq = c.freq.max(o.freq);
                        c.cost = c.cost.min(o.cost);
                    }
                    None => candidates.push(o),
                }
            }
        }
        candidates
            .into_iter()
            .filter(|c| {
                let lower = c.word.to_lowercase();
                !self.removed_words.contains(&lower)
                    && (self.profanity_enabled || !self.profanity.contains(lower.as_str()))
            })
            .collect()
    }

    /// Preposition agreement ("в Москве", not "в Москва") for languages the morphology knows.
    fn grammar_bonus(&self, lang: Language, ctx: &WordContext, word: &str) -> f32 {
        match ctx.prev_word() {
            Some(prev)
                if super::morphology::Morphology::matches_preposition_agreement(prev, &word.to_lowercase(), lang) =>
            {
                GRAMMAR_BONUS
            }
            _ => 0.0,
        }
    }

    /// The user's own words outrank corpus words of similar fit
    fn personal_bonus(&self, word: &str) -> f32 {
        let lower = word.to_lowercase();
        if self.user_dict.contains_key(&lower) {
            USER_WORD_BONUS
        } else if self.adaptive_dict.learned_words.contains_key(&lower) {
            LEARNED_WORD_BONUS
        } else {
            0.0
        }
    }

    /// Candidates for the typed input ranked by channel cost and context probability, best
    /// first. Includes completions of the typed prefix.
    pub fn rank_candidates(
        &self,
        lang: Language,
        ctx: &WordContext,
        obs: &[Observation],
        geometry: Option<&KeyGeometry>,
    ) -> Vec<Ranked> {
        let mut ranked: Vec<Ranked> = self
            .decode_candidates(lang, obs, geometry, true)
            .into_iter()
            .map(|c| Ranked {
                score: -c.cost
                    + LM_WEIGHT * self.context_log10(lang, ctx, c.word)
                    + self.grammar_bonus(lang, ctx, c.word)
                    + self.personal_bonus(c.word),
                word: c.word.to_string(),
                cost: c.cost,
                completed_chars: c.completed_chars,
            })
            .collect();
        ranked.sort_by(|a, b| b.score.total_cmp(&a.score));
        ranked
    }

    /// Replacement for the typed word when space is pressed, if confident enough. Considers
    /// full-word corrections, near-complete words and a missing space between two words.
    pub fn autocorrect_choice(
        &self,
        lang: Language,
        ctx: &WordContext,
        typed: &str,
        obs: &[Observation],
        geometry: Option<&KeyGeometry>,
        strength: AutocorrectStrength,
    ) -> Option<String> {
        if obs.len() < 2 || self.is_valid_input(typed, lang) {
            return None;
        }
        let ranked = self.rank_candidates(lang, ctx, obs, geometry);
        self.choose_correction(lang, ctx, typed, obs.len(), &ranked, strength)
    }

    /// The autocorrect decision over already ranked candidates (see `autocorrect_choice`).
    pub fn choose_correction(
        &self,
        lang: Language,
        ctx: &WordContext,
        typed: &str,
        n: usize,
        ranked: &[Ranked],
        strength: AutocorrectStrength,
    ) -> Option<String> {
        if !(2..=MAX_WORD_CHARS).contains(&n) || self.is_intended_input(typed, lang, ranked) {
            return None;
        }
        // Hand-curated corrections ("вопще" -> "вообще") are certain
        if let Some(fix) = super::typos::get_quick_correction_for(typed, lang) {
            if fix.to_lowercase() != typed.to_lowercase() {
                return Some(fix.to_string());
            }
        }
        let max_cost = strength.max_cost(n);
        // A word missing its last letter or two is still a correction ("пожалуйст")
        let max_completion = match n {
            0..=3 => 0,
            4..=6 => 1,
            _ => 2,
        };
        let typed_score = LM_WEIGHT * OOV_LOG10;

        let best = ranked
            .iter()
            .filter(|r| r.completed_chars <= max_completion && r.word.to_lowercase() != typed.to_lowercase())
            .find(|r| r.cost <= max_cost && r.score > typed_score)
            .cloned();

        let split = if strength.split_words { self.best_split(lang, ctx, typed, max_cost) } else { None };
        match (best, split) {
            (Some(b), Some(s)) if s.score > b.score => Some(s.word),
            (Some(b), _) => Some(b.word),
            (None, Some(s)) if s.score > typed_score => Some(s.word),
            _ => None,
        }
    }

    /// "приветкак" -> "привет как": both halves must be known words.
    fn best_split(&self, lang: Language, ctx: &WordContext, typed: &str, max_cost: f32) -> Option<Ranked> {
        if SPLIT_COST > max_cost {
            return None;
        }
        let chars: Vec<char> = typed.chars().collect();
        let lexicon = self.get_lexicon(lang);
        let mut best: Option<Ranked> = None;
        for k in 1..chars.len() {
            let left: String = chars[..k].iter().collect();
            let right: String = chars[k..].iter().collect();
            // Both halves must be words (one-letter halves only if "в", "и", "a" are words)
            if !self.is_valid_input(&left, lang) || !self.is_valid_input(&right, lang) {
                continue;
            }
            let l = lexicon.canonical(&left).unwrap_or(&left).to_string();
            let r = lexicon.canonical(&right).unwrap_or(&right).to_string();
            let after_left = WordContext { prev1: Some(ContextToken::Word(&l)), prev2: ctx.prev1 };
            let score = -SPLIT_COST
                + LM_WEIGHT * (self.context_log10(lang, ctx, &l) + self.context_log10(lang, &after_left, &r));
            if best.as_ref().is_none_or(|b| score > b.score) {
                best = Some(Ranked { word: format!("{l} {r}"), score, cost: SPLIT_COST, completed_chars: 0 });
            }
        }
        best
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction::lm_data::{LanguageModelData, WordEntry};

    const L: Language = Language::Ukrainian;

    fn dict(words: &[(&str, u16)]) -> Dictionary {
        let words = words
            .iter()
            .map(|&(w, freq)| WordEntry { word: w.to_string(), count: 10, freq })
            .collect();
        let mut d = Dictionary::new();
        d.install_model(L, LanguageModelData { words, ..Default::default() });
        d
    }

    fn obs(s: &str) -> Vec<Observation> {
        observations(s, &[])
    }

    fn choice(d: &Dictionary, typed: &str) -> Option<String> {
        d.autocorrect_choice(L, &WordContext::default(), typed, &obs(typed), None, AutocorrectStrength::NORMAL)
    }

    #[test]
    fn corrects_typos_but_not_valid_words() {
        let d = dict(&[("привет", 2000), ("как", 2400), ("дела", 2000)]);
        assert_eq!(choice(&d, "превет").as_deref(), Some("привет"));
        assert_eq!(choice(&d, "привет"), None, "valid word");
        assert_eq!(choice(&d, "абырвалг"), None, "nothing close enough");
    }

    #[test]
    fn splits_a_missing_space() {
        let d = dict(&[("привет", 2000), ("как", 2400), ("дела", 2000)]);
        assert_eq!(choice(&d, "приветкак").as_deref(), Some("привет как"));
        assert_eq!(choice(&d, "какдела").as_deref(), Some("как дела"));
    }

    #[test]
    fn touches_align_to_the_last_letters() {
        let o = observations("мага", &[Some((1.0, 2.0))]);
        assert_eq!(o.len(), 4);
        assert!(o[..3].iter().all(|o| o.point.is_none()));
        assert_eq!(o[3].point, Some((1.0, 2.0)));
        // More touches than letters: stale, ignored
        let o = observations("ab", &[Some((1.0, 1.0)); 3]);
        assert!(o.iter().all(|o| o.point.is_none()));
    }
}
