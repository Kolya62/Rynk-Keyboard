pub mod data;
pub mod search;

use crate::keyboard::key::{Key, KeyAction, KeyType, KeyboardMode};
use crate::keyboard::layout::{LayoutBuilder, LayoutMetrics};
use crate::keyboard::state::{HapticFeedbackType, Language, ShiftState};
use crate::keyboard::touch::TouchAction;
use crate::render::canvas::Canvas;
use crate::render::theme::RynkTheme;
use crate::render::TextLabel;
use data::EmojiCategory;

pub struct EmojiManager {
    pub active_category: EmojiCategory,
    pub scroll_offset_y: f32,
    pub touch_start_x: f32,
    pub touch_start_y: f32,
    pub last_touch_y: Option<f32>,
    pub last_touch_x: Option<f32>,
    pub is_dragging: bool,

    // Search state
    pub is_search_active: bool,
    pub search_query: String,
    pub search_results: Vec<&'static str>,
    pub search_scroll_x: f32,
    pub search_language: Language,
    pub search_shift_state: ShiftState,
    pub search_keys: Vec<Key>,
    pub pressed_key_id: Option<u32>,
}

impl Default for EmojiManager {
    fn default() -> Self {
        Self {
            active_category: EmojiCategory::Smileys,
            scroll_offset_y: 0.0,
            touch_start_x: 0.0,
            touch_start_y: 0.0,
            last_touch_y: None,
            last_touch_x: None,
            is_dragging: false,

            is_search_active: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_scroll_x: 0.0,
            search_language: Language::Russian,
            search_shift_state: ShiftState::Off,
            search_keys: Vec::new(),
            pressed_key_id: None,
        }
    }
}

pub enum EmojiTouchResult {
    None,
    SelectEmoji(&'static str),
    SwitchToAlphabet,
    Backspace,
    Space,
    OpenSearch,
    CloseSearch,
    Haptic(HapticFeedbackType),
}

impl EmojiManager {
    pub fn get_cols(&self, metrics: &LayoutMetrics) -> usize {
        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let desired_col_w = 48.0 * dp;
        let cols = (metrics.total_width / desired_col_w).floor() as usize;
        cols.clamp(7, 16)
    }

    pub fn select_category(&mut self, cat: EmojiCategory) {
        self.active_category = cat;
        self.scroll_offset_y = 0.0;
        self.last_touch_y = None;
        self.is_dragging = false;
    }

    pub fn open_search(&mut self, metrics: &LayoutMetrics, language: Language) {
        self.is_search_active = true;
        self.search_language = language;
        self.search_shift_state = ShiftState::Off;
        self.search_query.clear();
        self.search_results = search::DEFAULT_EMOJIS.to_vec();
        self.search_scroll_x = 0.0;
        self.pressed_key_id = None;
        self.rebuild_search_keys(metrics);
    }

    pub fn close_search(&mut self) {
        self.is_search_active = false;
        self.search_query.clear();
        self.search_results.clear();
        self.search_scroll_x = 0.0;
        self.pressed_key_id = None;
    }

    pub fn rebuild_search_keys(&mut self, metrics: &LayoutMetrics) {
        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let header_h = 40.0 * dp;
        let strip_h = 52.0 * dp;
        let top_offset = header_h + strip_h;
        let bottom_bar_h = metrics.bottom_bar_height;
        let padding_bottom = bottom_bar_h + 2.0 * dp;
        let key_area_top = top_offset + 2.0 * dp;
        let key_area_height = (metrics.total_height - key_area_top - padding_bottom).max(10.0);

        let search_metrics = LayoutMetrics {
            total_width: metrics.total_width,
            total_height: metrics.total_height,
            suggestion_bar_height: top_offset,
            key_area_top,
            key_area_height,
            padding_horizontal: metrics.padding_horizontal,
            padding_bottom,
            key_spacing_h: metrics.key_spacing_h,
            key_spacing_v: metrics.key_spacing_v,
            key_border_radius: metrics.key_border_radius,
            bottom_bar_height: bottom_bar_h,
        };

        self.search_keys = LayoutBuilder::build_layout(
            KeyboardMode::Alphabet,
            self.search_language,
            self.search_shift_state,
            &search_metrics,
        );
    }

    fn find_search_key(&self, x: f32, y: f32) -> Option<Key> {
        self.search_keys.iter().find(|k| k.contains(x, y)).cloned()
    }

