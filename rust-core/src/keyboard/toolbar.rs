//! The suggestion bar when there is nothing to suggest: tool buttons and punctuation shortcuts.
//! Shared by rendering and touch handling so both agree on the geometry.

/// Icon names are drawn by `SvgIcons` on the Kotlin side
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolbarItem {
    Settings,
    Clipboard,
    Edit,
    OneHanded,
    Voice,
    Shortcut(&'static str),
}

impl ToolbarItem {
    pub fn label(&self) -> &'static str {
        match self {
            ToolbarItem::Settings => "⚙",
            ToolbarItem::Clipboard => "tb_clipboard",
            ToolbarItem::Edit => "tb_edit",
            ToolbarItem::OneHanded => "tb_one_handed",
            ToolbarItem::Voice => "tb_mic",
            ToolbarItem::Shortcut(s) => s,
        }
    }

    pub fn is_icon(&self) -> bool {
        !matches!(self, ToolbarItem::Shortcut(_))
    }
}

const ICON_W_DP: f32 = 42.0;
const SHORTCUTS: [&str; 4] = [",", ".", "?", "!"];

/// Items with their horizontal extent `(item, x, width)`, left to right.
pub fn layout(total_width: f32, dp: f32, voice: bool) -> Vec<(ToolbarItem, f32, f32)> {
    let icon_w = ICON_W_DP * dp;
    let left = [ToolbarItem::Settings, ToolbarItem::OneHanded];
    let mut items = Vec::with_capacity(10);
    let mut x = 0.0;
    for item in left {
        items.push((item, x, icon_w));
        x += icon_w;
    }
    let right_w = if voice { icon_w } else { 0.0 };
    let shortcut_w = ((total_width - x - right_w) / SHORTCUTS.len() as f32).max(0.0);
    for s in SHORTCUTS {
        items.push((ToolbarItem::Shortcut(s), x, shortcut_w));
        x += shortcut_w;
    }
    if voice {
        items.push((ToolbarItem::Voice, total_width - icon_w, icon_w));
    }
    items
}

pub fn hit(total_width: f32, dp: f32, voice: bool, x: f32) -> Option<ToolbarItem> {
    layout(total_width, dp, voice)
        .into_iter()
        .find(|&(_, ix, w)| x >= ix && x < ix + w)
        .map(|(item, _, _)| item)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_tile_the_bar() {
        let items = layout(1080.0, 2.75, true);
        let end = items.iter().map(|&(_, x, w)| x + w).fold(0.0f32, f32::max);
        assert!((end - 1080.0).abs() < 0.5);
        assert_eq!(hit(1080.0, 2.75, true, 5.0), Some(ToolbarItem::Settings));
        assert_eq!(hit(1080.0, 2.75, true, 1075.0), Some(ToolbarItem::Voice));
        assert!(matches!(hit(1080.0, 2.75, false, 1075.0), Some(ToolbarItem::Shortcut(_))));
    }
}
