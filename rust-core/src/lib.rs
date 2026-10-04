pub mod emoji;
pub mod jni_bridge;
pub mod keyboard;
pub mod prediction;
pub mod render;

#[cfg(test)]
mod tests {
    use super::*;
    use keyboard::key::{KeyAction, KeyboardMode};
    use keyboard::layout::{LayoutBuilder, LayoutMetrics};
    use keyboard::state::{Language, ShiftState};
    use keyboard::KeyboardEngine;
    use prediction::PredictionService;
    use render::canvas::{Canvas, Color};
    use render::theme::RynkTheme;
    use render::KeyboardRenderer;

    #[test]
    fn test_layout_generation() {
        let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);
        let ru_keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            Language::Russian,
            ShiftState::Off,
            &metrics,
        );
        assert!(!ru_keys.is_empty());
        assert!(ru_keys.iter().any(|k| k.label == "й"));

        let en_keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            Language::English,
            ShiftState::Off,
            &metrics,
        );
        assert!(!en_keys.is_empty());
        assert!(en_keys.iter().any(|k| k.label == "q"));

        let num_keys = LayoutBuilder::build_layout(
            KeyboardMode::Numbers,
            Language::Russian,
            ShiftState::Off,
            &metrics,
        );
        assert!(num_keys.iter().any(|k| k.label == "1"));
    }

    #[test]
    fn test_prediction_and_autocorrect() {
        let mut service = PredictionService::new();
        let suggestions = service.get_suggestions("прив", None, true);
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("привет")));

        let typo_suggestions = service.get_suggestions("превет", None, true);
        assert!(!typo_suggestions.is_empty());
        assert!(typo_suggestions.iter().any(|s| s == "привет"));

        // Contextual next-word prediction
        let context_suggs = service.get_suggestions("", Some("как"), true);
        assert!(!context_suggs.is_empty());
        assert!(context_suggs.iter().any(|s| s == "дела"));

        service.learn_word("кастомноеслово", true);
        let custom_sugg = service.get_suggestions("кастом", None, true);
        assert!(custom_sugg.iter().any(|s| s == "кастомноеслово"));
    }

    #[test]
    fn test_emoji_shortcuts() {
        let service = PredictionService::new();
        let sugg_smile = service.get_suggestions(":)", None, false);
        assert!(sugg_smile.iter().any(|s| s == "😊"));

        let sugg_heart = service.get_suggestions("<3", None, false);
        assert!(sugg_heart.iter().any(|s| s == "❤️"));

        // Regular word "огонь" must NOT be converted to emoji
        let sugg_fire = service.get_suggestions("огонь", None, true);
        assert!(!sugg_fire.iter().any(|s| s == "🔥"));
    }

    #[test]
    fn test_user_dictionary_crud() {
        let mut service = PredictionService::new();
        service.add_user_word("суперслово", true);
        assert!(service.get_user_words().contains(&"суперслово".to_string()));
        let suggs = service.get_suggestions("суперсл", None, true);
        assert!(suggs.iter().any(|s| s.contains("суперслово")));

        service.remove_user_word("суперслово");
        assert!(!service.get_user_words().contains(&"суперслово".to_string()));
    }

    #[test]
    fn test_no_false_autocorrect_and_undo_backspace() {
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);

        // 1. Typing "ютуб" followed by Space must NOT replace it with "тут"
        for ch in "ютуб".chars() {
            engine.execute_key_action(keyboard::key::KeyAction::Character(ch));
        }
        let _events = engine.state.drain_events();
        engine.execute_key_action(keyboard::key::KeyAction::Space);
        let space_events = engine.state.drain_events();
        let has_delete = space_events.iter().any(|e| {
            matches!(
                e,
                keyboard::state::KeyboardOutputEvent::DeleteSurroundingText { .. }
            )
        });
        assert!(
            !has_delete,
            "Typing 'ютуб' + space must NOT delete surrounding text!"
        );
        let has_space_commit = space_events.iter().any(|e| match e {
            keyboard::state::KeyboardOutputEvent::CommitText(s) => s == " ",
            _ => false,
        });
        assert!(has_space_commit, "Must commit normal space for 'ютуб'!");

        // 2. Typing "привет" followed by Space must NOT delete or replace
        for ch in "привет".chars() {
            engine.execute_key_action(keyboard::key::KeyAction::Character(ch));
        }
        let _ = engine.state.drain_events();
        engine.execute_key_action(keyboard::key::KeyAction::Space);
        let privet_space_events = engine.state.drain_events();
        let has_delete_privet = privet_space_events.iter().any(|e| {
            matches!(
                e,
                keyboard::state::KeyboardOutputEvent::DeleteSurroundingText { .. }
            )
        });
        assert!(
            !has_delete_privet,
            "Typing 'привет' + space must NOT delete surrounding text!"
        );

        // 3. Typing clear typo "превет" followed by space -> autocorrects to "привет "
        for ch in "превет".chars() {
            engine.execute_key_action(keyboard::key::KeyAction::Character(ch));
        }
        let _ = engine.state.drain_events();
        engine.execute_key_action(keyboard::key::KeyAction::Space);
        let typo_events = engine.state.drain_events();
        let has_correct = typo_events.iter().any(|e| match e {
            keyboard::state::KeyboardOutputEvent::CommitText(s) => s.contains("привет"),
            _ => false,
        });
        assert!(has_correct, "Typo 'превет' should autocorrect to 'привет '");

        // 4. FlorisBoard feature: Immediate backspace after autocorrect restores original "превет "
        engine.execute_key_action(keyboard::key::KeyAction::Backspace);
        let undo_events = engine.state.drain_events();
        let has_restored = undo_events.iter().any(|e| match e {
            keyboard::state::KeyboardOutputEvent::CommitText(s) => s == "превет ",
            _ => false,
        });
        assert!(
            has_restored,
            "Backspace must undo autocorrect and restore original 'превет '"
        );
        assert_eq!(engine.state.last_committed_word, "превет");

        // 5. FlorisBoard feature: Hitting Space again after undo must NOT re-autocorrect!
        engine.execute_key_action(keyboard::key::KeyAction::Space);
        let re_space_events = engine.state.drain_events();
        let re_delete = re_space_events.iter().any(|e| {
            matches!(
                e,
                keyboard::state::KeyboardOutputEvent::DeleteSurroundingText { .. }
            )
        });
        assert!(
            !re_delete,
            "Space after undo must NOT delete or autocorrect again!"
        );
        let re_space = re_space_events.iter().any(|e| match e {
            keyboard::state::KeyboardOutputEvent::CommitText(s) => s == " ",
            _ => false,
        });
        assert!(re_space, "Space after undo must commit normal space!");
        assert!(
            engine.prediction.dictionary.contains_word("превет", true),
            "Rejected autocorrect word must be learned into user dictionary!"
        );

        // 6. FlorisBoard feature: Removing word from dictionary (blacklist/forget word)
        engine.prediction.remove_user_word("превет");
        assert!(
            !engine.prediction.dictionary.contains_word("превет", true),
            "Forgotten word must not be contained!"
        );
        assert_eq!(
            engine
                .prediction
                .dictionary
                .get_word_frequency("превет", true),
            0
        );

        // 7. Abbreviation tests:
        // - All-caps abbreviation: typing "ost" -> "OST", "afk" -> "AFK", "егэ" -> "ЕГЭ"
        // - Lowercase chat abbreviation: typing "хз" -> "хз", "спс" -> "спс"
        // - Mixed-case abbreviation: typing "macos" -> "macOS", "спб" -> "СПб"
        let ost_suggestions = engine.prediction.get_suggestions("ost", None, false);
        assert!(!ost_suggestions.is_empty(), "Must suggest OST!");
        assert_eq!(
            ost_suggestions[1], "OST",
            "Slot 1 (autocorrect/exact) for 'ost' must be canonical uppercase 'OST'!"
        );

        let afk_suggestions = engine.prediction.get_suggestions("afk", None, true);
        assert!(
            !afk_suggestions.is_empty(),
            "Must suggest AFK in Russian layout!"
        );
        assert_eq!(
            afk_suggestions[1], "AFK",
            "Slot 1 for 'afk' in Russian layout must be canonical 'AFK'!"
        );

        let ege_suggestions = engine.prediction.get_suggestions("егэ", None, true);
        assert!(
            !ege_suggestions.is_empty(),
            "Must suggest ЕГЭ in Russian layout!"
        );
        assert_eq!(ege_suggestions[1], "ЕГЭ", "Slot 1 for 'егэ' must be 'ЕГЭ'!");

        let xz_suggestions = engine.prediction.get_suggestions("хз", None, true);
        assert!(!xz_suggestions.is_empty(), "Must suggest хз!");
        assert_eq!(
            xz_suggestions[1], "хз",
            "Slot 1 for 'хз' must be canonical lowercase 'хз'!"
        );

        let macos_suggestions = engine.prediction.get_suggestions("macos", None, false);
        assert!(!macos_suggestions.is_empty(), "Must suggest macOS!");
        assert_eq!(
            macos_suggestions[1], "macOS",
            "Slot 1 for 'macos' must be canonical mixed-case 'macOS'!"
        );

        let spb_suggestions = engine.prediction.get_suggestions("спб", None, true);
        assert!(!spb_suggestions.is_empty(), "Must suggest СПб!");
        assert_eq!(
            spb_suggestions[1], "СПб",
            "Slot 1 for 'спб' must be canonical mixed-case 'СПб'!"
        );
    }

    #[test]
    fn test_autocorrect_typos() {
        use keyboard::key::KeyAction;
        use keyboard::state::KeyboardOutputEvent;
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);

        let test_cases = [
            ("превет", "привет"),
            ("спосибо", "спасибо"),
            ("пожалуста", "пожалуйста"),
            ("кароче", "короче"),
            ("зделал", "сделал"),
            ("ашибка", "ошибка"),
            ("хрошо", "хорошо"),
            ("севодня", "сегодня"),
            ("вобще", "вообще"),
            ("вопще", "вообще"),
            ("лудше", "лучше"),
            ("нравитса", "нравится"),
            ("чево", "чего"),
            ("каво", "кого"),
            ("помойму", "по-моему"),
            ("thnaks", "thanks"),
            ("teh", "the"),
            ("definately", "definitely"),
        ];

        for (typo, expected) in test_cases {
            engine.state.composing_text.clear();
            engine.state.last_committed_word.clear();
            let _ = engine.state.drain_events();

            let is_en = typo.chars().all(|c| c.is_ascii_alphabetic());
            let lang = if is_en {
                Language::English
            } else {
                Language::Russian
            };
            engine.state.language = lang;

            for ch in typo.chars() {
                engine.execute_key_action(KeyAction::Character(ch));
            }
            let _ = engine.state.drain_events();

            let suggestions = engine.prediction.get_suggestions_for_lang(typo, None, lang);
            println!("Typo '{}': suggestions = {:?}", typo, suggestions);
            assert!(
                suggestions.len() >= 2,
                "Must produce suggestions for '{}'",
                typo
            );
            assert_eq!(
                suggestions[1].to_lowercase(),
                expected.to_lowercase(),
                "Center chip for '{}' must be '{}', got '{}'",
                typo,
                expected,
                suggestions[1]
            );

            engine.execute_key_action(KeyAction::Space);
            let events = engine.state.drain_events();
            let committed = events.iter().find_map(|e| match e {
                KeyboardOutputEvent::CommitText(s) => Some(s.clone()),
                _ => None,
            });
            let expected_committed = format!("{} ", expected);
            assert_eq!(
                committed,
                Some(expected_committed.clone()),
                "Typo '{}' + space must commit '{}', got {:?}",
                typo,
                expected_committed,
                committed
            );
        }
    }

    #[test]
    fn test_autocorrect_disabled_toggle() {
        use keyboard::key::KeyAction;
        use keyboard::state::KeyboardOutputEvent;
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);

        // Disable autocorrect
        engine.autocorrect_enabled = false;

        for ch in "превет".chars() {
            engine.execute_key_action(KeyAction::Character(ch));
        }
        let _ = engine.state.drain_events();

        engine.execute_key_action(KeyAction::Space);
        let events = engine.state.drain_events();
        let has_delete = events
            .iter()
            .any(|e| matches!(e, KeyboardOutputEvent::DeleteSurroundingText { .. }));
        assert!(
            !has_delete,
            "When autocorrect is disabled, must not delete surrounding text!"
        );
        let has_space = events.iter().any(|e| match e {
            KeyboardOutputEvent::CommitText(s) => s == " ",
            _ => false,
        });
        assert!(
            has_space,
            "When autocorrect is disabled, must commit normal space!"
        );
    }

    #[test]
    fn test_emoji_manager_exit() {
        use emoji::{EmojiManager, EmojiTouchResult};
        use keyboard::touch::TouchAction;
        use keyboard::state::Language;
        let mut mgr = EmojiManager::default();
        let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);

        // Tap on top-left Back button
        let top_exit = mgr.handle_touch_event(TouchAction::Up, 20.0, 20.0, &metrics, Language::Russian);
        assert!(matches!(top_exit, EmojiTouchResult::SwitchToAlphabet));

        // Tap on bottom-left ABC button
        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let bottom_bar_h = 40.0 * dp;
        let abc_y = metrics.total_height - metrics.bottom_bar_height - bottom_bar_h + 10.0;
        let bot_exit = mgr.handle_touch_event(TouchAction::Up, 30.0, abc_y, &metrics, Language::Russian);
        assert!(matches!(bot_exit, EmojiTouchResult::SwitchToAlphabet));
    }

    #[test]
    fn test_keyboard_engine_touch_and_state() {
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
        let first_key = engine.keys[0].clone();
        let (cx, cy) = first_key.center();

        // Down event
        engine.on_touch(keyboard::touch::TouchAction::Down, 0, cx, cy, 100);
        assert_eq!(engine.active_popup_key_id, Some(first_key.id));

        // Up event
        engine.on_touch(keyboard::touch::TouchAction::Up, 0, cx, cy, 150);
        assert_eq!(engine.active_popup_key_id, None);
        assert!(!engine.state.composing_text.is_empty());

        let events = engine.state.drain_events();
        assert!(!events.is_empty());
    }

    #[test]
    fn test_canvas_rendering() {
        let mut buffer = vec![0u32; 400 * 300];
        {
            let mut canvas = Canvas::new(&mut buffer, 400, 300, 400);
            canvas.clear(Color::rgb(22, 24, 29));
            canvas.fill_rounded_rect(10.0, 10.0, 80.0, 40.0, 8.0, Color::rgb(0, 210, 255));
        }
        assert_eq!(buffer[0], Color::rgb(22, 24, 29).to_u32());
        let center_idx = 30 * 400 + 50;
        assert_eq!(buffer[center_idx], Color::rgb(0, 210, 255).to_u32());
    }

    #[test]
    fn test_full_renderer_pipeline() {
        let engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
        let mut renderer = KeyboardRenderer::new(RynkTheme::dark());
        let mut buffer = vec![0u32; 1080 * 800];
        let mut canvas = Canvas::new(&mut buffer, 1080, 800, 1080);
        let suggs = vec!["привет".to_string(), "как".to_string(), "дела".to_string()];

        renderer.render(&mut canvas, &engine, 500, &suggs);
        // Verify buffer is not empty
        assert!(buffer.iter().any(|&p| p != 0));
    }

    #[test]
    fn test_shift_double_tap_caps_lock() {
        use keyboard::key::KeyAction;
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
        assert_eq!(engine.state.shift_state, ShiftState::Off);
        let shift_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Shift))
            .unwrap();
        assert_eq!(shift_key.label, "⇧");

        // Tap 1
        engine.last_interaction_time_ms = 1000;
        engine.execute_key_action(KeyAction::Shift);
        assert_eq!(engine.state.shift_state, ShiftState::Shifted);
        let shift_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Shift))
            .unwrap();
        assert_eq!(shift_key.label, "⬆");

        // Tap 2 within 200ms -> CapsLock!
        engine.last_interaction_time_ms = 1200;
        engine.execute_key_action(KeyAction::Shift);
        assert_eq!(engine.state.shift_state, ShiftState::CapsLock);
        let shift_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Shift))
            .unwrap();
        assert_eq!(shift_key.label, "⇪");

        // Type a letter -> should stay in CapsLock
        engine.execute_key_action(KeyAction::Character('A'));
        assert_eq!(engine.state.shift_state, ShiftState::CapsLock);
        let shift_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Shift))
            .unwrap();
        assert_eq!(shift_key.label, "⇪");

        // Tap Shift again -> turns off CapsLock
        engine.last_interaction_time_ms = 2000;
        engine.execute_key_action(KeyAction::Shift);
        assert_eq!(engine.state.shift_state, ShiftState::Off);
        let shift_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Shift))
            .unwrap();
        assert_eq!(shift_key.label, "⇧");
    }

    #[test]
    fn test_space_double_tap_language_switch() {
        use keyboard::key::KeyAction;
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
        assert_eq!(engine.state.language, Language::Russian);

        // Tap 1 on empty input -> Space
        engine.last_interaction_time_ms = 1000;
        engine.execute_key_action(KeyAction::Space);
        assert_eq!(engine.state.language, Language::Russian);

        // Tap 2 within 200ms -> Language switches to English!
        engine.last_interaction_time_ms = 1200;
        engine.execute_key_action(KeyAction::Space);
        assert_eq!(engine.state.language, Language::English);
    }

    #[test]
    fn test_smart_punctuation_and_gestures() {
        use keyboard::key::KeyAction;
        use keyboard::state::KeyboardOutputEvent;
        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);

        // 1. Type word "привет" and tap Space
        for ch in "привет".chars() {
            engine.execute_key_action(KeyAction::Character(ch));
        }
        engine.execute_key_action(KeyAction::Space);
        let _ = engine.state.drain_events();
        assert!(engine.state.last_char_was_space);

        // 2. Tap Comma: must swallow space and commit ", "
        engine.execute_key_action(KeyAction::Character(','));
        let comma_events = engine.state.drain_events();
        let deleted_space = comma_events.iter().any(|e| {
            matches!(
                e,
                KeyboardOutputEvent::DeleteSurroundingText {
                    before: 1,
                    after: 0
                }
            )
        });
        assert!(deleted_space, "Must swallow space preceding comma!");
        let committed_comma = comma_events.iter().any(|e| match e {
            KeyboardOutputEvent::CommitText(s) => s == ", ",
            _ => false,
        });
        assert!(
            committed_comma,
            "Must commit comma followed by space (', ')!"
        );

        // 3. Dot punctuation: must auto-capitalize next word
        engine.execute_key_action(KeyAction::Character('.'));
        let dot_events = engine.state.drain_events();
        let committed_dot = dot_events.iter().any(|e| match e {
            KeyboardOutputEvent::CommitText(s) => s == ". ",
            _ => false,
        });
        assert!(
            committed_dot,
            "Must commit period followed by space ('. ')!"
        );
        assert_eq!(
            engine.state.shift_state,
            ShiftState::Shifted,
            "Must auto-capitalize after period!"
        );

        // 4. Spacebar drag moves cursor
        let space_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Space))
            .unwrap()
            .clone();
        let (sx, sy) = space_key.center();
        engine.on_touch(keyboard::touch::TouchAction::Down, 1, sx, sy, 100);
        let _ = engine.state.drain_events();
        // Drag right by 35px
        engine.on_touch(keyboard::touch::TouchAction::Move, 1, sx + 35.0, sy, 120);
        let move_events = engine.state.drain_events();
        let has_cursor = move_events
            .iter()
            .any(|e| matches!(e, KeyboardOutputEvent::MoveCursor(_)));
        assert!(has_cursor, "Spacebar drag must emit MoveCursor event!");
        // Release must NOT commit space
        engine.on_touch(keyboard::touch::TouchAction::Up, 1, sx + 35.0, sy, 140);
        let up_events = engine.state.drain_events();
        let has_space = up_events
            .iter()
            .any(|e| matches!(e, KeyboardOutputEvent::CommitText(_)));
        assert!(!has_space, "Space drag release must not commit space!");

        // 5. Backspace swipe left emits DeleteWord
        let bksp_key = engine
            .keys
            .iter()
            .find(|k| matches!(k.action, KeyAction::Backspace))
            .unwrap()
            .clone();
        let (bx, by) = bksp_key.center();
        engine.on_touch(keyboard::touch::TouchAction::Down, 2, bx, by, 200);
        let _ = engine.state.drain_events();
        // Swipe left by 45px
        engine.on_touch(keyboard::touch::TouchAction::Move, 2, bx - 45.0, by, 230);
        let bksp_events = engine.state.drain_events();
        let has_delete_word = bksp_events
            .iter()
            .any(|e| matches!(e, KeyboardOutputEvent::DeleteWord));
        assert!(
            has_delete_word,
            "Backspace swipe left must emit DeleteWord event!"
        );
    }

    #[test]
    fn test_binary_event_serialization() {
        use keyboard::state::{serialize_events_binary, HapticFeedbackType, KeyboardOutputEvent};

        let events = vec![
            KeyboardOutputEvent::CommitText("Hello".to_string()),
            KeyboardOutputEvent::DeleteSurroundingText {
                before: 2,
                after: 0,
            },
            KeyboardOutputEvent::SendKeyEvent(66),
            KeyboardOutputEvent::PerformHaptic(HapticFeedbackType::KeyClick),
            KeyboardOutputEvent::MoveCursor(-3),
            KeyboardOutputEvent::DeleteWord,
            KeyboardOutputEvent::OpenSettings,
            KeyboardOutputEvent::SwitchInputMethod,
            KeyboardOutputEvent::HideKeyboard,
        ];

        let binary = serialize_events_binary(&events);
        assert!(binary.len() >= 4);

        let count = u32::from_le_bytes(binary[0..4].try_into().unwrap());
        assert_eq!(count, 9);

        // Verify first event (CommitText "Hello")
        // Type: 1 (u8), Len: 5 (u32 LE), Payload: "Hello"
        assert_eq!(binary[4], 1);
        let len1 = u32::from_le_bytes(binary[5..9].try_into().unwrap());
        assert_eq!(len1, 5);
        let str1 = std::str::from_utf8(&binary[9..14]).unwrap();
        assert_eq!(str1, "Hello");
    }

    #[test]
    fn test_input_field_mode_privacy_and_behavior() {
        use keyboard::state::InputFieldMode;

        let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);

        // 1. Password mode disables suggestions, learning, and hides clipboard
        engine.set_input_field_mode(InputFieldMode::Password);
        assert!(!engine.state.field_mode.allows_suggestions());
        assert!(!engine.state.field_mode.allows_autocorrect());
        assert!(!engine.state.field_mode.allows_learning());

        // Type 'a' in password field
        engine.state.composing_text.push('a');
        let suggestions = engine.get_or_update_suggestions();
        assert!(
            suggestions.is_empty(),
            "Password field must never show suggestions!"
        );

        // 2. Email mode disables autocorrect but allows suggestions
        engine.set_input_field_mode(InputFieldMode::Email);
        assert!(engine.state.field_mode.allows_suggestions());
        assert!(!engine.state.field_mode.allows_autocorrect());

        // 3. Normal mode allows both
        engine.set_input_field_mode(InputFieldMode::Normal);
        assert!(engine.state.field_mode.allows_suggestions());
        assert!(engine.state.field_mode.allows_autocorrect());
        assert!(engine.state.field_mode.allows_learning());
    }

    #[test]
    fn test_adaptive_dictionary_lifecycle() {
        use prediction::dictionary::AdaptiveDictionary;

        let mut adaptive = AdaptiveDictionary::default();
        assert!(adaptive.enabled);

        // Learn words
        adaptive.learn_word("секретноеслово");
        adaptive.learn_word("секретноеслово");
        adaptive.learn_bigram("привет", "мир");

        assert!(adaptive.learned_words.contains_key("секретноеслово"));
        assert_eq!(
            adaptive.learned_bigrams.get("привет"),
            Some(&vec!["мир".to_string()])
        );

        // Serialize and Deserialize binary roundtrip
        let bytes = adaptive.serialize_binary();
        assert!(!bytes.is_empty());

        let mut restored = AdaptiveDictionary::default();
        restored.deserialize_binary(&bytes);

        assert!(restored.learned_words.contains_key("секретноеслово"));
        assert_eq!(
            restored.learned_bigrams.get("привет"),
            Some(&vec!["мир".to_string()])
        );

        // Test clear
        restored.clear();
        assert!(restored.learned_words.is_empty());
        assert!(restored.learned_bigrams.is_empty());
    }

    #[test]
    fn test_prediction_performance_benchmark() {
        let service = PredictionService::new();

        // Warm up
        let _ = service.get_suggestions("при", None, true);

        let start = std::time::Instant::now();
        let iterations = 1000;
        for _ in 0..iterations {
            let res = service.get_suggestions("прив", None, true);
            assert!(!res.is_empty());
        }
        let elapsed = start.elapsed();
        let avg_micros = elapsed.as_micros() as f64 / iterations as f64;

        // Verify average query is well under 1000 microseconds (1 millisecond)
        assert!(
            avg_micros < 1000.0,
            "Average suggestion query took {:.2} µs, expected < 1000 µs",
            avg_micros
        );
    }

    #[test]
    fn test_morphology_prefixes_and_suffixes() {
        use crate::prediction::morphology::Morphology;
        use crate::prediction::autocorrect::Autocorrect;

        // 1. Prefix tests
        assert!(Morphology::analyze_prefix_match("зделал", "сделал").is_some());
        assert!(Morphology::analyze_prefix_match("прикрасный", "прекрасный").is_some());
        assert!(Morphology::analyze_prefix_match("разсказ", "рассказ").is_some());
        assert!(Morphology::analyze_prefix_match("безполезный", "бесполезный").is_some());
        assert!(Morphology::analyze_prefix_match("unpossible", "impossible").is_some());
        assert!(Morphology::analyze_prefix_match("dissapoint", "disappoint").is_some());

        // 2. Suffix & Ending tests
        assert!(Morphology::analyze_suffix_and_ending("нравитса", "нравится").is_some());
        assert!(Morphology::analyze_suffix_and_ending("делаеш", "делаешь").is_some());
        assert!(Morphology::analyze_suffix_and_ending("знаеш", "знаешь").is_some());
        assert!(Morphology::analyze_suffix_and_ending("runing", "running").is_some());
        assert!(Morphology::analyze_suffix_and_ending("definately", "definitely").is_some());
        assert!(Morphology::analyze_suffix_and_ending("occurance", "occurrence").is_some());

        // 3. Confident corrections
        assert!(Autocorrect::is_confident_correction("зделал", "сделал", 100));
        assert!(Autocorrect::is_confident_correction("прикрасный", "прекрасный", 100));
        assert!(Autocorrect::is_confident_correction("делаеш", "делаешь", 100));
        assert!(Autocorrect::is_confident_correction("runing", "running", 100));
        assert!(Autocorrect::is_confident_correction("definately", "definitely", 100));
    }

    #[test]
    fn test_preposition_context_and_agreement() {
        use crate::prediction::morphology::Morphology;
        use crate::keyboard::state::Language;

        assert!(Morphology::is_preposition("в", Language::Russian));
        assert!(Morphology::is_preposition("на", Language::Russian));
        assert!(Morphology::is_preposition("with", Language::English));
        assert!(Morphology::is_preposition("to", Language::English));
        assert!(!Morphology::is_preposition("собака", Language::Russian));

        let ru_prep_nexts = Morphology::get_preposition_context_predictions("в", Language::Russian);
        assert!(!ru_prep_nexts.is_empty());
        assert!(ru_prep_nexts.contains(&"том") || ru_prep_nexts.contains(&"этом"));

        let en_prep_nexts = Morphology::get_preposition_context_predictions("to", Language::English);
        assert!(!en_prep_nexts.is_empty());
        assert!(en_prep_nexts.contains(&"be") || en_prep_nexts.contains(&"do"));

        assert!(Morphology::matches_preposition_agreement("в", "городе", Language::Russian));
        assert!(Morphology::matches_preposition_agreement("с", "друзьями", Language::Russian));
        assert!(Morphology::matches_preposition_agreement("to", "be", Language::English));
    }

    #[test]
    fn test_clipboard_output_events_and_serialization() {
        use crate::keyboard::state::KeyboardOutputEvent;
        let mut buf = Vec::new();
        let ev1 = KeyboardOutputEvent::ClearClipboard;
        let ev2 = KeyboardOutputEvent::ClipboardPasted("https://rynk.org".to_string());

        ev1.write_to_binary(&mut buf);
        ev2.write_to_binary(&mut buf);

        assert_eq!(buf[0], 10); // ClearClipboard Type 10
        assert_eq!(buf[5], 11); // ClipboardPasted Type 11
    }

    #[test]
    fn test_profanity_modern_words_and_affixes() {
        use crate::prediction::dictionary::Dictionary;
        use crate::keyboard::state::Language;

        let mut dict = Dictionary::new();
        dict.ensure_language_loaded(Language::Russian);
        dict.ensure_language_loaded(Language::English);
        dict.ensure_language_loaded(Language::German);
        dict.ensure_language_loaded(Language::French);
        dict.ensure_language_loaded(Language::Spanish);
        dict.ensure_language_loaded(Language::Polish);
        dict.ensure_language_loaded(Language::Turkish);

        // 1. Profanity test across multiple languages
        assert!(dict.profanity_enabled, "Profanity must be enabled by default");
        assert!(dict.profanity.contains("хуй"));
        assert!(dict.profanity.contains("пиздец"));
        assert!(dict.profanity.contains("fuck"));
        assert!(dict.profanity.contains("bullshit"));
        assert!(dict.profanity.contains("scheiße") || dict.profanity.contains("scheisse"));
        assert!(dict.profanity.contains("merde") || dict.profanity.contains("putain"));
        assert!(dict.profanity.contains("mierda") || dict.profanity.contains("puta"));
        assert!(dict.profanity.contains("kurwa"));
        assert!(dict.profanity.contains("siktir"));

        assert!(dict.contains_word_for_lang("хуй", Language::Russian));
        assert!(dict.contains_word_for_lang("fuck", Language::English));

        // 2. Modern words & abbreviations test
        assert!(dict.contains_word_for_lang("кринж", Language::Russian));
        assert!(dict.contains_word_for_lang("вайб", Language::Russian));
        assert!(dict.contains_word_for_lang("спс", Language::Russian));
        assert!(dict.contains_word_for_lang("пж", Language::Russian));
        assert!(dict.contains_word_for_lang("хз", Language::Russian));
        assert!(dict.contains_word_for_lang("rizz", Language::English));
        assert!(dict.contains_word_for_lang("skibidi", Language::English));
        assert!(dict.contains_word_for_lang("idk", Language::English));
        assert!(dict.contains_word_for_lang("btw", Language::English));
        assert!(dict.contains_word_for_lang("lol", Language::English));

        // 3. Affixes, prefixes, and endings
        assert!(dict.contains_word_for_lang("сделать", Language::Russian));
        assert!(dict.contains_word_for_lang("переписать", Language::Russian));
        assert!(dict.contains_word_for_lang("пойти", Language::Russian));
        assert!(dict.contains_word_for_lang("developer", Language::English));
        assert!(dict.contains_word_for_lang("unmanageable", Language::English) || dict.contains_word_for_lang("uncheck", Language::English));
    }

    #[test]
    fn test_underscore_symbol_availability() {
        let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);

        // 1. Check Symbols mode: '_' is a direct key
        let sym_keys = LayoutBuilder::build_layout(
            KeyboardMode::Symbols,
            Language::English,
            ShiftState::Off,
            &metrics,
        );
        let underscore_key = sym_keys.iter().find(|k| k.label == "_");
        assert!(underscore_key.is_some(), "Underscore '_' must be present in symbols layout");
        let u_key = underscore_key.unwrap();
        assert_eq!(u_key.action, KeyAction::Character('_'));

        // 2. Check Numbers mode: '-' has '_' in alternates
        let num_keys = LayoutBuilder::build_layout(
            KeyboardMode::Numbers,
            Language::English,
            ShiftState::Off,
            &metrics,
        );
        let hyphen_key = num_keys.iter().find(|k| k.label == "-");
        assert!(hyphen_key.is_some(), "Hyphen '-' must be present in numbers layout");
        assert!(
            hyphen_key.unwrap().alternate_chars.contains(&'_'),
            "Hyphen '-' must have '_' in alternate chars in numbers mode"
        );

        // 3. Check Russian Alphabet mode: comma and dot have '_' in alternates
        let ru_keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            Language::Russian,
            ShiftState::Off,
            &metrics,
        );
        let ru_comma = ru_keys.iter().find(|k| k.label == ",").unwrap();
        let ru_dot = ru_keys.iter().find(|k| k.label == ".").unwrap();
        assert!(ru_comma.alternate_chars.contains(&'_'));
        assert!(ru_dot.alternate_chars.contains(&'_'));

        // 4. Check English Alphabet mode: comma and dot have '_' in alternates
        let en_keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            Language::English,
            ShiftState::Off,
            &metrics,
        );
        let en_comma = en_keys.iter().find(|k| k.label == ",").unwrap();
        let en_dot = en_keys.iter().find(|k| k.label == ".").unwrap();
        assert!(en_comma.alternate_chars.contains(&'_'));
        assert!(en_dot.alternate_chars.contains(&'_'));
    }

    #[test]
    fn test_invisible_hit_box_gap_coverage() {
        let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);
        let keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            Language::Russian,
            ShiftState::Off,
            &metrics,
        );

        // Find two adjacent keys in row 1 (e.g. 'й' and 'ц')
        let key_j = keys.iter().find(|k| k.label == "й").unwrap();
        let key_c = keys.iter().find(|k| k.label == "ц").unwrap();

        // There is a visible gap between them
        let visual_gap = key_c.x - (key_j.x + key_j.width);
        assert!(visual_gap > 0.0, "There must be a visual gap between keys: {}", visual_gap);

        // The midpoint of the horizontal gap
        let mid_x = (key_j.x + key_j.width + key_c.x) * 0.5;
        let test_y = key_j.center().1;

        // Visual bounds do NOT contain the gap midpoint
        assert!(!key_j.contains_visual(mid_x, test_y));
        assert!(!key_c.contains_visual(mid_x, test_y));

        // Invisible expanded hit-box DOES contain the gap midpoint!
        assert!(
            key_j.contains(mid_x, test_y) || key_c.contains(mid_x, test_y),
            "Invisible hit-box must cover the gap between adjacent keys"
        );

        // Hit-boxes must expand horizontally beyond visual bounds
        assert!(key_j.hit_x <= key_j.x);
        assert!(key_j.hit_width > key_j.width);

        // First key expands to the very left edge of the screen (x = 0)
        assert_eq!(key_j.hit_x, 0.0);
        assert!(key_j.contains(0.0, test_y), "First key must hit at x = 0.0");

        // Top edge test: row 1 hit-box expands up to suggestion bar
        assert!(key_j.hit_y <= metrics.key_area_top);
        assert!(key_j.contains(key_j.center().0, metrics.suggestion_bar_height + 0.1));

        // Test vertical gap between row 1 and row 2:
        // Key in row 1: 'й', Key in row 2: 'ф'
        let key_f = keys.iter().find(|k| k.label == "ф").unwrap();
        let vertical_gap = key_f.y - (key_j.y + key_j.height);
        assert!(vertical_gap > 0.0, "There must be a vertical gap between rows");

        let mid_y = (key_j.y + key_j.height + key_f.y) * 0.5;
        let test_x = key_j.center().0;
        // Visual bounds do not contain vertical gap midpoint
        assert!(!key_j.contains_visual(test_x, mid_y));
        assert!(!key_f.contains_visual(test_x, mid_y));
        // Hit-box seamlessly contains vertical gap midpoint
        assert!(
            key_j.contains(test_x, mid_y) || key_f.contains(test_x, mid_y),
            "Invisible hit-box must cover vertical gap between rows"
        );
    }
}

