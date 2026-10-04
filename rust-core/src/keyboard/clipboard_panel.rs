//! Clipboard history panel: recent copies as cards in two scrollable columns.
//!
//! The history itself is stored by the app (Kotlin) and pushed in with `set_items`; actions on
//! it come back as `ClipboardPanelResult` and are turned into events by the caller.

use super::layout::LayoutMetrics;
use super::touch::TouchAction;
use crate::render::canvas::Canvas;
use crate::render::theme::RynkTheme;
use crate::render::TextLabel;

#[derive(Clone, Debug, PartialEq)]
pub struct ClipItem {
    pub text: String,
    pub pinned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClipboardPanelResult {
    None,
    Paste(String),
    TogglePin(usize),
    Delete(usize),
    ClearUnpinned,
    Close,
}

const CARD_H_DP: f32 = 58.0;
const GAP_DP: f32 = 6.0;
const CORNER_DP: f32 = 26.0;
const LONG_PRESS_MS: u64 = 450;
const SLOP_DP: f32 = 8.0;

#[derive(Default)]
pub struct ClipboardPanel {
    pub items: Vec<ClipItem>,
    scroll: f32,
    down: Option<(f32, f32, u64, f32)>,
    scrolling: bool,
}

struct Geometry {
    dp: f32,
    bar_h: f32,
    content_top: f32,
    content_bottom: f32,
    card_w: f32,
    card_h: f32,
    gap: f32,
}

impl ClipboardPanel {
    pub fn set_items(&mut self, items: Vec<ClipItem>) {
        self.items = items;
        self.scroll = 0.0;
    }

    fn geometry(m: &LayoutMetrics) -> Geometry {
        let dp = (m.suggestion_bar_height / 40.0).max(1.0);
        let gap = GAP_DP * dp;
        Geometry {
            dp,
            bar_h: m.suggestion_bar_height,
            content_top: m.suggestion_bar_height + gap,
            content_bottom: m.total_height - m.bottom_bar_height,
            card_w: (m.total_width - 3.0 * gap) / 2.0,
            card_h: CARD_H_DP * dp,
            gap,
        }
    }

    /// Card rectangle (x, y, w, h) on screen for item `i`
    fn card_rect(g: &Geometry, i: usize, scroll: f32) -> (f32, f32, f32, f32) {
        let col = i % 2;
        let row = i / 2;
        let x = g.gap + col as f32 * (g.card_w + g.gap);
        let y = g.content_top + row as f32 * (g.card_h + g.gap) - scroll;
        (x, y, g.card_w, g.card_h)
    }

    fn max_scroll(&self, g: &Geometry) -> f32 {
        let rows = self.items.len().div_ceil(2) as f32;
        let content = rows * (g.card_h + g.gap);
        (content - (g.content_bottom - g.content_top)).max(0.0)
    }

    pub fn render(&self, canvas: &mut Canvas, m: &LayoutMetrics, theme: &RynkTheme, labels: &mut Vec<TextLabel>) {
        let g = Self::geometry(m);
        canvas.clear(theme.bg_color);
        let label = |text: &str, cx: f32, cy: f32, size: f32, color: crate::render::canvas::Color, icon: bool| TextLabel {
            text: text.to_string(),
            cx,
            cy,
            font_size: size,
            color_r: color.r,
            color_g: color.g,
            color_b: color.b,
            color_a: color.a,
            is_bold: false,
            label_type: if icon { 5 } else { 2 },
        };

        // Cards (clipped to the content area by skipping those outside it)
        for (i, item) in self.items.iter().enumerate() {
            let (x, y, w, h) = Self::card_rect(&g, i, self.scroll);
            if y + h < g.content_top || y > g.content_bottom {
                continue;
            }
            let bg = if item.pinned { theme.key_modifier } else { theme.key_normal };
            canvas.fill_rounded_rect(x, y, w, h, 8.0 * g.dp, bg);
            let font = 13.0 * g.dp;
            labels.push(label(&preview(&item.text, w - 2.0 * CORNER_DP * g.dp, font), x + w / 2.0, y + h / 2.0, font, theme.text_primary, false));
            let corner = CORNER_DP * g.dp;
            labels.push(label("cb_delete", x + w - corner / 2.0, y + corner / 2.0, 11.0 * g.dp, theme.text_secondary, true));
            if item.pinned {
                labels.push(label("cb_pin", x + corner / 2.0, y + corner / 2.0, 11.0 * g.dp, theme.brand_accent, true));
            }
        }
        if self.items.is_empty() {
            labels.push(label("tb_clipboard", m.total_width / 2.0, (g.content_top + g.content_bottom) / 2.0, 34.0 * g.dp, theme.text_secondary, true));
        }

        // Header drawn last so scrolled cards slide under it
        canvas.fill_rect(0.0, 0.0, m.total_width, g.bar_h, theme.suggestion_bar_bg);
        canvas.fill_rect(0.0, g.bar_h - 1.0, m.total_width, 1.0, theme.divider_color);
        labels.push(label("ABC", 30.0 * g.dp, g.bar_h / 2.0, 15.0 * g.dp, theme.suggestion_text, false));
        labels.push(label("tb_clipboard", m.total_width / 2.0, g.bar_h / 2.0, 16.0 * g.dp, theme.text_secondary, true));
        if self.items.iter().any(|i| !i.pinned) {
            labels.push(label("cb_clear", m.total_width - 30.0 * g.dp, g.bar_h / 2.0, 16.0 * g.dp, theme.text_secondary, true));
        }
    }