    pub fn render(
        &mut self,
        canvas: &mut Canvas,
        metrics: &LayoutMetrics,
        theme: &RynkTheme,
        text_labels: &mut Vec<TextLabel>,
    ) {
        if self.is_search_active {
            if self.search_keys.is_empty() {
                self.rebuild_search_keys(metrics);
            }
            self.render_search(canvas, metrics, theme, text_labels);
            return;
        }

        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let tab_bar_h = 40.0 * dp;
        let bottom_bar_h = 40.0 * dp;
        let bot_y = metrics.total_height - metrics.bottom_bar_height - bottom_bar_h;
        let content_y = tab_bar_h;
        let content_h = (bot_y - tab_bar_h).max(10.0);

        // 1. Clear background
        canvas.clear(theme.bg_color);

        // 2. Tab Bar: Exit button on the far left, categories across remainder
        let back_btn_w = 46.0 * dp;
        canvas.fill_rounded_rect(
            4.0 * dp,
            4.0 * dp,
            back_btn_w - 8.0 * dp,
            tab_bar_h - 8.0 * dp,
            6.0 * dp,
            theme.key_modifier,
        );
        text_labels.push(TextLabel {
            text: "←".to_string(),
            cx: back_btn_w * 0.5,
            cy: tab_bar_h * 0.5,
            font_size: 18.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: true,
            label_type: 5,
        });

        let categories = [
            EmojiCategory::Smileys,
            EmojiCategory::Gestures,
            EmojiCategory::Nature,
            EmojiCategory::Food,
            EmojiCategory::Activities,
            EmojiCategory::Travel,
            EmojiCategory::Objects,
            EmojiCategory::Symbols,
            EmojiCategory::Flags,
        ];

        let remaining_w = metrics.total_width - back_btn_w;
        let tab_w = remaining_w / categories.len() as f32;
        for (i, &cat) in categories.iter().enumerate() {
            let tx = back_btn_w + i as f32 * tab_w;
            let is_active = self.active_category == cat;

            if is_active {
                canvas.fill_rounded_rect(
                    tx + 3.0 * dp,
                    4.0 * dp,
                    tab_w - 6.0 * dp,
                    tab_bar_h - 8.0 * dp,
                    6.0 * dp,
                    theme.key_normal,
                );
            }

            text_labels.push(TextLabel {
                text: cat.tab_icon().to_string(),
                cx: tx + tab_w * 0.5,
                cy: tab_bar_h * 0.5,
                font_size: 18.0 * dp,
                color_r: theme.text_primary.r,
                color_g: theme.text_primary.g,
                color_b: theme.text_primary.b,
                color_a: theme.text_primary.a,
                is_bold: false,
                label_type: 0,
            });
        }

        // Tab bar divider
        canvas.fill_rect(
            0.0,
            tab_bar_h - 1.0,
            metrics.total_width,
            1.0,
            theme.divider_color,
        );

        // 3. Emoji Grid (optimized with viewport culling for instant rendering of large categories)
        let emojis = self.active_category.emojis();
        let cols_count = self.get_cols(metrics);
        let cols = cols_count as f32;
        let cell_w = metrics.total_width / cols;
        let cell_h = 44.0 * dp;

        let em_min_y = content_y + 10.0 * dp;
        let em_max_y = bot_y - 10.0 * dp;

        let min_row = (self.scroll_offset_y / cell_h).floor().max(0.0) as usize;
        let visible_rows = (content_h / cell_h).ceil() as usize + 2;
        let max_row = min_row + visible_rows;

        let start_idx = min_row * cols_count;
        let end_idx = ((max_row + 1) * cols_count).min(emojis.len());

        if start_idx < emojis.len() {
            for idx in start_idx..end_idx {
                let em = emojis[idx];
                let row = (idx as f32 / cols).floor();
                let col = (idx as f32 % cols).floor();
                let cx = col * cell_w + cell_w * 0.5;
                let cy = content_y + row * cell_h + cell_h * 0.5 - self.scroll_offset_y;
                if cy >= em_min_y && cy <= em_max_y {
                    text_labels.push(TextLabel {
                        text: em.to_string(),
                        cx,
                        cy,
                        font_size: 24.0 * dp,
                        color_r: 255,
                        color_g: 255,
                        color_b: 255,
                        color_a: 255,
                        is_bold: false,
                        label_type: 6,
                    });
                }
            }
        }

        // 4. Bottom Navigation Bar with Search button
        canvas.fill_rect(
            0.0,
            bot_y,
            metrics.total_width,
            bottom_bar_h,
            theme.suggestion_bar_bg,
        );
        canvas.fill_rect(0.0, bot_y, metrics.total_width, 1.0, theme.divider_color);

        // Fill elevation zone below bottom bar to keep collapse arrow area clean
        if metrics.bottom_bar_height > 0.0 {
            canvas.fill_rect(
                0.0,
                bot_y + bottom_bar_h,
                metrics.total_width,
                metrics.bottom_bar_height,
                theme.bg_color,
            );
        }

        // ABC key (Return to Alphabet)
        let abc_w = 56.0 * dp;
        let abc_h = bottom_bar_h - 8.0 * dp;
        canvas.fill_rounded_rect(
            6.0 * dp,
            bot_y + 4.0 * dp,
            abc_w,
            abc_h,
            6.0 * dp,
            theme.key_modifier,
        );
        text_labels.push(TextLabel {
            text: "ABC".to_string(),
            cx: 6.0 * dp + abc_w * 0.5,
            cy: bot_y + bottom_bar_h * 0.5,
            font_size: 15.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: true,
            label_type: 0,
        });

        // Search key (magnifying glass)
        let search_w = 46.0 * dp;
        let search_x = 6.0 * dp + abc_w + 5.0 * dp;
        canvas.fill_rounded_rect(
            search_x,
            bot_y + 4.0 * dp,
            search_w,
            abc_h,
            6.0 * dp,
            theme.key_modifier,
        );
        text_labels.push(TextLabel {
            text: "🔍".to_string(),
            cx: search_x + search_w * 0.5,
            cy: bot_y + bottom_bar_h * 0.5,
            font_size: 16.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: false,
            label_type: 5,
        });

        // Backspace key
        let bs_w = 56.0 * dp;
        let bs_x = metrics.total_width - bs_w - 6.0 * dp;
        canvas.fill_rounded_rect(
            bs_x,
            bot_y + 4.0 * dp,
            bs_w,
            abc_h,
            6.0 * dp,
            theme.key_modifier,
        );
        text_labels.push(TextLabel {
            text: "⌫".to_string(),
            cx: bs_x + bs_w * 0.5,
            cy: bot_y + bottom_bar_h * 0.5,
            font_size: 16.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: false,
            label_type: 5,
        });

        // Space key
        let space_x = search_x + search_w + 5.0 * dp;
        let space_w = bs_x - space_x - 5.0 * dp;
        canvas.fill_rounded_rect(
            space_x,
            bot_y + 4.0 * dp,
            space_w,
            abc_h,
            6.0 * dp,
            theme.key_normal,
        );
        text_labels.push(TextLabel {
            text: "R Y N K".to_string(),
            cx: space_x + space_w * 0.5,
            cy: bot_y + bottom_bar_h * 0.5,
            font_size: 12.0 * dp,
            color_r: theme.space_branding_color.r,
            color_g: theme.space_branding_color.g,
            color_b: theme.space_branding_color.b,
            color_a: theme.space_branding_color.a,
            is_bold: false,
            label_type: 4,
        });
    }

