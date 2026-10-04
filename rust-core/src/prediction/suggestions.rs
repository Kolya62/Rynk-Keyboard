use super::autocorrect::Autocorrect;
use super::dictionary::Dictionary;
use super::lm::WordContext;
use crate::keyboard::state::Language;

/// Score points per decade of probability the context adds to a candidate
const CONTEXT_WEIGHT: f32 = 1.6;
const MAX_CONTEXT_BONUS: f32 = 4.0;
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

    fn context_bonus(dict: &Dictionary, lang: Language, ctx: &WordContext, word: &str) -> f32 {
        let mut bonus = (dict.context_gain(lang, ctx, word) * CONTEXT_WEIGHT).min(MAX_CONTEXT_BONUS);
        if let Some(lw) = ctx.prev_word() {
            if crate::prediction::morphology::Morphology::matches_preposition_agreement(lw, word, lang) {
                bonus += 3.5;
            }
        }
        bonus
    }

    pub fn get_suggestions_in_context(
        input: &str,
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
        if let Some(quick) = crate::prediction::typos::get_quick_correction(&clean) {
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


        let mut target_lang = lang;
        let trie = dict.get_lexicon(target_lang);
        let mut completions = trie.find_completions(&clean, 16);
        completions.retain(|(w, _)| !dict.removed_words.contains(w));

        if !dict.profanity_enabled {
            completions.retain(|(w, _)| !dict.profanity.contains(w.as_str()));
        }

        // Cross-script fallback: if no completions found in active language,
        // check English for ASCII input or Russian for Cyrillic input
        if completions.is_empty() {
            let alt_lang = if clean.chars().all(|c| c.is_ascii_alphabetic()) {
                Some(Language::English)
            } else if clean
                .chars()
                .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c))
            {
                Some(Language::Russian)
            } else {
                None
            };

            if let Some(alt) = alt_lang {
                if alt != target_lang {
                    let alt_trie = dict.get_lexicon(alt);
                    let mut alt_completions = alt_trie.find_completions(&clean, 16);
                    alt_completions.retain(|(w, _)| !dict.removed_words.contains(w));
                    if !dict.profanity_enabled {
                        alt_completions.retain(|(w, _)| !dict.profanity.contains(w.as_str()));
                    }
                    if !alt_completions.is_empty() {
                        completions = alt_completions;
                        target_lang = alt;
                    }
                }
            }
        }

        // 4. Exact match & Diacritic-equivalent match prioritization:
        // If the user's typed word matches a dictionary word exactly or when ignoring diacritics
        // (e.g. "buna" -> "bună", "dziekuje" -> "dziękuję", "uber" -> "über", "francais" -> "français"),
        // it must ALWAYS be the primary candidate (Slot 1, Center chip)!
        let clean_norm: String = clean.chars().map(Autocorrect::strip_diacritics).collect();
        if let Some(pos) = completions
            .iter()
            .position(|(w, _)| {
                let w_lower = w.to_lowercase();
                if w_lower == clean {
                    return true;
                }
                let w_norm: String = w_lower.chars().map(Autocorrect::strip_diacritics).collect();
                w_norm == clean_norm
            })
            // A rare exact match with a far more frequent near-twin is ranked like a typo
            .filter(|&p| {
                completions[p].0.to_lowercase() != clean
                    || dict.dominant_alternative(&clean, target_lang).is_none()
            })
        {
            let exact = completions.remove(pos);
            let center = Self::match_case(input, &exact.0);

            // Other completions, most likely in this context first
            completions.sort_by(|a, b| {
                let sa = dict.context_log10(target_lang, ctx, &a.0);
                let sb = dict.context_log10(target_lang, ctx, &b.0);
                sb.total_cmp(&sa)
            });
            let mut alternatives = completions
                .iter()
                .map(|(w, _)| Self::match_case(input, w))
                .filter(|w| w.to_lowercase() != center.to_lowercase());

            // Slot 1 (Center) is what space commits. When it is exactly the input, showing the
            // input again on the left wastes a slot: offer another completion instead. Without
            // alternatives only the real candidates are shown (a single chip is centered).
            let left = if center == input { alternatives.next() } else { Some(input.to_string()) };
            return match (left, alternatives.next()) {
                (Some(left), Some(right)) => vec![left, center, right],
                (Some(left), None) => vec![left, center],
                (None, _) => vec![center],
            };
        }

        // 5. Typo, Diacritic, Context, or Prefix matching:
        let mut candidates_map: std::collections::HashMap<String, f32> =
            std::collections::HashMap::new();

        for (comp, freq) in &completions {
            let comp_lower = comp.to_lowercase();
            let is_prefix = comp_lower.starts_with(&clean);
            let score = if is_prefix {
                let remaining = (comp_lower
                    .chars()
                    .count()
                    .saturating_sub(clean.chars().count())) as f32;
                let penalty = remaining * 0.15;
                let freq_weight = (*freq as f32).min(2000.0) / 2000.0 * 2.0;
                let user_bonus = if dict.adaptive_dict.learned_words.contains_key(&comp_lower)
                    || dict.user_dict.contains_key(&comp_lower)
                {
                    3.0
                } else {
                    0.0
                };
                15.0 - penalty + freq_weight + user_bonus
            } else {
                Autocorrect::score_candidate(&clean, &comp_lower, *freq)
            };
            let ctx_bonus = Self::context_bonus(dict, target_lang, ctx, &comp_lower);
            candidates_map.insert(comp.clone(), score + ctx_bonus);
        }

        // Fast fuzzy candidates: query small indexed candidate pool (~50-150 words)
        let fuzzy_pool = dict.get_fuzzy_candidates(&clean, target_lang);
        for &(w, freq) in &fuzzy_pool {
            if dict.removed_words.contains(w)
                || (!dict.profanity_enabled && dict.profanity.contains(w))
            {
                continue;
            }
            let score = Autocorrect::score_candidate(&clean, w, freq);
            if score > 1.5 {
                let ctx_bonus = Self::context_bonus(dict, target_lang, ctx, w);
                let entry = candidates_map.entry(w.to_string()).or_insert(0.0);
                if score + ctx_bonus > *entry {
                    *entry = score + ctx_bonus;
                }
            }
        }

        let mut scored: Vec<(String, f32)> = candidates_map.into_iter().collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        if !scored.is_empty() {
            let best_fix = &scored[0].0;
            // The literal input already occupies the left slot
            let second_fix = scored
                .iter()
                .skip(1)
                .map(|s| s.0.as_str())
                .find(|w| w.to_lowercase() != clean)
                .unwrap_or("");

            let mut chips = vec![
                input.to_string(),                 // Left: literal input
                Self::match_case(input, best_fix), // Center: autocorrect / best completion
            ];
            if !second_fix.is_empty() {
                chips.push(Self::match_case(input, second_fix));
            }
            chips
        } else {
            // No correction found, return literal
            vec![input.to_string()]
        }
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
