use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jfloat, jint, jlong, jobject, jstring};
use jni::JNIEnv;
use std::sync::Mutex;

use crate::emoji::{EmojiManager, EmojiTouchResult};
use crate::keyboard::key::{KeyAction, KeyboardMode};
use crate::keyboard::state::{HapticFeedbackType, KeyboardOutputEvent, Language};
use crate::keyboard::touch::TouchAction;
use crate::keyboard::KeyboardEngine;
use crate::render::theme::{RynkTheme, ThemeId};
use crate::render::KeyboardRenderer;

#[cfg(target_os = "android")]
use crate::render::canvas::Canvas;

#[cfg(target_os = "android")]
#[repr(C)]
pub struct AndroidBitmapInfo {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: i32,
    pub flags: u32,
}

#[cfg(target_os = "android")]
#[link(name = "jnigraphics")]
extern "C" {
    pub fn AndroidBitmap_getInfo(
        env: *mut jni::sys::JNIEnv,
        jbitmap: jobject,
        info: *mut AndroidBitmapInfo,
    ) -> i32;

    pub fn AndroidBitmap_lockPixels(
        env: *mut jni::sys::JNIEnv,
        jbitmap: jobject,
        addr_ptr: *mut *mut std::ffi::c_void,
    ) -> i32;

    pub fn AndroidBitmap_unlockPixels(env: *mut jni::sys::JNIEnv, jbitmap: jobject) -> i32;
}

pub struct RynkCore {
    pub engine: KeyboardEngine,
    pub renderer: KeyboardRenderer,
    pub emoji_mgr: EmojiManager,
}

impl RynkCore {
    pub fn new(width: f32, height: f32, density: f32, theme_id: ThemeId) -> Self {
        let engine = KeyboardEngine::new(width, height, density);
        let theme = RynkTheme::from_id(theme_id);
        let renderer = KeyboardRenderer::new(theme);
        let emoji_mgr = EmojiManager::default();

        Self {
            engine,
            renderer,
            emoji_mgr,
        }
    }
}