    fn render_search(
        &self,
        canvas: &mut Canvas,
        metrics: &LayoutMetrics,
        theme: &RynkTheme,
        text_labels: &mut Vec<TextLabel>,
    ) {
        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let header_h = 40.0 * dp;
        let strip_h = 52.0 * dp;

        // 1. Background
        canvas.clear(theme.bg_color);

        // 2. Search Header
        // Back button (left)
        let back_btn_w = 42.0 * dp;
        canvas.fill_rounded_rect(
            4.0 * dp,
            4.0 * dp,
            back_btn_w - 8.0 * dp,
            header_h - 8.0 * dp,
            6.0 * dp,
            theme.key_modifier,
        );
        text_labels.push(TextLabel {
            text: "←".to_string(),
            cx: back_btn_w * 0.5,
            cy: header_h * 0.5,
            font_size: 18.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: true,
            label_type: 5,
        });

        // Search capsule (center/right)
        let capsule_x = back_btn_w + 2.0 * dp;
        let capsule_w = metrics.total_width - capsule_x - 6.0 * dp;
        let capsule_h = header_h - 8.0 * dp;
        let capsule_y = 4.0 * dp;
        canvas.fill_rounded_rect(
            capsule_x,
            capsule_y,
            capsule_w,
            capsule_h,
            capsule_h * 0.5,
            theme.key_normal,
        );

        // Magnifying glass icon inside capsule
        let search_icon_cx = capsule_x + 16.0 * dp;
        text_labels.push(TextLabel {
            text: "🔍".to_string(),
            cx: search_icon_cx,
            cy: capsule_y + capsule_h * 0.5,
            font_size: 15.0 * dp,
            color_r: theme.text_secondary.r,
            color_g: theme.text_secondary.g,
            color_b: theme.text_secondary.b,
            color_a: theme.text_secondary.a,
            is_bold: false,
            label_type: 5,
        });

        // Query text or placeholder
        if self.search_query.is_empty() {
            let placeholder = if self.search_language == Language::Russian {
                "Поиск эмодзи"
            } else {
                "Search emojis"
            };
            text_labels.push(TextLabel {
                text: placeholder.to_string(),
                cx: capsule_x + capsule_w * 0.45,
                cy: capsule_y + capsule_h * 0.5,
                font_size: 14.0 * dp,
                color_r: theme.text_secondary.r,
                color_g: theme.text_secondary.g,
                color_b: theme.text_secondary.b,
                color_a: theme.text_secondary.a,
                is_bold: false,
                label_type: 2,
            });
        } else {
            let display_text = format!("{}|", self.search_query);
            let clear_w = 28.0 * dp;
            let text_area_w = capsule_w - 32.0 * dp - clear_w;
            text_labels.push(TextLabel {
                text: display_text,
                cx: capsule_x + 30.0 * dp + text_area_w * 0.5,
                cy: capsule_y + capsule_h * 0.5,
                font_size: 15.0 * dp,
                color_r: theme.text_primary.r,
                color_g: theme.text_primary.g,
                color_b: theme.text_primary.b,
                color_a: theme.text_primary.a,
                is_bold: true,
                label_type: 0,
            });

            // Clear button (✕)
            let clear_btn_cx = capsule_x + capsule_w - 16.0 * dp;
            text_labels.push(TextLabel {
                text: "✕".to_string(),
                cx: clear_btn_cx,
                cy: capsule_y + capsule_h * 0.5,
                font_size: 14.0 * dp,
                color_r: theme.text_secondary.r,
                color_g: theme.text_secondary.g,
                color_b: theme.text_secondary.b,
                color_a: theme.text_secondary.a,
                is_bold: true,
                label_type: 5,
            });
        }

        // Divider below header
        canvas.fill_rect(0.0, header_h - 1.0, metrics.total_width, 1.0, theme.divider_color);

        // 3. Search Results Strip
        let strip_y = header_h;
        canvas.fill_rect(0.0, strip_y, metrics.total_width, strip_h, theme.suggestion_bar_bg);
        canvas.fill_rect(0.0, strip_y + strip_h - 1.0, metrics.total_width, 1.0, theme.divider_color);

        if self.search_results.is_empty() {
            let empty_msg = if self.search_language == Language::Russian {
                "Ничего не найдено"
            } else {
                "No emojis found"
            };
            text_labels.push(TextLabel {
                text: empty_msg.to_string(),
                cx: metrics.total_width * 0.5,
                cy: strip_y + strip_h * 0.5,
                font_size: 14.0 * dp,
                color_r: theme.text_secondary.r,
                color_g: theme.text_secondary.g,
                color_b: theme.text_secondary.b,
                color_a: theme.text_secondary.a,
                is_bold: false,
                label_type: 2,
            });
        } else {
            let cell_w = 46.0 * dp;
            let em_cy = strip_y + strip_h * 0.5;
            for (i, &em) in self.search_results.iter().enumerate() {
                let cx = 8.0 * dp + i as f32 * cell_w + cell_w * 0.5 - self.search_scroll_x;
                if cx >= -cell_w && cx <= metrics.total_width + cell_w {
                    text_labels.push(TextLabel {
                        text: em.to_string(),
                        cx,
                        cy: em_cy,
                        font_size: 24.0 * dp,
                        color_r: 255,
                        color_g: 255,
                        color_b: 255,
                        color_a: 255,
                        is_bold: false,
                        label_type: 6,
                    });
                }
            }
        }

        // 4. Alphabet Keys
        for key in &self.search_keys {
            let radius = 6.0 * dp;
            let is_pressed = self.pressed_key_id == Some(key.id);

            if !is_pressed {
                canvas.draw_drop_shadow(
                    key.x,
                    key.y,
                    key.width,
                    key.height,
                    radius,
                    2.0 * dp,
                    theme.key_shadow,
                );
            }

            let is_shift_active = matches!(key.action, KeyAction::Shift)
                && self.search_shift_state != ShiftState::Off;

            let fill_color = if is_pressed {
                theme.key_pressed
            } else if is_shift_active {
                theme.brand_accent
            } else {
                match key.key_type {
                    KeyType::Normal | KeyType::Space => theme.key_normal,
                    KeyType::Modifier | KeyType::Icon => theme.key_modifier,
                    KeyType::Accent => theme.key_accent,
                }
            };

            canvas.fill_rounded_rect(key.x, key.y, key.width, key.height, radius, fill_color);

            let font_size = match key.action {
                KeyAction::Character(_) => 19.0 * dp,
                KeyAction::Space => 12.0 * dp,
                KeyAction::Shift | KeyAction::Backspace | KeyAction::Enter => 17.0 * dp,
                _ => 14.0 * dp,
            };

            let (label_text, label_type) = match &key.action {
                KeyAction::Space => ("пробел".to_string(), 4),
                KeyAction::Shift => (
                    match self.search_shift_state {
                        ShiftState::Off => "⇧".to_string(),
                        ShiftState::Shifted => "⬆".to_string(),
                        ShiftState::CapsLock => "⇪".to_string(),
                    },
                    5,
                ),
                KeyAction::Backspace => ("⌫".to_string(), 5),
                KeyAction::Enter => ("↵".to_string(), 5),
                KeyAction::SwitchLanguage => ("🌐".to_string(), 5),
                _ => (key.label.clone(), 0),
            };

            let primary_color = if is_shift_active || matches!(key.key_type, KeyType::Accent) {
                theme.key_accent_text
            } else {
                theme.text_primary
            };

            text_labels.push(TextLabel {
                text: label_text,
                cx: key.center().0,
                cy: key.center().1,
                font_size,
                color_r: primary_color.r,
                color_g: primary_color.g,
                color_b: primary_color.b,
                color_a: primary_color.a,
                is_bold: matches!(key.action, KeyAction::Character(_)),
                label_type,
            });
        }

        // 5. Fill elevation zone at bottom
        if metrics.bottom_bar_height > 0.0 {
            canvas.fill_rect(
                0.0,
                metrics.total_height - metrics.bottom_bar_height,
                metrics.total_width,
                metrics.bottom_bar_height,
                theme.bg_color,
            );
        }
    }

