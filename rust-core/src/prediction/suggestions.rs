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
        let lang = if is_ru { Language::Russian } else { Language::English };
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

        let context_nexts = last_word.map(|lw| dict.get_context_predictions(lw, lang)).unwrap_or_default();

        let mut target_lang = lang;
        let trie = dict.get_trie(target_lang);
        let mut completions = trie.find_completions(&clean, 12);
        completions.retain(|(w, _)| !dict.removed_words.contains(w));

        if !dict.profanity_enabled {
            completions.retain(|(w, _)| !dict.profanity.contains(w.as_str()));
        }

        // Cross-script fallback: if no completions found in active language,
        // check English for ASCII input or Russian for Cyrillic input
        if completions.is_empty() {
            let alt_lang = if clean.chars().all(|c| c.is_ascii_alphabetic()) {
                Some(Language::English)
            } else if clean.chars().any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)) {
                Some(Language::Russian)
            } else {
                None
            };

            if let Some(alt) = alt_lang {
                if alt != target_lang {
                    let alt_trie = dict.get_trie(alt);
                    let mut alt_completions = alt_trie.find_completions(&clean, 12);
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

        // 4. Exact match prioritization:
        // If the user's typed word matches a dictionary word case-insensitively,
        // it must ALWAYS be the primary candidate (Slot 1, Center chip)!
        if let Some(pos) = completions.iter().position(|(w, _)| w.to_lowercase() == clean) {
            let exact = completions.remove(pos);
            let mut top_candidates = Vec::with_capacity(3);

            // Slot 0 (Left): Exact literal input typed by user
            top_candidates.push(input.to_string());

            // Slot 1 (Center): Exact word (canonical casing)
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

        // 5. Typo or Prefix matching:
        // Combine prefix completions and fuzzy autocorrect, ranking by score_candidate
        let mut candidates_map: std::collections::HashMap<String, f32> = std::collections::HashMap::new();

        for (comp, freq) in &completions {
            let comp_lower = comp.to_lowercase();
            let score = if comp_lower.starts_with(&clean) {
                let remaining = (comp_lower.chars().count().saturating_sub(clean.chars().count())) as f32;
                let penalty = remaining * 0.15;
                let freq_weight = (*freq as f32).min(1500.0) / 1500.0 * 1.5;
                4.0 - penalty + freq_weight
            } else {
                Autocorrect::score_candidate(&clean, &comp_lower, *freq)
            };
            let ctx_bonus = if context_nexts.iter().any(|c| c.to_lowercase() == comp_lower) { 0.8 } else { 0.0 };
            candidates_map.insert(comp.clone(), score + ctx_bonus);
        }

        let wordlist = dict.get_word_list(target_lang);
        for &(w, freq) in wordlist {
            if dict.removed_words.contains(w) || (!dict.profanity_enabled && dict.profanity.contains(w)) {
                continue;
            }
            let score = Autocorrect::score_candidate(&clean, &w.to_lowercase(), freq);
            if score > 1.8 {
                let ctx_bonus = if context_nexts.iter().any(|c| c.to_lowercase() == w.to_lowercase()) { 0.8 } else { 0.0 };
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
                input.to_string(), // Left: literal input
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
            && template.chars().all(|c| !c.is_alphabetic() || c.is_uppercase());
        let input_first_upper = template.chars().next().is_some_and(|c| c.is_uppercase());

        let target_has_upper = target.chars().any(|c| c.is_uppercase());
        let target_has_lower = target.chars().any(|c| c.is_lowercase());

        if target_has_upper && !target_has_lower {
            // Target is an ALL-UPPERCASE abbreviation (e.g. "OST", "API", "ЕГЭ", "ООО", "ГОСТ")
            // Always preserve all-caps!
            target.to_string()
        } else if target_has_upper && target_has_lower {
            // Target has MIXED casing (e.g. "macOS", "iOS", "СПб", "СНиП", "ChatGPT", "GitHub")
            if input_all_upper {
                target.to_uppercase()
            } else {
                target.to_string()
            }
        } else {
            // Target is normal lowercase (e.g. "привет", "hello", "etc", "спс", "хз")
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