    pub fn handle_touch(&mut self, action: TouchAction, x: f32, y: f32, time_ms: u64, m: &LayoutMetrics) -> ClipboardPanelResult {
        let g = Self::geometry(m);
        match action {
            TouchAction::Down => {
                self.down = Some((x, y, time_ms, self.scroll));
                self.scrolling = false;
                ClipboardPanelResult::None
            }
            TouchAction::Move => {
                if let Some((_, sy, _, start_scroll)) = self.down {
                    if self.scrolling || (y - sy).abs() > SLOP_DP * g.dp {
                        self.scrolling = true;
                        self.scroll = (start_scroll - (y - sy)).clamp(0.0, self.max_scroll(&g));
                    }
                }
                ClipboardPanelResult::None
            }
            TouchAction::Cancel => {
                self.down = None;
                ClipboardPanelResult::None
            }
            TouchAction::Up => {
                let Some((dx, dy, t0, _)) = self.down.take() else { return ClipboardPanelResult::None };
                if self.scrolling {
                    return ClipboardPanelResult::None;
                }
                if dy < g.bar_h {
                    if dx < 70.0 * g.dp {
                        return ClipboardPanelResult::Close;
                    }
                    if dx > m.total_width - 70.0 * g.dp && self.items.iter().any(|i| !i.pinned) {
                        return ClipboardPanelResult::ClearUnpinned;
                    }
                    return ClipboardPanelResult::None;
                }
                let corner = CORNER_DP * g.dp;
                for (i, item) in self.items.iter().enumerate() {
                    let (cx, cy, w, h) = Self::card_rect(&g, i, self.scroll);
                    if dx >= cx && dx <= cx + w && dy >= cy && dy <= cy + h {
                        if dx >= cx + w - corner && dy <= cy + corner {
                            return ClipboardPanelResult::Delete(i);
                        }
                        if time_ms.saturating_sub(t0) >= LONG_PRESS_MS {
                            return ClipboardPanelResult::TogglePin(i);
                        }
                        return ClipboardPanelResult::Paste(item.text.clone());
                    }
                }
                ClipboardPanelResult::None
            }
        }
    }
}

/// Single-line preview that roughly fits `width` at `font` size
fn preview(text: &str, width: f32, font: f32) -> String {
    let max_chars = ((width / (font * 0.55)) as usize).max(4);
    let flat: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= max_chars {
        flat
    } else {
        let cut: String = flat.chars().take(max_chars - 1).collect();
        format!("{}…", cut.trim_end())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panel() -> (ClipboardPanel, LayoutMetrics) {
        let mut p = ClipboardPanel::default();
        p.set_items(
            (0..9)
                .map(|i| ClipItem { text: format!("copied text {i}\nsecond line"), pinned: i == 0 })
                .collect(),
        );
        (p, LayoutMetrics::new(1080.0, 800.0, 2.75))
    }

    fn tap(p: &mut ClipboardPanel, m: &LayoutMetrics, x: f32, y: f32, hold: u64) -> ClipboardPanelResult {
        p.handle_touch(TouchAction::Down, x, y, 1000, m);
        p.handle_touch(TouchAction::Up, x, y, 1000 + hold, m)
    }

    #[test]
    fn tap_pastes_long_press_pins_corner_deletes() {
        let (mut p, m) = panel();
        let g = ClipboardPanel::geometry(&m);
        let (x, y, w, h) = ClipboardPanel::card_rect(&g, 1, 0.0);
        assert_eq!(tap(&mut p, &m, x + w / 2.0, y + h / 2.0, 50), ClipboardPanelResult::Paste("copied text 1\nsecond line".into()));
        assert_eq!(tap(&mut p, &m, x + w / 2.0, y + h / 2.0, 600), ClipboardPanelResult::TogglePin(1));
        assert_eq!(tap(&mut p, &m, x + w - 5.0, y + 5.0, 50), ClipboardPanelResult::Delete(1));
        assert_eq!(tap(&mut p, &m, 20.0, 20.0, 50), ClipboardPanelResult::Close);
        assert_eq!(tap(&mut p, &m, 1070.0, 20.0, 50), ClipboardPanelResult::ClearUnpinned);
    }

    #[test]
    fn dragging_scrolls_instead_of_pasting() {
        let (mut p, m) = panel();
        p.handle_touch(TouchAction::Down, 300.0, 600.0, 0, &m);
        p.handle_touch(TouchAction::Move, 300.0, 300.0, 50, &m);
        assert_eq!(p.handle_touch(TouchAction::Up, 300.0, 300.0, 100, &m), ClipboardPanelResult::None);
        assert!(p.scroll > 0.0);
    }

    #[test]
    fn previews_are_single_line_and_bounded() {
        assert_eq!(preview("a\nb   c", 500.0, 10.0), "a b c");
        let long = preview(&"x".repeat(200), 100.0, 10.0);
        assert!(long.ends_with('…') && long.chars().count() <= 18);
    }
}