    fn handle_search_touch(
        &mut self,
        action: TouchAction,
        x: f32,
        y: f32,
        metrics: &LayoutMetrics,
    ) -> EmojiTouchResult {
        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let header_h = 40.0 * dp;
        let strip_h = 52.0 * dp;
        let back_btn_w = 42.0 * dp;

        let cell_w = 46.0 * dp;
        let total_results_w = self.search_results.len() as f32 * cell_w + 16.0 * dp;
        let max_scroll_x = (total_results_w - metrics.total_width).max(0.0);

        match action {
            TouchAction::Down => {
                self.touch_start_x = x;
                self.touch_start_y = y;
                self.last_touch_x = Some(x);
                self.last_touch_y = Some(y);
                self.is_dragging = false;

                if y >= (header_h + strip_h) {
                    if let Some(key) = self.find_search_key(x, y) {
                        self.pressed_key_id = Some(key.id);
                    }
                }
                EmojiTouchResult::None
            }
            TouchAction::Move => {
                if let Some(last_x) = self.last_touch_x {
                    let dx = last_x - x;
                    if (x - self.touch_start_x).abs() > 6.0 * dp {
                        self.is_dragging = true;
                    }
                    if self.is_dragging && y >= header_h && y < (header_h + strip_h) {
                        self.search_scroll_x =
                            (self.search_scroll_x + dx).clamp(0.0, max_scroll_x);
                    }
                }
                self.last_touch_x = Some(x);
                self.last_touch_y = Some(y);

                if y >= (header_h + strip_h) {
                    if let Some(key) = self.find_search_key(x, y) {
                        self.pressed_key_id = Some(key.id);
                    } else {
                        self.pressed_key_id = None;
                    }
                }
                EmojiTouchResult::None
            }
            TouchAction::Up => {
                self.last_touch_x = None;
                self.last_touch_y = None;
                self.pressed_key_id = None;

                // 1. Header tap
                if y < header_h {
                    if x < back_btn_w {
                        self.close_search();
                        return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                    }

                    // Check clear button tap (right side of search capsule)
                    let capsule_x = back_btn_w + 2.0 * dp;
                    let capsule_w = metrics.total_width - capsule_x - 6.0 * dp;
                    let clear_btn_x = capsule_x + capsule_w - 32.0 * dp;
                    if !self.search_query.is_empty() && x >= clear_btn_x {
                        self.search_query.clear();
                        self.search_results = search::DEFAULT_EMOJIS.to_vec();
                        self.search_scroll_x = 0.0;
                        return EmojiTouchResult::Haptic(HapticFeedbackType::KeyTick);
                    }
                    return EmojiTouchResult::None;
                }

                // 2. Results strip tap (only if not dragged)
                if y >= header_h && y < (header_h + strip_h) {
                    if !self.is_dragging {
                        let rel_x = x - 8.0 * dp + self.search_scroll_x;
                        if rel_x >= 0.0 {
                            let idx = (rel_x / cell_w) as usize;
                            if let Some(&em) = self.search_results.get(idx) {
                                return EmojiTouchResult::SelectEmoji(em);
                            }
                        }
                    }
                    self.is_dragging = false;
                    return EmojiTouchResult::None;
                }

                // 3. Alphabet keys tap
                if y >= (header_h + strip_h) {
                    if let Some(key) = self.find_search_key(x, y) {
                        match &key.action {
                            KeyAction::Character(c) => {
                                self.search_query.push(*c);
                                self.search_results =
                                    search::search_emojis(&self.search_query);
                                self.search_scroll_x = 0.0;
                                if self.search_shift_state == ShiftState::Shifted {
                                    self.search_shift_state = ShiftState::Off;
                                    self.rebuild_search_keys(metrics);
                                }
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyTick);
                            }
                            KeyAction::Backspace => {
                                self.search_query.pop();
                                if self.search_query.is_empty() {
                                    self.search_results = search::DEFAULT_EMOJIS.to_vec();
                                } else {
                                    self.search_results =
                                        search::search_emojis(&self.search_query);
                                }
                                self.search_scroll_x = 0.0;
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                            }
                            KeyAction::Space => {
                                self.search_query.push(' ');
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyTick);
                            }
                            KeyAction::Shift => {
                                self.search_shift_state = match self.search_shift_state {
                                    ShiftState::Off => ShiftState::Shifted,
                                    _ => ShiftState::Off,
                                };
                                self.rebuild_search_keys(metrics);
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                            }
                            KeyAction::SwitchLanguage => {
                                self.search_language =
                                    if self.search_language == Language::Russian {
                                        Language::English
                                    } else {
                                        Language::Russian
                                    };
                                self.rebuild_search_keys(metrics);
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                            }
                            KeyAction::Enter | KeyAction::SwitchMode(_) => {
                                self.close_search();
                                return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                            }
                            _ => {}
                        }
                    }
                }

                self.is_dragging = false;
                EmojiTouchResult::None
            }
            TouchAction::Cancel => {
                self.last_touch_x = None;
                self.last_touch_y = None;
                self.pressed_key_id = None;
                self.is_dragging = false;
                EmojiTouchResult::None
            }
        }
    }

    pub fn handle_touch_event(
        &mut self,
        action: TouchAction,
        x: f32,
        y: f32,
        metrics: &LayoutMetrics,
        current_language: Language,
    ) -> EmojiTouchResult {
        if self.is_search_active {
            if self.search_keys.is_empty() {
                self.rebuild_search_keys(metrics);
            }
            return self.handle_search_touch(action, x, y, metrics);
        }

        let dp = (metrics.suggestion_bar_height / 40.0).max(1.0);
        let tab_bar_h = 40.0 * dp;
        let bottom_bar_h = 40.0 * dp;
        let bot_y = metrics.total_height - metrics.bottom_bar_height - bottom_bar_h;
        let back_btn_w = 46.0 * dp;

        let content_y = tab_bar_h;
        let content_h = (bot_y - tab_bar_h).max(10.0);
        let cell_h = 42.0 * dp;
        let cols_count = self.get_cols(metrics);
        let total_rows = (self.active_category.emojis().len() as f32 / cols_count as f32).ceil();
        let total_grid_h = total_rows * cell_h;
        let max_scroll = (total_grid_h - content_h + 16.0 * dp).max(0.0);

        match action {
            TouchAction::Down => {
                self.touch_start_x = x;
                self.touch_start_y = y;
                self.last_touch_y = Some(y);
                self.is_dragging = false;
                EmojiTouchResult::None
            }
            TouchAction::Move => {
                if let Some(last_y) = self.last_touch_y {
                    let dy = last_y - y;
                    if (y - self.touch_start_y).abs() > 6.0 * dp {
                        self.is_dragging = true;
                    }
                    if self.is_dragging {
                        self.scroll_offset_y = (self.scroll_offset_y + dy).clamp(0.0, max_scroll);
                    }
                }
                self.last_touch_y = Some(y);
                EmojiTouchResult::None
            }
            TouchAction::Up => {
                self.last_touch_y = None;

                // 1. Top Bar click
                if y < tab_bar_h {
                    if x < back_btn_w {
                        return EmojiTouchResult::SwitchToAlphabet;
                    }

                    let categories = [
                        EmojiCategory::Smileys,
                        EmojiCategory::Gestures,
                        EmojiCategory::Nature,
                        EmojiCategory::Food,
                        EmojiCategory::Activities,
                        EmojiCategory::Travel,
                        EmojiCategory::Objects,
                        EmojiCategory::Symbols,
                        EmojiCategory::Flags,
                    ];
                    let remaining_w = metrics.total_width - back_btn_w;
                    let tab_w = remaining_w / categories.len() as f32;
                    let rel_x = x - back_btn_w;
                    if rel_x >= 0.0 {
                        let idx = (rel_x / tab_w) as usize;
                        if let Some(&cat) = categories.get(idx) {
                            self.select_category(cat);
                        }
                    }
                    return EmojiTouchResult::None;
                }

                // 2. Bottom bar click (or elevation padding click)
                if y >= bot_y && y < (bot_y + bottom_bar_h) {
                    let abc_w = 56.0 * dp;
                    let search_w = 46.0 * dp;
                    let search_x = 6.0 * dp + abc_w + 5.0 * dp;
                    let bs_w = 56.0 * dp;
                    let bs_x = metrics.total_width - bs_w - 6.0 * dp;

                    if x <= (6.0 * dp + abc_w + 2.0 * dp) {
                        return EmojiTouchResult::SwitchToAlphabet;
                    } else if x >= (search_x - 2.0 * dp) && x <= (search_x + search_w + 2.0 * dp) {
                        self.open_search(metrics, current_language);
                        return EmojiTouchResult::Haptic(HapticFeedbackType::KeyClick);
                    } else if x >= (bs_x - 2.0 * dp) {
                        return EmojiTouchResult::Backspace;
                    } else {
                        return EmojiTouchResult::Space;
                    }
                }
                if y >= (bot_y + bottom_bar_h) {
                    return EmojiTouchResult::None;
                }

                // 3. Grid cell click (only if not dragged)
                if !self.is_dragging {
                    let cols_count = self.get_cols(metrics);
                    let cols = cols_count as f32;
                    let cell_w = metrics.total_width / cols;
                    let rel_y = y - content_y + self.scroll_offset_y;

                    if rel_y >= 0.0 {
                        let row = (rel_y / cell_h) as usize;
                        let col = (x / cell_w) as usize;
                        if col < cols_count {
                            let idx = row * cols_count + col;
                            let emojis = self.active_category.emojis();
                            if let Some(&em) = emojis.get(idx) {
                                return EmojiTouchResult::SelectEmoji(em);
                            }
                        }
                    }
                }

                self.is_dragging = false;
                EmojiTouchResult::None
            }
            TouchAction::Cancel => {
                self.last_touch_y = None;
                self.is_dragging = false;
                EmojiTouchResult::None
            }
        }
    }
}
