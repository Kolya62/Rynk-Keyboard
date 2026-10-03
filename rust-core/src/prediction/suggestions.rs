use super::autocorrect::Autocorrect;
use super::dictionary::Dictionary;
use crate::keyboard::state::Language;

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
        let clean = input.trim().to_lowercase();

        // 1. If input is empty, return contextual next-word predictions based on last_word
        if clean.is_empty() {
            if let Some(lw) = last_word {
                let context_words = dict.get_context_predictions(lw, lang);
                if !context_words.is_empty() {
                    return context_words.into_iter().take(3).collect();
                }
            }
            return Vec::new();
        }

        // 2. Check emoji shortcut
        if let Some(emoji) = Self::check_emoji_shortcut(&clean) {
            return vec![input.to_string(), input.to_string(), emoji.to_string()];
        }

        // 3. Quick typo / phonetic table lookup (instant O(1) canonical correction)
        if let Some(quick) = crate::prediction::typos::get_quick_correction(&clean) {
            let formatted = Self::match_case(input, quick);
            return vec![input.to_string(), formatted, format!("{}...", input)];
        }

        let context_nexts = last_word
            .map(|lw| dict.get_context_predictions(lw, lang))
            .unwrap_or_default();

        let mut target_lang = lang;
        let trie = dict.get_trie(target_lang);
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
                    let alt_trie = dict.get_trie(alt);
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
        {
            let exact = completions.remove(pos);
            let mut top_candidates = Vec::with_capacity(3);

            // Slot 0 (Left): Exact literal input typed by user
            top_candidates.push(input.to_string());

            // Slot 1 (Center): Exact / Diacritic-corrected word
            top_candidates.push(Self::match_case(input, &exact.0));

            // Slot 2 (Right): Second best completion or context prediction
            if !completions.is_empty() {
                top_candidates.push(Self::match_case(input, &completions[0].0));
            } else if !context_nexts.is_empty() {
                top_candidates.push(context_nexts[0].clone());
            } else {
                top_candidates.push(format!("{}...", input));
            }

            return top_candidates;
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
            let ctx_bonus = if context_nexts.iter().any(|c| c.eq_ignore_ascii_case(&comp_lower)) {
                2.5
            } else {
                0.0
            };
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
                let ctx_bonus = if context_nexts.iter().any(|c| c.eq_ignore_ascii_case(w)) {
                    2.5
                } else {
                    0.0
                };
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
            let second_fix = scored.get(1).map(|s| s.0.as_str()).unwrap_or("");

            vec![
                input.to_string(),                 // Left: literal input
                Self::match_case(input, best_fix), // Center: autocorrect / best completion
                if second_fix.is_empty() {
                    format!("{}...", input)
                } else {
                    Self::match_case(input, second_fix)
                },
            ]
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
