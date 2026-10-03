pub mod animation;
pub mod canvas;
pub mod font;
pub mod popup;
pub mod theme;

use animation::AnimationManager;
use canvas::Canvas;
use popup::PopupRenderer;
use theme::RynkTheme;
use crate::keyboard::key::{Key, KeyAction, KeyType};
use crate::keyboard::KeyboardEngine;

#[derive(Clone, Debug, PartialEq)]
pub struct TextLabel {
    pub text: String,
    pub cx: f32,
    pub cy: f32,
    pub font_size: f32,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
    pub color_a: u8,
    pub is_bold: bool,
    pub label_type: u8,  // 0=key_primary, 1=key_secondary, 2=suggestion, 3=popup, 4=brand
}

pub struct KeyboardRenderer {
    pub theme: RynkTheme,
    pub animation_mgr: AnimationManager,
    pub text_labels: Vec<TextLabel>,
    pub previous_labels: Vec<TextLabel>,
    pub labels_version: u64,
}

impl KeyboardRenderer {
    pub fn new(theme: RynkTheme) -> Self {
        Self {
            theme,
            animation_mgr: AnimationManager::default(),
            text_labels: Vec::new(),
            previous_labels: Vec::new(),
            labels_version: 1,
        }
    }

    pub fn set_theme(&mut self, theme: RynkTheme) {
        self.theme = theme;
    }

    pub fn render(
        &mut self,
        canvas: &mut Canvas,
        engine: &KeyboardEngine,
        current_time_ms: u64,
        suggestions: &[String],
    ) {
        let m = &engine.metrics;
        let density = (m.suggestion_bar_height / 44.0).max(1.0);

        self.text_labels.clear();

        // 1. Keyboard background
        canvas.clear(self.theme.bg_color);

        // 2. Suggestion bar
        self.render_suggestion_bar(canvas, engine, density, suggestions);

        // 3. Render all keys
        for key in &engine.keys {
            self.render_key(canvas, key, density, &engine.state);
        }

        // 4. Tap Ripples and interactive animations
        self.animation_mgr.render_and_update(canvas, current_time_ms);

        // 5. Active Key Popup or Long-press strip
        self.render_popups(canvas, engine, density);

        if self.text_labels != self.previous_labels {
            self.labels_version = self.labels_version.wrapping_add(1);
            self.previous_labels = self.text_labels.clone();
        }
    }