static CORE_INSTANCE: Mutex<Option<RynkCore>> = Mutex::new(None);

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeInit(
    _env: JNIEnv,
    _class: JClass,
    width: jfloat,
    height: jfloat,
    density: jfloat,
    theme_id: jint,
) {
    let theme_enum = match theme_id {
        1 => ThemeId::Light,
        2 => ThemeId::Amoled,
        3 => ThemeId::Sunset,
        _ => ThemeId::Dark,
    };

    let core = RynkCore::new(width, height, density, theme_enum);
    let mut guard = CORE_INSTANCE.lock().unwrap();
    *guard = Some(core);
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeDestroy(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    *guard = None;
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeResize(
    _env: JNIEnv,
    _class: JClass,
    width: jfloat,
    height: jfloat,
    density: jfloat,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.resize(width, height, density);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetTheme(
    _env: JNIEnv,
    _class: JClass,
    theme_id: jint,
) {
    let theme_enum = match theme_id {
        1 => ThemeId::Light,
        2 => ThemeId::Amoled,
        3 => ThemeId::Sunset,
        _ => ThemeId::Dark,
    };

    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.renderer.set_theme(RynkTheme::from_id(theme_enum));
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetLanguage(
    _env: JNIEnv,
    _class: JClass,
    lang_id: jint,
) {
    let lang = Language::from_id(lang_id);

    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.set_language(lang);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetEnabledLanguages(
    mut env: JNIEnv,
    _class: JClass,
    lang_codes: JString,
) {
    if let Ok(codes_str) = env.get_string(&lang_codes) {
        let codes: String = codes_str.into();
        let languages: Vec<Language> = codes
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                trimmed
                    .parse::<i32>()
                    .ok()
                    .map(Language::from_id)
                    .or_else(|| Language::from_code(trimmed))
            })
            .collect();

        let mut guard = CORE_INSTANCE.lock().unwrap();
        if let Some(core) = guard.as_mut() {
            core.engine.set_enabled_languages(languages);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetProfanityEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.prediction.set_profanity_enabled(enabled != 0);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetAutocorrectEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.autocorrect_enabled = enabled != 0;
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetMode(
    _env: JNIEnv,
    _class: JClass,
    mode_id: jint,
) {
    let mode = match mode_id {
        1 => KeyboardMode::Numbers,
        2 => KeyboardMode::Symbols,
        3 => KeyboardMode::Emoji,
        _ => KeyboardMode::Alphabet,
    };

    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.set_mode(mode);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeTouch(
    _env: JNIEnv,
    _class: JClass,
    action: jint,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
    time_ms: jlong,
) -> jboolean {
    let touch_act = match action {
        0 => TouchAction::Down,
        1 => TouchAction::Up,
        2 => TouchAction::Move,
        3 => TouchAction::Cancel,
        _ => return 0,
    };

    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        // Emoji mode handling
        if core.engine.state.mode == KeyboardMode::Emoji {
            let current_language = core.engine.state.language;
            let res = core.emoji_mgr.handle_touch_event(
                touch_act,
                x,
                y,
                &core.engine.metrics,
                current_language,
            );
            match res {
                EmojiTouchResult::SelectEmoji(em) => {
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::CommitText(em.to_string()));
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                }
                EmojiTouchResult::SwitchToAlphabet => {
                    core.engine.set_mode(KeyboardMode::Alphabet);
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                }
                EmojiTouchResult::Backspace => {
                    core.engine.execute_key_action(KeyAction::Backspace);
                }
                EmojiTouchResult::Space => {
                    core.engine.execute_key_action(KeyAction::Space);
                }
                EmojiTouchResult::OpenSearch | EmojiTouchResult::CloseSearch => {}
                EmojiTouchResult::Haptic(h) => {
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(h));
                }
                EmojiTouchResult::None => {}
            }
            return 1;
        }

        // Check suggestion bar tap on Up
        if touch_act == TouchAction::Up && y < core.engine.metrics.suggestion_bar_height {
            let total_w = core.engine.metrics.total_width;
            let dp = (core.engine.metrics.suggestion_bar_height / 44.0).max(1.0);

            if !core.engine.state.field_mode.allows_suggestions() {
                return 1;
            }

            let suggestions = core.engine.get_or_update_suggestions().to_vec();

            if !suggestions.is_empty() {
                let num_cands = suggestions.len().min(3);
                let chip_w = total_w / num_cands as f32;
                let idx = (x / chip_w) as usize;

                if let Some(word) = suggestions.get(idx) {
                    let before_count =
                        core.engine.state.composing_text.encode_utf16().count() as u32;
                    if before_count > 0 {
                        core.engine
                            .state
                            .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                                before: before_count,
                                after: 0,
                            });
                    }
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::CommitText(format!("{} ", word)));

                    // FlorisBoard Undo behavior: if suggestion replaced raw typing, allow Backspace to undo!
                    if !core.engine.state.composing_text.is_empty()
                        && &core.engine.state.composing_text != word
                    {
                        core.engine.state.last_autocorrect_original =
                            Some(core.engine.state.composing_text.clone());
                        core.engine.state.last_autocorrect_replacement = Some(word.clone());
                    } else {
                        core.engine.state.last_autocorrect_original = None;
                        core.engine.state.last_autocorrect_replacement = None;
                    }
                    core.engine.state.rejected_autocorrect_word = None;

                    if core.engine.state.field_mode.allows_learning() {
                        if idx == 0 {
                            // User explicitly tapped literal typed word (FlorisBoard behavior: prioritize/learn user word)
                            core.engine.prediction.add_user_word(
                                word,
                                core.engine.state.language == Language::Russian,
                            );
                        } else {
                            core.engine
                                .prediction
                                .learn_word(word, core.engine.state.language == Language::Russian);
                        }

                        if !core.engine.state.last_committed_word.is_empty() {
                            core.engine
                                .prediction
                                .learn_bigram(&core.engine.state.last_committed_word, word);
                        }
                    }
                    core.engine.state.last_committed_word = word.clone();
                    core.engine.state.composing_text.clear();
                    core.engine.state.last_char_was_space = true;
                    core.engine.suggestions_dirty = true;
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                    return 1;
                }
            } else if let Some(clip_text) = core.engine.state.clipboard_preview.clone() {
                let settings_w = 40.0 * dp;
                if x <= settings_w {
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::OpenSettings);
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                    return 1;
                } else {
                    core.engine.state.clipboard_preview = None;
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::CommitText(clip_text));
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                    return 1;
                }
            } else {
                let settings_w = 40.0 * dp;

                // Settings icon tap on left
                if x <= settings_w {
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::OpenSettings);
                    core.engine
                        .state
                        .push_event(KeyboardOutputEvent::PerformHaptic(
                            HapticFeedbackType::KeyClick,
                        ));
                    return 1;
                }

                // Shortcuts across the remainder of the bar
                let shortcuts = [",", ".", "!", "?", "—", ";", ":"];
                let available_w = total_w - settings_w - 8.0 * dp;
                let item_w = available_w / shortcuts.len() as f32;
                let rel_x = x - (settings_w + 4.0 * dp);
                if rel_x >= 0.0 {
                    let idx = (rel_x / item_w) as usize;
                    if let Some(&sc) = shortcuts.get(idx) {
                        core.engine
                            .state
                            .push_event(KeyboardOutputEvent::CommitText(format!("{} ", sc)));
                        core.engine
                            .state
                            .push_event(KeyboardOutputEvent::PerformHaptic(
                                HapticFeedbackType::KeyTick,
                            ));
                        return 1;
                    }
                }
            }
        }

        // Tap ripple animation trigger
        if touch_act == TouchAction::Down {
            core.renderer.animation_mgr.add_ripple(
                x,
                y,
                40.0 * (core.engine.metrics.suggestion_bar_height / 44.0).max(1.0),
                time_ms as u64,
                core.renderer.theme.ripple_color,
            );
        }

        // Regular keyboard touch
        core.engine
            .on_touch(touch_act, pointer_id, x, y, time_ms as u64);
        return 1;
    }
    0
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeOnTouchEvent(
    env: JNIEnv,
    class: JClass,
    action: jint,
    pointer_id: jint,
    x: jfloat,
    y: jfloat,
    time_ms: jlong,
) -> jboolean {
    Java_org_rynk_keyboard_NativeBridge_nativeTouch(env, class, action, pointer_id, x, y, time_ms)
}

