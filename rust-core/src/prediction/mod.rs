pub mod adaptive;
pub mod autocorrect;
pub mod cjk;
pub mod dictionary;
pub mod lexicon;
pub mod lm;
pub mod lm_data;
pub mod model_source;
pub mod morphology;
pub mod suggestions;
pub mod typos;

use dictionary::Dictionary;
use suggestions::SuggestionEngine;

pub struct PredictionService {
    pub dictionary: Dictionary,
}

impl Default for PredictionService {
    fn default() -> Self {
        Self::new()
    }
}

impl PredictionService {
    pub fn new() -> Self {
        Self {
            dictionary: Dictionary::new(),
        }
    }

    pub fn get_suggestions(
        &self,
        input: &str,
        last_word: Option<&str>,
        is_ru: bool,
    ) -> Vec<String> {
        SuggestionEngine::get_suggestions(input, last_word, is_ru, &self.dictionary)
    }

    pub fn get_suggestions_for_lang(
        &self,
        input: &str,
        last_word: Option<&str>,
        lang: crate::keyboard::state::Language,
    ) -> Vec<String> {
        SuggestionEngine::get_suggestions_for_lang(input, last_word, lang, &self.dictionary)
    }

    pub fn get_suggestions_in_context(
        &self,
        input: &str,
        ctx: &lm::WordContext,
        lang: crate::keyboard::state::Language,
    ) -> Vec<String> {
        SuggestionEngine::get_suggestions_in_context(input, ctx, lang, &self.dictionary)
    }

    pub fn set_profanity_enabled(&mut self, enabled: bool) {
        self.dictionary.set_profanity_enabled(enabled);
    }

    pub fn learn_word(&mut self, word: &str, lang: crate::keyboard::state::Language) {
        self.dictionary.learn_word(word, lang);
    }

    pub fn learn_bigram(&mut self, w1: &str, w2: &str) {
        self.dictionary.learn_bigram(w1, w2);
    }

    pub fn add_user_word(&mut self, word: &str, is_ru: bool) {
        self.dictionary.add_user_word(word, is_ru);
    }

    pub fn add_user_word_in(&mut self, word: &str, lang: crate::keyboard::state::Language) {
        self.dictionary.add_user_word_in(word, lang);
    }

    pub fn remove_user_word(&mut self, word: &str) {
        self.dictionary.remove_user_word(word);
    }

    pub fn get_user_words(&self) -> Vec<String> {
        self.dictionary.get_user_words()
    }
}
