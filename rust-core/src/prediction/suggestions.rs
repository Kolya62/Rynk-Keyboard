use super::dictionary::Dictionary;
use super::correction::{observations, AutocorrectStrength};
use super::decoder::KeyGeometry;
use super::lm::WordContext;
use crate::keyboard::state::Language;

const NEXT_WORD_SLOTS: usize = 3;

pub struct SuggestionEngine;

impl SuggestionEngine {
    pub fn get_suggestions(
        input: &str,
        last_word: Option<&str>,
        is_ru: bool,
        dict: &Dictionary,
    ) -> Vec<String> {
        let lang = if is_ru {
            Language::Russian
        } else {
            Language::English
        };
        Self::get_suggestions_for_lang(input, last_word, lang, dict)
    }

    pub fn get_suggestions_for_lang(
        input: &str,
        last_word: Option<&str>,
        lang: Language,
        dict: &Dictionary,
    ) -> Vec<String> {
        Self::get_suggestions_in_context(input, &WordContext::after(last_word), lang, dict)
    }

    /// Next-word predictions for an empty input: model and learned pairs first, then the
    /// preposition-based guesses and fallbacks of `get_context_predictions`.
    fn next_word_predictions(ctx: &WordContext, lang: Language, dict: &Dictionary, limit: usize) -> Vec<String> {
        let mut words = dict.predict_next_words(lang, ctx, limit);
        if let Some(prev) = ctx.prev_word() {
            for w in dict.get_context_predictions(prev, lang) {
                if words.len() >= limit {
                    break;
                }
                if !words.iter().any(|x| x.to_lowercase() == w.to_lowercase()) {
                    words.push(w);
                }
            }
        }
        words
    }

    pub fn get_suggestions_in_context(
        input: &str,
        ctx: &WordContext,
        lang: Language,
        dict: &Dictionary,
    ) -> Vec<String> {
        Self::get_suggestions_typed(input, &[], None, AutocorrectStrength::default(), ctx, lang, dict)
    }

    /// Suggestion chips for the word being typed: `[alternative/input, center, alternative]`,
    /// where the center is what space commits. `touches` are the tap positions of the typed
    /// letters (see `correction::observations`).
    pub fn get_suggestions_typed(
        input: &str,
        touches: &[Option<(f32, f32)>],
        geometry: Option<&KeyGeometry>,
        strength: AutocorrectStrength,
        ctx: &WordContext,
        lang: Language,
        dict: &Dictionary,
    ) -> Vec<String> {
        let clean = input.trim().to_lowercase();

        // 1. If input is empty, return contextual next-word predictions
        if clean.is_empty() {
            return Self::next_word_predictions(ctx, lang, dict, NEXT_WORD_SLOTS);
        }

        // 2. Check emoji shortcut
        // Case-sensitive on purpose: ":D" and ":d" are different shortcuts
        if let Some(emoji) = Self::check_emoji_shortcut(input.trim()) {
            return vec![input.to_string(), input.to_string(), emoji.to_string()];
        }

        // 3. Quick typo / phonetic table lookup (instant O(1) canonical correction)
        if let Some(quick) = crate::prediction::typos::get_quick_correction_for(&clean, lang) {
            let formatted = Self::match_case(input, quick);
            // The table also normalizes casing ("москва" -> "Москва"): nothing to fix if the
            // input already matches
            if formatted != input {
                return vec![input.to_string(), formatted];
            }
        }

        // 3.1 CJK Pinyin / Romaji candidates for Chinese and Japanese
        if matches!(
            lang,
            Language::ChineseSimplified
                | Language::ChineseTraditional
                | Language::Cantonese
                | Language::Japanese
        ) {
            let cjk_cands = crate::prediction::cjk::get_cjk_candidates(&clean, lang);
            if !cjk_cands.is_empty() {
                let mut top = Vec::with_capacity(3);
                top.push(input.to_string());
                top.push(cjk_cands[0].clone());
                if cjk_cands.len() > 1 {
                    top.push(cjk_cands[1].clone());
                }
                return top;
            }
        }


        let obs = observations(input.trim(), touches);
        let mut target_lang = lang;
        let mut ranked = dict.rank_candidates(lang, ctx, &obs, geometry);
        // Cross-script fallback: Latin letters typed while a non-Latin language is active, or
        // Cyrillic while a non-Cyrillic one is
        if ranked.is_empty() {
            let alt = if clean.chars().all(|c| c.is_ascii_alphabetic()) {
                Some(Language::English)
            } else if clean.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
                Some(Language::Russian)
            } else {
                None
            };
            if let Some(alt) = alt.filter(|&a| a != lang) {
                ranked = dict.rank_candidates(alt, ctx, &obs, geometry);
                target_lang = alt;
            }
        }