#[no_mangle]
#[allow(unused_mut)]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeRender(
    mut env: JNIEnv,
    _class: JClass,
    bitmap: jobject,
    time_ms: jlong,
) -> jboolean {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.tick(time_ms as u64);

        #[cfg(target_os = "android")]
        unsafe {
            let mut info = AndroidBitmapInfo {
                width: 0,
                height: 0,
                stride: 0,
                format: 0,
                flags: 0,
            };

            let raw_env = env.get_raw();
            if AndroidBitmap_getInfo(raw_env, bitmap, &mut info) != 0 {
                return 0;
            }

            let mut pixels_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
            if AndroidBitmap_lockPixels(raw_env, bitmap, &mut pixels_ptr) != 0
                || pixels_ptr.is_null()
            {
                return 0;
            }

            let stride_u32 = info.stride as usize / 4;
            let total_pixels = stride_u32 * info.height as usize;
            let slice = std::slice::from_raw_parts_mut(pixels_ptr as *mut u32, total_pixels);

            let mut canvas =
                Canvas::new(slice, info.width as usize, info.height as usize, stride_u32);

            if core.engine.state.mode == KeyboardMode::Emoji {
                core.renderer.text_labels.clear();
                core.emoji_mgr.render(
                    &mut canvas,
                    &core.engine.metrics,
                    &core.renderer.theme,
                    &mut core.renderer.text_labels,
                );
                if core.renderer.text_labels != core.renderer.previous_labels {
                    core.renderer.labels_version = core.renderer.labels_version.wrapping_add(1);
                    core.renderer.previous_labels = core.renderer.text_labels.clone();
                }
            } else {
                let suggestions = core.engine.get_or_update_suggestions().to_vec();
                core.renderer
                    .render(&mut canvas, &core.engine, time_ms as u64, &suggestions);
            }

            AndroidBitmap_unlockPixels(raw_env, bitmap);
            return if core.renderer.animation_mgr.has_active_animations() {
                1
            } else {
                0
            };
        }

        #[cfg(not(target_os = "android"))]
        {
            let _ = (env, bitmap, time_ms);
            return 0;
        }
    }
    0
}

