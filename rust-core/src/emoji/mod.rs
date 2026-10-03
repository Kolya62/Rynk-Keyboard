pub mod data;

use data::EmojiCategory;
use crate::render::canvas::Canvas;
use crate::render::theme::RynkTheme;
use crate::render::TextLabel;
use crate::keyboard::layout::LayoutMetrics;
use crate::keyboard::touch::TouchAction;

pub struct EmojiManager {
    pub active_category: EmojiCategory,
    pub scroll_offset_y: f32,
    pub touch_start_x: f32,
    pub touch_start_y: f32,
    pub last_touch_y: Option<f32>,
    pub is_dragging: bool,
}

impl Default for EmojiManager {
    fn default() -> Self {
        Self {
            active_category: EmojiCategory::Smileys,
            scroll_offset_y: 0.0,
            touch_start_x: 0.0,
            touch_start_y: 0.0,
            last_touch_y: None,
            is_dragging: false,
        }
    }
}

pub enum EmojiTouchResult {
    None,
    SelectEmoji(&'static str),
    SwitchToAlphabet,
    Backspace,
    Space,
}

impl EmojiManager {
    pub fn select_category(&mut self, cat: EmojiCategory) {
        self.active_category = cat;
        self.scroll_offset_y = 0.0;
        self.last_touch_y = None;
        self.is_dragging = false;
    }

    pub fn render(
        &self,
        canvas: &mut Canvas,
        metrics: &LayoutMetrics,
        theme: &RynkTheme,
        text_labels: &mut Vec<TextLabel>,
    ) {
        let dp = (metrics.suggestion_bar_height / 44.0).max(1.0);
        let tab_bar_h = 42.0 * dp;
        let bottom_bar_h = 44.0 * dp;
        let bot_y = metrics.total_height - metrics.bottom_bar_height - bottom_bar_h;
        let content_y = tab_bar_h;
        let _content_h = (bot_y - tab_bar_h).max(10.0);

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
        canvas.fill_rect(0.0, tab_bar_h - 1.0, metrics.total_width, 1.0, theme.divider_color);

        // 3. Emoji Grid
        let emojis = self.active_category.emojis();
        let cols = 7.0;
        let cell_w = metrics.total_width / cols;
        let cell_h = 44.0 * dp;

        for (idx, &em) in emojis.iter().enumerate() {
            let row = (idx as f32 / cols).floor();
            let col = (idx as f32 % cols).floor();
            let cx = col * cell_w + cell_w * 0.5;
            let cy = content_y + row * cell_h + cell_h * 0.5 - self.scroll_offset_y;
            let em_min_y = content_y + 10.0 * dp;
            let em_max_y = bot_y - 10.0 * dp;
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

        // 4. Bottom Navigation Bar (comfortably elevated above system navigation bar)
        canvas.fill_rect(0.0, bot_y, metrics.total_width, bottom_bar_h, theme.suggestion_bar_bg);
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
        let abc_w = 76.0 * dp;
        let abc_h = bottom_bar_h - 8.0 * dp;
        canvas.fill_rounded_rect(8.0 * dp, bot_y + 4.0 * dp, abc_w, abc_h, 6.0 * dp, theme.key_modifier);
        text_labels.push(TextLabel {
            text: "ABC".to_string(),
            cx: 8.0 * dp + abc_w * 0.5,
            cy: bot_y + bottom_bar_h * 0.5,
            font_size: 15.0 * dp,
            color_r: theme.text_primary.r,
            color_g: theme.text_primary.g,
            color_b: theme.text_primary.b,
            color_a: theme.text_primary.a,
            is_bold: true,
            label_type: 0,
        });

        // Backspace key
        let bs_w = 64.0 * dp;
        let bs_x = metrics.total_width - bs_w - 6.0 * dp;
        canvas.fill_rounded_rect(bs_x, bot_y + 4.0 * dp, bs_w, abc_h, 6.0 * dp, theme.key_modifier);
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
        let space_x = 8.0 * dp + abc_w + 6.0 * dp;
        let space_w = bs_x - space_x - 6.0 * dp;
        canvas.fill_rounded_rect(space_x, bot_y + 4.0 * dp, space_w, abc_h, 6.0 * dp, theme.key_normal);
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

    pub fn handle_touch_event(
        &mut self,
        action: TouchAction,
        x: f32,
        y: f32,
        metrics: &LayoutMetrics,
    ) -> EmojiTouchResult {
        let dp = (metrics.suggestion_bar_height / 44.0).max(1.0);
        let tab_bar_h = 42.0 * dp;
        let bottom_bar_h = 44.0 * dp;
        let bot_y = metrics.total_height - metrics.bottom_bar_height - bottom_bar_h;
        let back_btn_w = 46.0 * dp;

        let content_y = tab_bar_h;
        let content_h = (bot_y - tab_bar_h).max(10.0);
        let cell_h = 44.0 * dp;
        let total_rows = (self.active_category.emojis().len() as f32 / 7.0).ceil();
        let total_grid_h = total_rows * cell_h;
        let max_scroll = (total_grid_h - content_h).max(0.0);

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
                    let abc_w = 76.0 * dp;
                    let bs_w = 64.0 * dp;
                    let bs_x = metrics.total_width - bs_w - 6.0 * dp;

                    if x <= (abc_w + 14.0 * dp) {
                        return EmojiTouchResult::SwitchToAlphabet;
                    } else if x >= bs_x {
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
                    let cols = 7.0;
                    let cell_w = metrics.total_width / cols;
                    let rel_y = y - content_y + self.scroll_offset_y;

                    if rel_y >= 0.0 {
                        let row = (rel_y / cell_h) as usize;
                        let col = (x / cell_w) as usize;
                        let idx = row * 7 + col;
                        let emojis = self.active_category.emojis();
                        if let Some(&em) = emojis.get(idx) {
                            return EmojiTouchResult::SelectEmoji(em);
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

    pub fn handle_touch(
        &mut self,
        x: f32,
        y: f32,
        metrics: &LayoutMetrics,
    ) -> EmojiTouchResult {
        self.handle_touch_event(TouchAction::Up, x, y, metrics)
    }
}
