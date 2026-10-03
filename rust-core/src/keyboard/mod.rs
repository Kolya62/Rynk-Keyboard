pub mod key;
pub mod layout;
pub mod state;
pub mod touch;

use crate::prediction::PredictionService;
use key::{Key, KeyAction, KeyboardMode};
use layout::{LayoutBuilder, LayoutMetrics};
use state::{HapticFeedbackType, KeyboardOutputEvent, KeyboardState, Language};
use touch::{TouchAction, TouchResult, TouchTracker};

pub struct KeyboardEngine {
    pub state: KeyboardState,
    pub metrics: LayoutMetrics,
    pub keys: Vec<Key>,
    pub touch_tracker: TouchTracker,
    pub active_popup_key_id: Option<u32>,
    pub last_interaction_time_ms: u64,
    pub last_space_tap_time_ms: u64,
    pub last_shift_tap_time_ms: u64,
    pub prediction: PredictionService,
    pub autocorrect_enabled: bool,
    pub popup_enabled: bool,
    pub cached_suggestions: Vec<String>,
    pub suggestions_dirty: bool,
    pub enabled_languages: Vec<Language>,
}

impl KeyboardEngine {
    pub fn new(width: f32, height: f32, density: f32) -> Self {
        let metrics = LayoutMetrics::new(width, height, density);
        let state = KeyboardState::default();
        let keys =
            LayoutBuilder::build_layout(state.mode, state.language, state.shift_state, &metrics);
        let prediction = PredictionService::new();

        Self {
            state,
            metrics,
            keys,
            touch_tracker: TouchTracker::default(),
            active_popup_key_id: None,
            last_interaction_time_ms: 0,
            last_space_tap_time_ms: 0,
            last_shift_tap_time_ms: 0,
            prediction,
            autocorrect_enabled: true,
            popup_enabled: true,
            cached_suggestions: Vec::new(),
            suggestions_dirty: true,
            enabled_languages: vec![Language::Russian, Language::English],
        }
    }

    pub fn resize(&mut self, width: f32, height: f32, density: f32) {
        self.metrics = LayoutMetrics::new(width, height, density);
        self.rebuild_layout();
    }

    pub fn set_input_field_mode(&mut self, mode: state::InputFieldMode) {
        if self.state.field_mode != mode {
            self.state.field_mode = mode;
            if mode.is_password() {
                self.cached_suggestions.clear();
                self.suggestions_dirty = false;
                self.state.composing_text.clear();
                self.state.last_committed_word.clear();
            } else {
                self.suggestions_dirty = true;
            }

            if mode == state::InputFieldMode::Number || mode == state::InputFieldMode::Phone {
                if self.state.mode != KeyboardMode::Numbers {
                    self.set_mode(KeyboardMode::Numbers);
                }
            } else if self.state.mode == KeyboardMode::Numbers {
                self.set_mode(KeyboardMode::Alphabet);
            }
        }
    }

    pub fn rebuild_layout(&mut self) {
        self.keys = LayoutBuilder::build_layout(
            self.state.mode,
            self.state.language,
            self.state.shift_state,
            &self.metrics,
        );
    }

    pub fn get_or_update_suggestions(&mut self) -> &[String] {
        if !self.state.field_mode.allows_suggestions() {
            self.cached_suggestions.clear();
            self.suggestions_dirty = false;
            return &self.cached_suggestions;
        }

        if self.suggestions_dirty {
            self.cached_suggestions = self.prediction.get_suggestions_for_lang(
                &self.state.composing_text,
                if self.state.last_committed_word.is_empty() {
                    None
                } else {
                    Some(&self.state.last_committed_word)
                },
                self.state.language,
            );
            self.suggestions_dirty = false;
        }
        &self.cached_suggestions
    }

    pub fn set_enabled_languages(&mut self, langs: Vec<Language>) {
        if !langs.is_empty() {
            self.enabled_languages = langs;
            self.prediction
                .dictionary
                .ensure_languages_loaded(&self.enabled_languages);
            if !self.enabled_languages.contains(&self.state.language) {
                self.set_language(self.enabled_languages[0]);
            }
        }
    }

    pub fn next_language(&mut self) {
        if self.enabled_languages.is_empty() {
            self.enabled_languages = vec![Language::Russian, Language::English];
        }
        let current_idx = self
            .enabled_languages
            .iter()
            .position(|&l| l == self.state.language)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % self.enabled_languages.len();
        self.set_language(self.enabled_languages[next_idx]);
        self.state.push_event(KeyboardOutputEvent::PerformHaptic(
            HapticFeedbackType::KeyTick,
        ));
    }