#[no_mangle]
#[allow(unused_mut)]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativePollEvents(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        let events = core.engine.state.drain_events();
        if events.is_empty() {
            return env.new_string("").unwrap().into_raw();
        }

        let mut output = String::with_capacity(128);
        for event in events {
            match event {
                KeyboardOutputEvent::CommitText(text) => {
                    output.push_str(&format!("COMMIT\t{}\n", text));
                }
                KeyboardOutputEvent::DeleteSurroundingText { before, after } => {
                    output.push_str(&format!("DELETE\t{}\t{}\n", before, after));
                }
                KeyboardOutputEvent::SendKeyEvent(code) => {
                    output.push_str(&format!("KEY\t{}\n", code));
                }
                KeyboardOutputEvent::PerformHaptic(haptic) => {
                    let h_code = match haptic {
                        HapticFeedbackType::KeyTick => 0,
                        HapticFeedbackType::KeyClick => 1,
                        HapticFeedbackType::KeyHeavyClick => 2,
                        HapticFeedbackType::LongPress => 3,
                    };
                    output.push_str(&format!("HAPTIC\t{}\n", h_code));
                }
                KeyboardOutputEvent::MoveCursor(delta) => {
                    output.push_str(&format!("CURSOR\t{}\n", delta));
                }
                KeyboardOutputEvent::DeleteWord => {
                    output.push_str("DELETE_WORD\n");
                }
                KeyboardOutputEvent::OpenSettings => {
                    output.push_str("SETTINGS\n");
                }
                KeyboardOutputEvent::SwitchInputMethod => {
                    output.push_str("SWITCH_IME\n");
                }
                KeyboardOutputEvent::HideKeyboard => {
                    output.push_str("HIDE\n");
                }
            }
        }

        let jstr = env.new_string(output).unwrap();
        return jstr.into_raw();
    }
    env.new_string("").unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeReset(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.state.composing_text.clear();
        core.engine.state.last_committed_word.clear();
        core.engine.suggestions_dirty = true;
        core.engine.active_popup_key_id = None;
        core.engine.touch_tracker.pointers.clear();
    }
}