    fn render_suggestion_bar(
        &mut self,
        canvas: &mut Canvas,
        engine: &KeyboardEngine,
        density: f32,
        suggestions: &[String],
    ) {
        let m = &engine.metrics;
        let bar_h = m.suggestion_bar_height;
        let dp = density.max(1.0);

        // Background
        canvas.fill_rect(0.0, 0.0, m.total_width, bar_h, self.theme.suggestion_bar_bg);

        // Subtle divider
        canvas.fill_rect(0.0, bar_h - 1.0, m.total_width, 1.0, self.theme.divider_color);

        if !suggestions.is_empty() {
            // Typing mode: 3 symmetric candidate chips spanning the full width
            let num_cands = suggestions.len().min(3);
            let chip_w = m.total_width / num_cands as f32;
            let chip_h = bar_h - 10.0 * dp;
            let chip_y = 5.0 * dp;

            for (i, cand) in suggestions.iter().take(3).enumerate() {
                let chip_x = i as f32 * chip_w;
                let is_center = i == 1 || (num_cands == 1);

                let bg_color = if is_center {
                    self.theme.suggestion_chip_active
                } else {
                    self.theme.suggestion_chip_bg
                };

                let text_color = if is_center {
                    self.theme.suggestion_text_active
                } else {
                    self.theme.suggestion_text
                };

                let pad = 4.0 * dp;
                canvas.fill_rounded_rect(
                    chip_x + pad,
                    chip_y,
                    chip_w - pad * 2.0,
                    chip_h,
                    8.0 * dp,
                    bg_color,
                );

                self.text_labels.push(TextLabel {
                    text: cand.clone(),
                    cx: chip_x + chip_w * 0.5,
                    cy: chip_y + chip_h * 0.5,
                    font_size: 15.0 * dp,
                    color_r: text_color.r,
                    color_g: text_color.g,
                    color_b: text_color.b,
                    color_a: text_color.a,
                    is_bold: is_center,
                    label_type: 2,
                });
            }
        } else if let Some(ref clip_text) = engine.state.clipboard_preview {
            // Idle mode with Clipboard Chip (FlorisBoard / Gboard style)
            let settings_w = 40.0 * dp;

            // Settings icon
            self.text_labels.push(TextLabel {
                text: "⚙".to_string(),
                cx: settings_w * 0.5,
                cy: bar_h * 0.5,
                font_size: 16.0 * dp,
                color_r: self.theme.text_secondary.r,
                color_g: self.theme.text_secondary.g,
                color_b: self.theme.text_secondary.b,
                color_a: self.theme.text_secondary.a,
                is_bold: false,
                label_type: 5,
            });

            let chip_x = settings_w + 4.0 * dp;
            let chip_w = (m.total_width - chip_x - 8.0 * dp).max(60.0);
            let chip_h = bar_h - 10.0 * dp;
            let chip_y = 5.0 * dp;

            canvas.fill_rounded_rect(
                chip_x,
                chip_y,
                chip_w,
                chip_h,
                8.0 * dp,
                self.theme.suggestion_chip_active,
            );

            let preview_display = if clip_text.chars().count() > 28 {
                let s: String = clip_text.chars().take(25).collect();
                format!("📋 Вставить: {}...", s)
            } else {
                format!("📋 Вставить: {}", clip_text)
            };

            self.text_labels.push(TextLabel {
                text: preview_display,
                cx: chip_x + chip_w * 0.5,
                cy: chip_y + chip_h * 0.5,
                font_size: 14.0 * dp,
                color_r: self.theme.suggestion_text_active.r,
                color_g: self.theme.suggestion_text_active.g,
                color_b: self.theme.suggestion_text_active.b,
                color_a: self.theme.suggestion_text_active.a,
                is_bold: true,
                label_type: 2,
            });
        } else {
            // Idle mode: Settings on left, clean punctuation shortcuts across the rest of the bar
            let settings_w = 40.0 * dp;

            // Settings icon
            self.text_labels.push(TextLabel {
                text: "⚙".to_string(),
                cx: settings_w * 0.5,
                cy: bar_h * 0.5,
                font_size: 16.0 * dp,
                color_r: self.theme.text_secondary.r,
                color_g: self.theme.text_secondary.g,
                color_b: self.theme.text_secondary.b,
                color_a: self.theme.text_secondary.a,
                is_bold: false,
                label_type: 5,
            });

            let shortcuts = [",", ".", "!", "?", "—", ";", ":"];
            let available_w = m.total_width - settings_w - 8.0 * dp;
            let item_w = available_w / shortcuts.len() as f32;
            let start_x = settings_w + 4.0 * dp;

            for (i, &sc) in shortcuts.iter().enumerate() {
                let cx = start_x + (i as f32 * item_w) + item_w * 0.5;
                let cy = bar_h * 0.5;
                self.text_labels.push(TextLabel {
                    text: sc.to_string(),
                    cx,
                    cy,
                    font_size: 16.0 * dp,
                    color_r: self.theme.suggestion_text.r,
                    color_g: self.theme.suggestion_text.g,
                    color_b: self.theme.suggestion_text.b,
                    color_a: self.theme.suggestion_text.a,
                    is_bold: false,
                    label_type: 2,
                });
            }
        }
    }