    pub fn prev_language(&mut self) {
        if self.enabled_languages.is_empty() {
            self.enabled_languages = vec![Language::Russian, Language::English];
        }
        let current_idx = self
            .enabled_languages
            .iter()
            .position(|&l| l == self.state.language)
            .unwrap_or(0);
        let prev_idx = if current_idx == 0 {
            self.enabled_languages.len() - 1
        } else {
            current_idx - 1
        };
        self.set_language(self.enabled_languages[prev_idx]);
        self.state.push_event(KeyboardOutputEvent::PerformHaptic(
            HapticFeedbackType::KeyTick,
        ));
    }

    pub fn set_language(&mut self, lang: Language) {
        if self.state.language != lang {
            self.state.language = lang;
            self.prediction.dictionary.ensure_language_loaded(lang);
            self.suggestions_dirty = true;
            self.rebuild_layout();
        }
    }

    pub fn toggle_language(&mut self) {
        self.next_language();
    }

    pub fn set_mode(&mut self, mode: KeyboardMode) {
        if self.state.mode != mode {
            self.state.mode = mode;
            self.rebuild_layout();
            self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                HapticFeedbackType::KeyTick,
            ));
        }
    }

    pub fn on_touch(&mut self, action: TouchAction, pointer_id: i32, x: f32, y: f32, time_ms: u64) {
        self.last_interaction_time_ms = time_ms;
        if action == TouchAction::Cancel {
            self.active_popup_key_id = None;
            for key in self.keys.iter_mut() {
                key.is_pressed = false;
            }
        }

        let touch_res = self
            .touch_tracker
            .handle_touch(action, pointer_id, x, y, time_ms, &self.keys);

        match touch_res {
            TouchResult::KeyPress { key_id, haptic } => {
                self.active_popup_key_id = Some(key_id);
                if let Some(key) = self.keys.iter_mut().find(|k| k.id == key_id) {
                    key.is_pressed = true;
                    key.press_time_ms = time_ms;
                }
                self.state
                    .push_event(KeyboardOutputEvent::PerformHaptic(haptic));
            }

            TouchResult::KeyRelease { key_id, action } => {
                self.active_popup_key_id = None;
                if let Some(key) = self.keys.iter_mut().find(|k| k.id == key_id) {
                    key.is_pressed = false;
                }
                self.execute_key_action(action);
            }

            TouchResult::AlternateKeyRelease { character } => {
                self.active_popup_key_id = None;
                for key in self.keys.iter_mut() {
                    key.is_pressed = false;
                }
                self.suggestions_dirty = true;
                let final_ch = if self.state.shift_state.is_uppercase() {
                    character.to_uppercase().next().unwrap_or(character)
                } else {
                    character
                };
                self.execute_key_action(KeyAction::Character(final_ch));
            }

            TouchResult::LongPressTriggered {
                key_id: _,
                alternates: _,
            } => {
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::LongPress,
                ));
            }

            TouchResult::CursorMove { delta } => {
                self.state
                    .push_event(KeyboardOutputEvent::MoveCursor(delta));
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyTick,
                ));
            }

            TouchResult::DeleteWordSwipe => {
                let count = if !self.state.composing_text.is_empty() {
                    let len = self.state.composing_text.chars().count() as u32;
                    self.state.composing_text.clear();
                    len
                } else {
                    0
                };
                self.state.last_committed_word.clear();
                self.suggestions_dirty = true;
                if count > 0 {
                    self.state
                        .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                            before: count,
                            after: 0,
                        });
                } else {
                    self.state.push_event(KeyboardOutputEvent::DeleteWord);
                }
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyHeavyClick,
                ));
            }

            TouchResult::SwitchLanguageSwipe { is_next } => {
                self.active_popup_key_id = None;
                for key in self.keys.iter_mut() {
                    key.is_pressed = false;
                }
                if is_next {
                    self.next_language();
                } else {
                    self.prev_language();
                }
            }

            TouchResult::None => {}
        }
    }

    pub fn tick(&mut self, current_time_ms: u64) {
        if let Some(TouchResult::LongPressTriggered { .. }) = self
            .touch_tracker
            .check_long_press(current_time_ms, &self.keys)
        {
            self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                HapticFeedbackType::LongPress,
            ));
        }
    }

    pub fn execute_key_action(&mut self, action: KeyAction) {
        self.suggestions_dirty = true;
        match action {
            KeyAction::Character(ch) => {
                self.last_space_tap_time_ms = 0;
                self.state.last_autocorrect_original = None;
                self.state.last_autocorrect_replacement = None;
                self.state.rejected_autocorrect_word = None;

                let is_punctuation =
                    ch == '.' || ch == ',' || ch == '!' || ch == '?' || ch == ';' || ch == ':';
                if is_punctuation && self.state.mode == KeyboardMode::Alphabet {
                    // Smart Punctuation (FlorisBoard style):
                    // If preceding character was a space, swallow it before punctuation
                    if self.state.last_char_was_space {
                        self.state
                            .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                                before: 1,
                                after: 0,
                            });
                    }
                    if !self.state.composing_text.is_empty() {
                        self.state.last_committed_word = self.state.composing_text.clone();
                        self.state.composing_text.clear();
                    }
                    // Commit punctuation followed by auto-spacing
                    self.state
                        .push_event(KeyboardOutputEvent::CommitText(format!("{} ", ch)));
                    self.state.last_char_was_space = true;

                    // Auto-capitalize after sentence ending punctuation
                    if ch == '.' || ch == '!' || ch == '?' {
                        self.state.shift_state = state::ShiftState::Shifted;
                        self.rebuild_layout();
                    }
                } else {
                    self.state.last_char_was_space = false;
                    self.state.composing_text.push(ch);
                    self.state
                        .push_event(KeyboardOutputEvent::CommitText(ch.to_string()));
                    let next_shift = self.state.shift_state.on_char_typed();
                    if next_shift != self.state.shift_state {
                        self.state.shift_state = next_shift;
                        self.rebuild_layout();
                    }
                }
            }

            KeyAction::Shift => {
                let now = self.last_interaction_time_ms;
                let is_double_tap = now.saturating_sub(self.last_shift_tap_time_ms) < 280;

                if self.state.shift_state == state::ShiftState::CapsLock {
                    self.state.shift_state = state::ShiftState::Off;
                    self.last_shift_tap_time_ms = 0;
                } else if is_double_tap {
                    self.state.shift_state = state::ShiftState::CapsLock;
                    self.last_shift_tap_time_ms = 0;
                } else {
                    self.state.shift_state = if self.state.shift_state == state::ShiftState::Off {
                        state::ShiftState::Shifted
                    } else {
                        state::ShiftState::Off
                    };
                    self.last_shift_tap_time_ms = now;
                }

                self.rebuild_layout();
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyTick,
                ));
            }

            KeyAction::Backspace => {
                self.last_space_tap_time_ms = 0;
                self.state.last_char_was_space = false;
                if let (Some(orig), Some(repl)) = (
                    self.state.last_autocorrect_original.take(),
                    self.state.last_autocorrect_replacement.take(),
                ) {
                    // Undo autocorrect: restore original typed text + space, keeping cursor after space
                    let repl_len = (repl.encode_utf16().count() + 1) as u32;
                    self.state
                        .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                            before: repl_len,
                            after: 0,
                        });
                    self.state
                        .push_event(KeyboardOutputEvent::CommitText(format!("{} ", orig)));
                    self.state.composing_text.clear();
                    self.state.last_committed_word = orig.clone();
                    self.state.last_char_was_space = true;
                    self.state.rejected_autocorrect_word = Some(orig.to_lowercase());
                    let is_ru = self.state.language == Language::Russian;
                    self.prediction.add_user_word(&orig, is_ru);
                    self.suggestions_dirty = true;
                    self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                        HapticFeedbackType::KeyClick,
                    ));
                    return;
                }

                self.state.last_autocorrect_original = None;
                self.state.last_autocorrect_replacement = None;
                self.state.rejected_autocorrect_word = None;

                if !self.state.composing_text.is_empty() {
                    self.state.composing_text.pop();
                } else {
                    self.state.last_committed_word.clear();
                }
                self.state
                    .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                        before: 1,
                        after: 0,
                    });
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyTick,
                ));
            }

            KeyAction::Enter => {
                self.last_space_tap_time_ms = 0;
                self.state.last_char_was_space = false;
                self.state.last_autocorrect_original = None;
                self.state.last_autocorrect_replacement = None;
                self.state.rejected_autocorrect_word = None;

                if !self.state.composing_text.is_empty() {
                    self.state.last_committed_word = self.state.composing_text.clone();
                    self.state.composing_text.clear();
                }
                if self.state.field_mode == state::InputFieldMode::Multiline {
                    self.state
                        .push_event(KeyboardOutputEvent::CommitText("\n".to_string()));
                } else {
                    self.state.push_event(KeyboardOutputEvent::SendKeyEvent(66));
                    // KEYCODE_ENTER
                }
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyHeavyClick,
                ));
            }

            KeyAction::Space => {
                let now = self.last_interaction_time_ms;
                let is_double_tap = self.last_space_tap_time_ms != 0
                    && now.saturating_sub(self.last_space_tap_time_ms) < 300
                    && self.state.composing_text.is_empty();

                if is_double_tap {
                    // Double tap on spacebar: switch language
                    self.state
                        .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                            before: 1,
                            after: 0,
                        });
                    self.next_language();
                    self.last_space_tap_time_ms = 0;
                    self.state.last_char_was_space = false;
                    self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                        HapticFeedbackType::KeyHeavyClick,
                    ));
                    return;
                }

                self.last_space_tap_time_ms = now;

                if !self.state.composing_text.is_empty() {
                    let clean = self.state.composing_text.trim().to_lowercase();

                    let is_rejected =
                        self.state.rejected_autocorrect_word.as_deref() == Some(&clean);
                    self.state.rejected_autocorrect_word = None;

                    // Check dictionaries + user dictionary + whether autocorrect was rejected:
                    let is_valid_word = is_rejected
                        || self
                            .prediction
                            .dictionary
                            .contains_word_for_lang(&clean, self.state.language);

                    let mut word_to_commit = self.state.composing_text.clone();
                    let mut did_autocorrect = false;

                    if !is_valid_word
                        && self.autocorrect_enabled
                        && self.state.field_mode.allows_autocorrect()
                    {
                        let suggestions = self.prediction.get_suggestions_for_lang(
                            &self.state.composing_text,
                            if self.state.last_committed_word.is_empty() {
                                None
                            } else {
                                Some(&self.state.last_committed_word)
                            },
                            self.state.language,
                        );

                        if suggestions.len() >= 2 {
                            let candidate = &suggestions[1];
                            if !candidate.chars().any(|c| (c as u32) > 0x1F000) {
                                let freq = self
                                    .prediction
                                    .dictionary
                                    .get_word_frequency_for_lang(candidate, self.state.language);
                                if crate::prediction::autocorrect::Autocorrect::is_confident_correction(
                                    &self.state.composing_text,
                                    candidate,
                                    freq,
                                    ) {
                                    word_to_commit = candidate.clone();
                                    did_autocorrect = true;
                                }
                            }
                        }
                    }

                    if did_autocorrect {
                        let before_count = self.state.composing_text.encode_utf16().count() as u32;
                        if before_count > 0 {
                            self.state
                                .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                                    before: before_count,
                                    after: 0,
                                });
                        }
                        self.state
                            .push_event(KeyboardOutputEvent::CommitText(format!(
                                "{} ",
                                word_to_commit
                            )));
                        self.state.last_autocorrect_original =
                            Some(self.state.composing_text.clone());
                        self.state.last_autocorrect_replacement = Some(word_to_commit.clone());
                    } else {
                        // Word was already committed character-by-character!
                        // Simply commit a space without altering or deleting anything!
                        self.state
                            .push_event(KeyboardOutputEvent::CommitText(" ".to_string()));
                        self.state.last_autocorrect_original = None;
                        self.state.last_autocorrect_replacement = None;
                    }

                    if self.state.field_mode.allows_learning() {
                        if !self.state.last_committed_word.is_empty() {
                            self.prediction
                                .learn_bigram(&self.state.last_committed_word, &word_to_commit);
                        }
                        let is_ru = self.state.language == Language::Russian;
                        if is_rejected {
                            self.prediction.add_user_word(&word_to_commit, is_ru);
                        } else if is_valid_word || did_autocorrect {
                            self.prediction.learn_word(&word_to_commit, is_ru);
                        }
                    }
                    self.state.last_committed_word = word_to_commit;
                    self.state.composing_text.clear();
                    self.state.last_char_was_space = true;
                } else {
                    self.state.last_autocorrect_original = None;
                    self.state.last_autocorrect_replacement = None;
                    self.state.rejected_autocorrect_word = None;
                    self.state
                        .push_event(KeyboardOutputEvent::CommitText(" ".to_string()));
                    self.state.last_char_was_space = true;
                }
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyTick,
                ));
            }

            KeyAction::SwitchLanguage => {
                self.toggle_language();
            }

            KeyAction::SwitchMode(mode) => {
                self.set_mode(mode);
            }

            KeyAction::SwitchEmoji => {
                self.set_mode(KeyboardMode::Emoji);
            }

            KeyAction::Settings => {
                self.state.push_event(KeyboardOutputEvent::OpenSettings);
            }

            KeyAction::HideKeyboard => {
                self.state.push_event(KeyboardOutputEvent::HideKeyboard);
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyClick,
                ));
            }

            KeyAction::SwitchInputMethod => {
                self.state
                    .push_event(KeyboardOutputEvent::SwitchInputMethod);
                self.state.push_event(KeyboardOutputEvent::PerformHaptic(
                    HapticFeedbackType::KeyClick,
                ));
            }

            KeyAction::None => {}
        }
    }
}