#[no_mangle]
#[allow(unused_mut)]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeGetTextLabels(
    mut env: JNIEnv,
    _class: JClass,
) -> jstring {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        let labels = &core.renderer.text_labels;
        if labels.is_empty() {
            return env.new_string("").unwrap().into_raw();
        }
        let mut out = String::with_capacity(labels.len() * 40);
        for label in labels {
            let escaped_text = label.text.replace('\t', " ");
            out.push_str(&format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                escaped_text,
                label.cx,
                label.cy,
                label.font_size,
                label.color_r,
                label.color_g,
                label.color_b,
                label.color_a,
                if label.is_bold { 1 } else { 0 },
                label.label_type
            ));
        }
        return env.new_string(out).unwrap().into_raw();
    }
    env.new_string("").unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeHandleBack(
    _env: JNIEnv,
    _class: JClass,
) -> jboolean {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        if core.engine.state.mode == KeyboardMode::Emoji && core.emoji_mgr.is_search_active {
            core.emoji_mgr.close_search();
            return 1;
        }
        if core.engine.state.mode != KeyboardMode::Alphabet {
            core.engine.set_mode(KeyboardMode::Alphabet);
            return 1;
        }
    }
    0
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeIsBackspaceAt(
    _env: JNIEnv,
    _class: JClass,
    x: jfloat,
    y: jfloat,
) -> jboolean {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        if core.engine.state.mode == KeyboardMode::Emoji {
            if core.emoji_mgr.is_search_active {
                // In search mode, no dedicated backspace hold (backspace is inline key)
                return 0;
            }
            let dp = (core.engine.metrics.suggestion_bar_height / 40.0).max(1.0);
            let bottom_bar_h = 40.0 * dp;
            let bot_y = core.engine.metrics.total_height
                - core.engine.metrics.bottom_bar_height
                - bottom_bar_h;
            let bs_w = 56.0 * dp;
            let bs_x = core.engine.metrics.total_width - bs_w - 6.0 * dp;
            if y >= bot_y && y < (bot_y + bottom_bar_h) && x >= bs_x {
                return 1;
            }
        } else {
            if let Some(key) = core.engine.keys.iter().find(|k| k.contains(x, y)) {
                if matches!(key.action, KeyAction::Backspace) {
                    return 1;
                }
            }
        }
    }
    0
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeRepeatBackspace(
    _env: JNIEnv,
    _class: JClass,
    count: jint,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.state.last_autocorrect_original = None;
        core.engine.state.last_autocorrect_replacement = None;
        core.engine.state.rejected_autocorrect_word = None;
        let cnt = (count as u32).max(1);
        for _ in 0..cnt {
            if !core.engine.state.composing_text.is_empty() {
                core.engine.state.composing_text.pop();
            } else {
                core.engine.state.last_committed_word.clear();
            }
        }
        core.engine.suggestions_dirty = true;
        core.engine
            .state
            .push_event(KeyboardOutputEvent::DeleteSurroundingText {
                before: cnt,
                after: 0,
            });
        core.engine
            .state
            .push_event(KeyboardOutputEvent::PerformHaptic(
                HapticFeedbackType::KeyTick,
            ));
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeGetSuggestionAt(
    env: JNIEnv,
    _class: JClass,
    x: jfloat,
    y: jfloat,
) -> jstring {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        let m = &core.engine.metrics;
        if y < m.suggestion_bar_height {
            let suggestions = &core.engine.cached_suggestions;
            if !suggestions.is_empty() {
                let num_cands = suggestions.len().min(3);
                let chip_w = m.total_width / num_cands as f32;
                let idx = (x / chip_w) as usize;
                if let Some(word) = suggestions.get(idx) {
                    if let Ok(js) = env.new_string(word) {
                        return js.into_raw();
                    }
                }
            }
        }
    }
    env.new_string("").unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeGetLabelsVersion(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        return core.renderer.labels_version as jlong;
    }
    0
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeAddUserWord(
    mut env: JNIEnv,
    _class: JClass,
    word: JString,
    is_ru: jboolean,
) {
    if let Ok(w) = env.get_string(&word) {
        let w_str: String = w.into();
        let mut guard = CORE_INSTANCE.lock().unwrap();
        if let Some(core) = guard.as_mut() {
            core.engine.prediction.add_user_word(&w_str, is_ru != 0);
            core.engine.suggestions_dirty = true;
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeRemoveUserWord(
    mut env: JNIEnv,
    _class: JClass,
    word: JString,
) {
    if let Ok(w) = env.get_string(&word) {
        let w_str: String = w.into();
        let mut guard = CORE_INSTANCE.lock().unwrap();
        if let Some(core) = guard.as_mut() {
            core.engine.prediction.remove_user_word(&w_str);
            core.engine.suggestions_dirty = true;
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeGetUserWords(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        let words = core.engine.prediction.get_user_words();
        let joined = words.join("\n");
        return env.new_string(joined).unwrap().into_raw();
    }
    env.new_string("").unwrap().into_raw()
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeTick(
    _env: JNIEnv,
    _class: JClass,
    time_ms: jlong,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.tick(time_ms as u64);
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetClipboardText(
    mut env: JNIEnv,
    _class: JClass,
    text: JString,
) {
    let clip_str = if !text.is_null() {
        env.get_string(&text).ok().map(|s| s.into())
    } else {
        None
    };

    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.state.clipboard_preview = clip_str;
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeGetMode(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_ref() {
        return match core.engine.state.mode {
            KeyboardMode::Alphabet => 0,
            KeyboardMode::Numbers => 1,
            KeyboardMode::Symbols => 2,
            KeyboardMode::Emoji => 3,
        };
    }
    0
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativePollEventsBinary(
    env: JNIEnv,
    _class: JClass,
) -> jni::sys::jbyteArray {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        let events = core.engine.state.drain_events();
        if events.is_empty() {
            return std::ptr::null_mut();
        }
        let binary_bytes = crate::keyboard::state::serialize_events_binary(&events);
        if let Ok(byte_array) = env.byte_array_from_slice(&binary_bytes) {
            return byte_array.into_raw();
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetInputFieldMode(
    _env: JNIEnv,
    _class: JClass,
    mode_id: jint,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine
            .set_input_field_mode(crate::keyboard::state::InputFieldMode::from_id(mode_id));
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetPopupEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.popup_enabled = enabled != 0;
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSetAdaptiveLearningEnabled(
    _env: JNIEnv,
    _class: JClass,
    enabled: jboolean,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.prediction.dictionary.adaptive_dict.enabled = enabled != 0;
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeClearAdaptiveData(
    _env: JNIEnv,
    _class: JClass,
) {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine.prediction.dictionary.adaptive_dict.clear();
    }
}

#[no_mangle]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeSaveAdaptiveData(
    env: JNIEnv,
    _class: JClass,
) -> jni::sys::jbyteArray {
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        if core.engine.prediction.dictionary.adaptive_dict.is_dirty {
            let data = core
                .engine
                .prediction
                .dictionary
                .adaptive_dict
                .serialize_binary();
            core.engine.prediction.dictionary.adaptive_dict.is_dirty = false;
            if let Ok(arr) = env.byte_array_from_slice(&data) {
                return arr.into_raw();
            }
        }
    }
    std::ptr::null_mut()
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "system" fn Java_org_rynk_keyboard_NativeBridge_nativeLoadAdaptiveData(
    env: JNIEnv,
    _class: JClass,
    data: jni::objects::JByteArray,
) {
    if data.is_null() {
        return;
    }
    let Ok(byte_vec) = env.convert_byte_array(&data) else {
        return;
    };
    let mut guard = CORE_INSTANCE.lock().unwrap();
    if let Some(core) = guard.as_mut() {
        core.engine
            .prediction
            .dictionary
            .adaptive_dict
            .deserialize_binary(&byte_vec);
    }
}