        // The typed word is a known word: it stays in the center (space keeps it)
        if dict.is_intended_input(&clean, target_lang, &ranked) {
            let canonical = dict
                .get_lexicon(target_lang)
                .canonical(&clean)
                .map(str::to_string)
                .unwrap_or_else(|| clean.clone());
            let center = Self::match_case(input, &canonical);
            let mut alternatives = ranked
                .iter()
                .map(|r| Self::match_case(input, &r.word))
                .filter(|w| w.to_lowercase() != center.to_lowercase());

            // Showing the input again on the left would waste a slot when the center already
            // is the input: offer another candidate instead. Only real candidates are shown.
            let left = if center == input { alternatives.next() } else { Some(input.to_string()) };
            return match (left, alternatives.next()) {
                (Some(left), Some(right)) => vec![left, center, right],
                (Some(left), None) => vec![left, center],
                (None, _) => vec![center],
            };
        }

        // Center is what space will commit when autocorrect is confident, else the best guess
        let choice = dict.choose_correction(target_lang, ctx, &clean, obs.len(), &ranked, strength);
        let center = choice.or_else(|| ranked.first().map(|r| r.word.clone()));
        let Some(center) = center.map(|w| Self::match_case(input, &w)) else {
            return vec![input.to_string()];
        };
        let mut chips = vec![input.to_string(), center.clone()];
        if let Some(right) = ranked
            .iter()
            .map(|r| Self::match_case(input, &r.word))
            .find(|w| w.to_lowercase() != center.to_lowercase() && w.to_lowercase() != clean)
        {
            chips.push(right);
        }
        chips
    }

    pub fn match_case(template: &str, target: &str) -> String {
        let input_all_upper = !template.is_empty()
            && template.chars().any(|c| c.is_alphabetic())
            && template
                .chars()
                .all(|c| !c.is_alphabetic() || c.is_uppercase());
        let input_first_upper = template.chars().next().is_some_and(|c| c.is_uppercase());

        let target_has_upper = target.chars().any(|c| c.is_uppercase());
        let target_has_lower = target.chars().any(|c| c.is_lowercase());

        if target_has_upper && !target_has_lower {
            // Target is an ALL-UPPERCASE abbreviation (e.g. "OST", "API", "ЕГЭ", "ООО", "ГОСТ")
            target.to_string()
        } else if target_has_upper && target_has_lower {
            // Target has MIXED casing (e.g. "macOS", "iOS", "СПб", "СНиП", "ChatGPT", "GitHub")
            if input_all_upper {
                target.to_uppercase()
            } else {
                target.to_string()
            }
        } else {
            // Target is normal lowercase
            if input_all_upper {
                target.to_uppercase()
            } else if input_first_upper {
                let mut chars = target.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            } else {
                target.to_string()
            }
        }
    }

    fn check_emoji_shortcut(text: &str) -> Option<&'static str> {
        match text {
            ":)" | ":-)" => Some("😊"),
            ";)" | ";-)" => Some("😉"),
            ":D" | ":-D" => Some("😃"),
            ":(" | ":-(" => Some("😢"),
            "<3" => Some("❤️"),
            _ => None,
        }
    }
}