    fn render_key(&mut self, canvas: &mut Canvas, key: &Key, density: f32, state: &crate::keyboard::state::KeyboardState) {
        let dp = density.max(1.0);
        let radius = 7.0 * dp;

        let is_shift_active = matches!(key.action, KeyAction::Shift)
            && state.shift_state != crate::keyboard::state::ShiftState::Off;

        // Key elevation shadow (only if not pressed)
        if !key.is_pressed {
            canvas.draw_drop_shadow(key.x, key.y, key.width, key.height, radius, 2.2 * dp, self.theme.key_shadow);
        }

        // Fill color
        let fill_color = if key.is_pressed {
            self.theme.key_pressed
        } else if is_shift_active {
            self.theme.brand_accent
        } else {
            match key.key_type {
                KeyType::Normal | KeyType::Space => self.theme.key_normal,
                KeyType::Modifier | KeyType::Icon => self.theme.key_modifier,
                KeyType::Accent => self.theme.key_accent,
            }
        };

        canvas.fill_rounded_rect(key.x, key.y, key.width, key.height, radius, fill_color);

        // Text & Icons
        let primary_color = if is_shift_active || matches!(key.key_type, KeyType::Accent) {
            self.theme.key_accent_text
        } else {
            self.theme.text_primary
        };

        let font_size = match key.action {
            KeyAction::Character(_) => 20.0 * dp,
            KeyAction::Space => 13.0 * dp,
            KeyAction::Shift | KeyAction::Backspace | KeyAction::Enter => 18.0 * dp,
            _ => 15.0 * dp,
        };

        // If spacebar, draw current language
        if matches!(key.action, KeyAction::Space) {
            let space_label = state.language.display_name();
            self.text_labels.push(TextLabel {
                text: space_label.to_string(),
                cx: key.center().0,
                cy: key.center().1,
                font_size: 13.0 * dp,
                color_r: self.theme.text_secondary.r,
                color_g: self.theme.text_secondary.g,
                color_b: self.theme.text_secondary.b,
                color_a: self.theme.text_secondary.a,
                is_bold: false,
                label_type: 4,
            });
        } else {
            let is_icon = matches!(
                key.action,
                KeyAction::Shift | KeyAction::Backspace | KeyAction::Enter | KeyAction::SwitchLanguage | KeyAction::SwitchEmoji
            );
            // Draw main label (rendered via hardware-accelerated SVG vectors or system typography)
            self.text_labels.push(TextLabel {
                text: key.label.clone(),
                cx: key.center().0,
                cy: key.center().1,
                font_size,
                color_r: primary_color.r,
                color_g: primary_color.g,
                color_b: primary_color.b,
                color_a: primary_color.a,
                is_bold: matches!(key.key_type, KeyType::Accent) || matches!(key.action, KeyAction::Character(_)),
                label_type: if is_icon { 5 } else { 0 },
            });
        }


        // Secondary hint label in top right
        if let Some(ref sub) = key.sub_label {
            let sub_x = key.x + key.width - 7.0 * dp;
            let sub_y = key.y + 7.0 * dp;
            self.text_labels.push(TextLabel {
                text: sub.clone(),
                cx: sub_x,
                cy: sub_y,
                font_size: 9.0 * dp,
                color_r: self.theme.text_secondary.r,
                color_g: self.theme.text_secondary.g,
                color_b: self.theme.text_secondary.b,
                color_a: self.theme.text_secondary.a,
                is_bold: false,
                label_type: 1,
            });
        }
    }

    fn render_popups(&mut self, canvas: &mut Canvas, engine: &KeyboardEngine, density: f32) {
        // Check long-press active pointer
        for pointer in &engine.touch_tracker.pointers {
            if pointer.is_long_pressed {
                if let Some(key_id) = pointer.active_key_id {
                    if let Some(key) = engine.keys.iter().find(|k| k.id == key_id) {
                        PopupRenderer::draw_alternate_menu(
                            canvas,
                            key,
                            &key.alternate_chars,
                            pointer.selected_alternate_index,
                            &self.theme,
                            density,
                            &mut self.text_labels,
                        );
                        return;
                    }
                }
            }
        }

        // Normal key preview on press
        if let Some(key_id) = engine.active_popup_key_id {
            if let Some(key) = engine.keys.iter().find(|k| k.id == key_id) {
                // Show popup only for character keys
                if matches!(key.action, KeyAction::Character(_)) {
                    PopupRenderer::draw_preview(canvas, key, &self.theme, density, &mut self.text_labels);
                }
            }
        }
    }
}
