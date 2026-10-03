use super::canvas::Canvas;
use super::theme::RynkTheme;
use super::TextLabel;
use crate::keyboard::key::Key;

pub struct PopupRenderer;

impl PopupRenderer {
    /// Renders a magnified key preview popup above the pressed key
    pub fn draw_preview(
        canvas: &mut Canvas,
        key: &Key,
        theme: &RynkTheme,
        density: f32,
        text_labels: &mut Vec<TextLabel>,
    ) {
        let dp = density.max(1.0);
        let popup_w = key.width * 1.35;
        let popup_h = key.height * 1.15;
        let popup_x = key.center().0 - popup_w * 0.5;
        let popup_y = (key.y - popup_h - 4.0 * dp).max(2.0 * dp);
        let radius = 10.0 * dp;

        // Soft elevation shadow
        canvas.draw_drop_shadow(popup_x, popup_y, popup_w, popup_h, radius, 4.0 * dp, theme.key_shadow);

        // Body fill with subtle border
        canvas.fill_rounded_rect(popup_x, popup_y, popup_w, popup_h, radius, theme.popup_border);

        let border_stroke = 1.0 * dp;
        canvas.fill_rounded_rect(
            popup_x + border_stroke,
            popup_y + border_stroke,
            popup_w - border_stroke * 2.0,
            popup_h - border_stroke * 2.0,
            radius - border_stroke,
            theme.popup_bg,
        );

        // Connecting neck towards the key
        let neck_w = key.width * 0.7;
        let neck_h = 6.0 * dp;
        let neck_x = key.center().0 - neck_w * 0.5;
        let neck_y = popup_y + popup_h - 2.0 * dp;
        canvas.fill_rounded_rect(neck_x, neck_y, neck_w, neck_h, 3.0 * dp, theme.popup_bg);

        // Large magnified character label
        let font_size = 26.0 * dp;
        text_labels.push(TextLabel {
            text: key.label.clone(),
            cx: popup_x + popup_w * 0.5,
            cy: popup_y + popup_h * 0.48,
            font_size,
            color_r: theme.popup_text.r,
            color_g: theme.popup_text.g,
            color_b: theme.popup_text.b,
            color_a: theme.popup_text.a,
            is_bold: true,
            label_type: 3,
        });
    }

    /// Renders the long-press alternate character strip above the key
    pub fn draw_alternate_menu(
        canvas: &mut Canvas,
        key: &Key,
        alternates: &[char],
        selected_index: Option<usize>,
        theme: &RynkTheme,
        density: f32,
        text_labels: &mut Vec<TextLabel>,
    ) {
        if alternates.is_empty() {
            return;
        }

        let dp = density.max(1.0);
        let item_w = (key.width * 1.1).max(38.0 * dp);
        let item_h = key.height * 1.15;
        let total_w = alternates.len() as f32 * item_w;
        let total_h = item_h;

        let mut start_x = key.center().0 - total_w * 0.5;
        // Clamp within screen boundaries
        if start_x < 6.0 * dp {
            start_x = 6.0 * dp;
        } else if (start_x + total_w) > (canvas.width as f32 - 6.0 * dp) {
            start_x = (canvas.width as f32 - 6.0 * dp) - total_w;
        }

        let start_y = (key.y - total_h - 6.0 * dp).max(2.0 * dp);
        let radius = 10.0 * dp;

        // Shadow
        canvas.draw_drop_shadow(start_x, start_y, total_w, total_h, radius, 6.0 * dp, theme.key_shadow);

        // Container
        canvas.fill_rounded_rect(start_x, start_y, total_w, total_h, radius, theme.popup_bg);

        // Items
        for (i, &alt_ch) in alternates.iter().enumerate() {
            let item_x = start_x + (i as f32 * item_w);
            let is_selected = selected_index == Some(i);

            if is_selected {
                let pill_pad = 4.0 * dp;
                canvas.fill_rounded_rect(
                    item_x + pill_pad,
                    start_y + pill_pad,
                    item_w - pill_pad * 2.0,
                    item_h - pill_pad * 2.0,
                    radius * 0.7,
                    theme.brand_accent,
                );
            }

            let text_color = if is_selected {
                theme.key_accent_text
            } else {
                theme.popup_text
            };

            let font_size = 22.0 * dp;
            text_labels.push(TextLabel {
                text: alt_ch.to_string(),
                cx: item_x + item_w * 0.5,
                cy: start_y + item_h * 0.5,
                font_size,
                color_r: text_color.r,
                color_g: text_color.g,
                color_b: text_color.b,
                color_a: text_color.a,
                is_bold: is_selected,
                label_type: 3,
            });
        }
    }
}
