use super::key::{Key, KeyAction, KeyType, KeyboardMode};
use super::state::{Language, ShiftState};

pub type KeyDef<'a> = (&'a str, Option<&'a str>, &'a [char]);

pub struct LayoutMetrics {
    pub total_width: f32,
    pub total_height: f32,
    pub suggestion_bar_height: f32,
    pub key_area_top: f32,
    pub key_area_height: f32,
    pub padding_horizontal: f32,
    pub padding_bottom: f32,
    pub key_spacing_h: f32,
    pub key_spacing_v: f32,
    pub key_border_radius: f32,
    pub bottom_bar_height: f32,
}

impl LayoutMetrics {
    pub fn new(width: f32, height: f32, density: f32) -> Self {
        let dp = density.max(1.0);
        let suggestion_bar_height = 40.0 * dp;
        let padding_horizontal = 2.0 * dp;
        let bottom_bar_height = 36.0 * dp; // Elevation zone for collapse arrow and nav bar
        let padding_bottom = bottom_bar_height + 2.0 * dp;
        let key_spacing_h = 3.0 * dp;
        let key_spacing_v = 4.0 * dp;
        let key_border_radius = 5.0 * dp;

        let key_area_top = suggestion_bar_height + 2.0 * dp;
        let key_area_height = (height - key_area_top - padding_bottom).max(10.0);

        Self {
            total_width: width,
            total_height: height,
            suggestion_bar_height,
            key_area_top,
            key_area_height,
            padding_horizontal,
            padding_bottom,
            key_spacing_h,
            key_spacing_v,
            key_border_radius,
            bottom_bar_height,
        }
    }
}

/// Which side the keys hug in one-handed mode
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OneHanded {
    #[default]
    Off,
    Left,
    Right,
}

impl OneHanded {
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => OneHanded::Left,
            2 => OneHanded::Right,
            _ => OneHanded::Off,
        }
    }

    pub fn to_id(self) -> i32 {
        match self {
            OneHanded::Off => 0,
            OneHanded::Left => 1,
            OneHanded::Right => 2,
        }
    }
}

/// Settings that reshape any language's layout
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LayoutOptions {
    /// A row of digits above the letters
    pub number_row: bool,
    pub one_handed: OneHanded,
    /// Edit panel: the select key is lit while arrows extend the selection
    pub edit_selecting: bool,
}

/// Share of the width the keys keep in one-handed mode
const ONE_HANDED_WIDTH: f32 = 0.82;

pub struct LayoutBuilder;

impl LayoutBuilder {
    pub fn build_layout(
        mode: KeyboardMode,
        language: Language,
        shift_state: ShiftState,
        metrics: &LayoutMetrics,
    ) -> Vec<Key> {
        Self::build_layout_with(mode, language, shift_state, metrics, &LayoutOptions::default())
    }

    /// Builds a layout, then applies the number row and one-handed mode on top of it, so every
    /// language gets them without per-layout code.
    pub fn build_layout_with(
        mode: KeyboardMode,
        language: Language,
        shift_state: ShiftState,
        metrics: &LayoutMetrics,
        options: &LayoutOptions,
    ) -> Vec<Key> {
        let mut keys = if mode == KeyboardMode::Edit {
            Self::build_edit(metrics, options.edit_selecting)
        } else {
            Self::build_base_layout(mode, language, shift_state, metrics)
        };
        if mode == KeyboardMode::Alphabet && options.number_row && !keys.is_empty() {
            Self::add_number_row(&mut keys, metrics);
            Self::expand_invisible_hit_boxes(&mut keys, metrics);
        }
        if mode != KeyboardMode::Emoji && options.one_handed != OneHanded::Off && !keys.is_empty() {
            Self::make_one_handed(&mut keys, metrics, options.one_handed);
        }
        keys
    }

    /// Text editing panel: 4×4 grid of icons (drawn by SvgIcons) plus "ABC" back to letters.
    pub fn build_edit(m: &LayoutMetrics, selecting: bool) -> Vec<Key> {
        use super::key::EditAction as E;
        let rows: [[(E, &str); 4]; 4] = [
            [(E::SelectAll, "ed_select_all"), (E::Copy, "ed_copy"), (E::Cut, "ed_cut"), (E::Paste, "ed_paste")],
            [(E::Home, "ed_home"), (E::Up, "ed_up"), (E::End, "ed_end"), (E::Undo, "ed_undo")],
            [(E::Left, "ed_left"), (E::SelectMode, "ed_select"), (E::Right, "ed_right"), (E::Redo, "ed_redo")],
            [(E::Close, "ABC"), (E::Down, "ed_down"), (E::Close, ""), (E::Close, "")],
        ];
        let row_h = (m.key_area_height - 3.0 * m.key_spacing_v) / 4.0;
        let key_w = (m.total_width - 2.0 * m.padding_horizontal - 3.0 * m.key_spacing_h) / 4.0;
        let mut keys = Vec::with_capacity(16);
        let mut id = 1;
        for (r, row) in rows.iter().enumerate() {
            for (c, &(action, label)) in row.iter().enumerate() {
                let x = m.padding_horizontal + c as f32 * (key_w + m.key_spacing_h);
                let y = m.key_area_top + r as f32 * (row_h + m.key_spacing_v);
                // Bottom row: backspace and enter in the last two cells
                let (action, label, key_type) = match (r, c) {
                    (3, 2) => (KeyAction::Backspace, "⌫", KeyType::Modifier),
                    (3, 3) => (KeyAction::Enter, "↵", KeyType::Accent),
                    (_, _) if action == E::SelectMode && selecting => (KeyAction::Edit(action), label, KeyType::Accent),
                    (_, _) if action == E::Close => (KeyAction::Edit(action), label, KeyType::Modifier),
                    _ => (KeyAction::Edit(action), label, KeyType::Normal),
                };
                keys.push(Key::new(id, x, y, key_w, row_h, action, label, key_type));
                id += 1;
            }
        }
        Self::expand_invisible_hit_boxes(&mut keys, m);
        keys
    }

    /// Squeezes the rows down and puts the digits 1..0 on top.
    fn add_number_row(keys: &mut Vec<Key>, m: &LayoutMetrics) {
        let rows = 1 + keys
            .iter()
            .map(|k| (k.y * 10.0).round() as i64)
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        let row_h = (m.key_area_height - (rows as f32 - 1.0) * m.key_spacing_v) / rows as f32;
        let top = m.key_area_top;
        let offset = row_h + m.key_spacing_v;
        let scale = (m.key_area_height - offset) / m.key_area_height;
        for k in keys.iter_mut() {
            k.y = top + offset + (k.y - top) * scale;
            k.height *= scale;
        }
        let key_w = (m.total_width - 2.0 * m.padding_horizontal - 9.0 * m.key_spacing_h) / 10.0;
        let mut id = keys.iter().map(|k| k.id).max().unwrap_or(0) + 1;
        for (i, d) in "1234567890".chars().enumerate() {
            let x = m.padding_horizontal + i as f32 * (key_w + m.key_spacing_h);
            keys.push(Key::new(id, x, top, key_w, row_h, KeyAction::Character(d), d.to_string(), KeyType::Normal));
            id += 1;
        }
    }

    /// Narrows the keys towards one side and puts two control keys in the freed strip.
    fn make_one_handed(keys: &mut Vec<Key>, m: &LayoutMetrics, side: OneHanded) {
        let width = m.total_width * ONE_HANDED_WIDTH;
        let shift = if side == OneHanded::Right { m.total_width - width } else { 0.0 };
        for k in keys.iter_mut() {
            k.x = shift + k.x * ONE_HANDED_WIDTH;
            k.width *= ONE_HANDED_WIDTH;
            k.hit_x = shift + k.hit_x * ONE_HANDED_WIDTH;
            k.hit_width *= ONE_HANDED_WIDTH;
        }
        let strip_x = if side == OneHanded::Right { 0.0 } else { width };
        let strip_w = m.total_width - width;
        let pad = m.key_spacing_h * 2.0;
        let half = (m.key_area_height - m.key_spacing_v) / 2.0;
        let mut id = keys.iter().map(|k| k.id).max().unwrap_or(0) + 1;
        let switch_label = if side == OneHanded::Right { "oh_left" } else { "oh_right" };
        for (i, (action, label)) in
            [(KeyAction::OneHandedSwitchSide, switch_label), (KeyAction::OneHandedOff, "oh_full")].into_iter().enumerate()
        {
            let y = m.key_area_top + i as f32 * (half + m.key_spacing_v);
            let key = Key::new(id, strip_x + pad, y, strip_w - 2.0 * pad, half, action, label, KeyType::Modifier)
                .with_hit_box(strip_x, y, strip_w, half);
            keys.push(key);
            id += 1;
        }
    }

    fn build_base_layout(
        mode: KeyboardMode,
        language: Language,
        shift_state: ShiftState,
        metrics: &LayoutMetrics,
    ) -> Vec<Key> {
        let mut keys = match mode {
            KeyboardMode::Alphabet => match language {
                Language::Russian => Self::build_russian(shift_state, metrics),
                Language::English => Self::build_english(shift_state, metrics),
                Language::German => Self::build_german(shift_state, metrics),
                Language::French => Self::build_french(shift_state, metrics),
                Language::Spanish => Self::build_spanish(shift_state, metrics),
                Language::Portuguese => Self::build_portuguese(shift_state, metrics),
                Language::Italian => Self::build_italian(shift_state, metrics),
                Language::Turkish => Self::build_turkish(shift_state, metrics),
                Language::Ukrainian => Self::build_ukrainian(shift_state, metrics),
                Language::Belarusian => Self::build_belarusian(shift_state, metrics),
                Language::Kazakh => Self::build_kazakh(shift_state, metrics),
                Language::Arabic | Language::Persian | Language::Urdu => Self::build_arabic(shift_state, metrics),
                Language::Polish => Self::build_polish(shift_state, metrics),
                Language::Czech | Language::Slovak => Self::build_czech(shift_state, metrics),
                Language::Romanian => Self::build_romanian(shift_state, metrics),
                Language::Hebrew => Self::build_hebrew(shift_state, metrics),
                Language::Korean => Self::build_korean(shift_state, metrics),
                _ => Self::build_custom_qwerty(shift_state, language, metrics),
            },
            KeyboardMode::Numbers => Self::build_numbers(metrics),
            KeyboardMode::Symbols => Self::build_symbols(metrics),
            KeyboardMode::Emoji => Vec::new(),
            // Built by build_layout_with, which knows the selection state
            KeyboardMode::Edit => Self::build_edit(metrics, false),
        };

        Self::expand_invisible_hit_boxes(&mut keys, metrics);
        keys
    }

    pub fn expand_invisible_hit_boxes(keys: &mut [Key], metrics: &LayoutMetrics) {
        if keys.is_empty() {
            return;
        }

        // 1. Group keys into rows by their y position
        let mut indices: Vec<usize> = (0..keys.len()).collect();
        indices.sort_by(|&a, &b| {
            let ya = keys[a].y;
            let yb = keys[b].y;
            ya.partial_cmp(&yb).unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut row_indices: Vec<Vec<usize>> = Vec::new();
        for idx in indices {
            let key_y = keys[idx].y;
            let mut matched = false;
            for row in &mut row_indices {
                let first_idx = row[0];
                if (keys[first_idx].y - key_y).abs() < 8.0 {
                    row.push(idx);
                    matched = true;
                    break;
                }
            }
            if !matched {
                row_indices.push(vec![idx]);
            }
        }

        // Sort rows strictly from top to bottom
        row_indices.sort_by(|r1, r2| {
            let y1 = keys[r1[0]].y;
            let y2 = keys[r2[0]].y;
            y1.partial_cmp(&y2).unwrap_or(std::cmp::Ordering::Equal)
        });

        // For each row, sort keys strictly from left to right
        for row in &mut row_indices {
            row.sort_by(|&a, &b| {
                let xa = keys[a].x;
                let xb = keys[b].x;
                xa.partial_cmp(&xb).unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let num_rows = row_indices.len();
        if num_rows == 0 {
            return;
        }

        // Calculate vertical boundaries for each row
        let mut row_top_bounds = vec![0.0f32; num_rows];
        let mut row_bottom_bounds = vec![0.0f32; num_rows];

        for r in 0..num_rows {
            let row = &row_indices[r];
            let row_min_y = row.iter().map(|&i| keys[i].y).fold(f32::INFINITY, f32::min);
            let row_max_bottom = row
                .iter()
                .map(|&i| keys[i].y + keys[i].height)
                .fold(f32::NEG_INFINITY, f32::max);

            if r == 0 {
                // Top row extends up to suggestion bar
                row_top_bounds[r] = metrics.suggestion_bar_height.min(row_min_y);
            } else {
                let prev_row = &row_indices[r - 1];
                let prev_bottom = prev_row
                    .iter()
                    .map(|&i| keys[i].y + keys[i].height)
                    .fold(f32::NEG_INFINITY, f32::max);
                let mid_y = (prev_bottom + row_min_y) * 0.5;
                row_top_bounds[r] = mid_y;
                row_bottom_bounds[r - 1] = mid_y;
            }

            if r == num_rows - 1 {
                // Bottom row extends down to total height
                row_bottom_bounds[r] = metrics.total_height.max(row_max_bottom);
            }
        }

        // Calculate horizontal boundaries and assign hit-boxes for each key
        for (r, row) in row_indices.iter().enumerate() {
            let hit_y = row_top_bounds[r];
            let hit_height = (row_bottom_bounds[r] - hit_y).max(keys[row[0]].height);
            let row_len = row.len();

            let mut left_bounds = vec![0.0f32; row_len];
            let mut right_bounds = vec![0.0f32; row_len];

            for i in 0..row_len {
                let curr_key = &keys[row[i]];
                let curr_left = curr_key.x;
                let curr_right = curr_key.x + curr_key.width;

                if i == 0 {
                    // First key expands to the left screen edge
                    left_bounds[i] = 0.0;
                } else {
                    let prev_key = &keys[row[i - 1]];
                    let prev_right = prev_key.x + prev_key.width;
                    let mid_x = (prev_right + curr_left) * 0.5;
                    left_bounds[i] = mid_x;
                    right_bounds[i - 1] = mid_x;
                }

                if i == row_len - 1 {
                    // Last key expands to the right screen edge
                    right_bounds[i] = metrics.total_width.max(curr_right);
                }
            }

            for i in 0..row_len {
                let key_idx = row[i];
                let hit_x = left_bounds[i];
                let hit_width = (right_bounds[i] - hit_x).max(keys[key_idx].width);
                keys[key_idx].set_hit_box(hit_x, hit_y, hit_width, hit_height);
            }
        }
    }

    fn build_russian(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 4] = [
            &[
                ("й", Some("1"), &['1']),
                ("ц", Some("2"), &['2']),
                ("у", Some("3"), &['3']),
                ("к", Some("4"), &['4']),
                ("е", Some("5"), &['ё', '5']),
                ("н", Some("6"), &['6']),
                ("г", Some("7"), &['7']),
                ("ш", Some("8"), &['8']),
                ("щ", Some("9"), &['9']),
                ("з", Some("0"), &['0']),
                ("х", Some("%"), &['%']),
            ],
            &[
                ("ф", Some("@"), &['@']),
                ("ы", Some("#"), &['#']),
                ("в", Some("$"), &['$', '₽']),
                ("а", Some("&"), &['&']),
                ("п", Some("*"), &['*']),
                ("р", Some("-"), &['-', '_']),
                ("о", Some("+"), &['+']),
                ("л", Some("("), &['(']),
                ("д", Some(")"), &[')']),
                ("ж", Some("/"), &['/']),
                ("э", Some("\\"), &['\\']),
            ],
            &[
                // Shift is added separately
                ("я", Some("~"), &['~']),
                ("ч", Some("`"), &['`']),
                ("с", Some("<"), &['<']),
                ("м", Some(">"), &['>']),
                ("и", Some("["), &['[']),
                ("т", Some("]"), &[']']),
                ("ь", Some("ъ"), &['ъ', '{']),
                ("б", Some("}"), &['}']),
                ("ю", Some(";"), &[';']),
                // Backspace is added separately
            ],
            &[
                // Row 4 special layout
            ],
        ];

        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(40);
        let mut key_id = 1;

        // Row 1: 12 keys
        let r1_count = rows_data[0].len() as f32;
        let r1_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h)
                / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in rows_data[0] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2: 11 keys centered
        let r2_count = rows_data[1].len() as f32;
        let r2_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r2_count - 1.0) * m.key_spacing_h)
                / r2_count;
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in rows_data[1] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r2_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            r2_x += r2_key_w + m.key_spacing_h;
        }

        // Row 3: Shift + 9 letters + Backspace
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let r3_letter_count = rows_data[2].len() as f32; // 9
        let shift_backspace_w = r1_key_w * 1.30;
        let available_w = m.total_width
            - 2.0 * m.padding_horizontal
            - 2.0 * shift_backspace_w
            - 10.0 * m.key_spacing_h;
        let r3_letter_w = available_w / r3_letter_count;

        // Shift Key
        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Shift,
            match shift {
                ShiftState::CapsLock => "⇪",
                ShiftState::Shifted => "⬆",
                ShiftState::Off => "⇧",
            },
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + shift_backspace_w + m.key_spacing_h;
        for &(ch_str, sub, alts) in rows_data[2] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut final_sub = sub.map(|s| s.to_string());
            let mut final_alts = alts.to_vec();
            if ch == 'ь' {
                final_sub = Some(if is_upper { "Ъ".to_string() } else { "ъ".to_string() });
                final_alts = if is_upper { vec!['Ъ', '{'] } else { vec!['ъ', '{'] };
            }
            let mut key = Key::new(
                key_id,
                r3_x,
                r3_y,
                r3_letter_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(ref s) = final_sub {
                key = key.with_sub_label(s);
            }
            if !final_alts.is_empty() {
                key = key.with_alternates(final_alts);
            }
            keys.push(key);
            key_id += 1;
            r3_x += r3_letter_w + m.key_spacing_h;
        }

        // Backspace Key
        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [?123] [Globe] [Space] [,] [.] [Enter]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = shift_backspace_w;
        let enter_w = shift_backspace_w;
        let lang_btn_w = r1_key_w * 1.15;
        let comma_w = r1_key_w;
        let dot_w = r1_key_w;

        let total_fixed =
            mode_btn_w + lang_btn_w + comma_w + dot_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;

        // Mode switch
        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            mode_btn_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Numbers),
            "?123",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        // Lang switch
        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            lang_btn_w,
            row_height,
            KeyAction::SwitchLanguage,
            "🌐",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        // Spacebar
        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            space_w,
            row_height,
            KeyAction::Space,
            "RYNK",
            KeyType::Space,
        ));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        // Comma
        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                comma_w,
                row_height,
                KeyAction::Character(','),
                ",",
                KeyType::Normal,
            )
            .with_alternates(vec![';', ':', '_', '<']),
        );
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        // Dot
        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                dot_w,
                row_height,
                KeyAction::Character('.'),
                ".",
                KeyType::Normal,
            )
            .with_alternates(vec!['?', '!', '_', '>']),
        );
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        // Enter
        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            enter_w,
            row_height,
            KeyAction::Enter,
            "↵",
            KeyType::Accent,
        ));

        keys
    }

    fn build_english(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("q", Some("1"), &['1']),
                ("w", Some("2"), &['2']),
                ("e", Some("3"), &['3', 'é', 'ë', 'è', 'ê']),
                ("r", Some("4"), &['4']),
                ("t", Some("5"), &['5']),
                ("y", Some("6"), &['6', 'ÿ']),
                ("u", Some("7"), &['7', 'ú', 'ü', 'û', 'ù']),
                ("i", Some("8"), &['8', 'í', 'ï', 'î', 'ì']),
                ("o", Some("9"), &['9', 'ó', 'ö', 'ô', 'ò', 'œ']),
                ("p", Some("0"), &['0']),
            ],
            &[
                ("a", Some("@"), &['@', 'á', 'ä', 'à', 'â', 'æ', 'ã', 'å']),
                ("s", Some("#"), &['#', 'ß', 'ś', 'š']),
                ("d", Some("$"), &['$', '₽', '€', '£']),
                ("f", Some("%"), &['%']),
                ("g", Some("&"), &['&']),
                ("h", Some("*"), &['*']),
                ("j", Some("-"), &['-', '_']),
                ("k", Some("+"), &['+']),
                ("l", Some("/"), &['/']),
            ],
            &[
                ("z", Some("("), &['(']),
                ("x", Some(")"), &[')']),
                ("c", Some("\""), &['"', 'ç']),
                ("v", Some("'"), &['\'']),
                ("b", Some(":"), &[':']),
                ("n", Some(";"), &[';', 'ñ']),
                ("m", Some("!"), &['!']),
            ],
        ];

        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(36);
        let mut key_id = 100;

        // Row 1: 10 keys
        let r1_count = 10.0;
        let r1_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h)
                / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in rows_data[0] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2: 9 keys centered
        let r2_indent = r1_key_w * 0.5;
        let mut r2_x = m.padding_horizontal + r2_indent;
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in rows_data[1] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            r2_x += r1_key_w + m.key_spacing_h;
        }

        // Row 3: Shift + 7 letters + Backspace
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let shift_backspace_w =
            (m.total_width - 2.0 * m.padding_horizontal - 7.0 * r1_key_w - 8.0 * m.key_spacing_h)
                * 0.5;

        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Shift,
            match shift {
                ShiftState::CapsLock => "⇪",
                ShiftState::Shifted => "⬆",
                ShiftState::Off => "⇧",
            },
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + shift_backspace_w + m.key_spacing_h;
        for &(ch_str, sub, alts) in rows_data[2] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                r3_x,
                r3_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            r3_x += r1_key_w + m.key_spacing_h;
        }

        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [?123] [Globe] [Space] [,] [.] [Enter]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = shift_backspace_w;
        let enter_w = shift_backspace_w;
        let lang_btn_w = r1_key_w * 1.15;
        let comma_w = r1_key_w;
        let dot_w = r1_key_w;

        let total_fixed =
            mode_btn_w + lang_btn_w + comma_w + dot_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            mode_btn_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Numbers),
            "?123",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            lang_btn_w,
            row_height,
            KeyAction::SwitchLanguage,
            "🌐",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            space_w,
            row_height,
            KeyAction::Space,
            "RYNK",
            KeyType::Space,
        ));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                comma_w,
                row_height,
                KeyAction::Character(','),
                ",",
                KeyType::Normal,
            )
            .with_alternates(vec![';', ':', '_', '<']),
        );
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                dot_w,
                row_height,
                KeyAction::Character('.'),
                ".",
                KeyType::Normal,
            )
            .with_alternates(vec!['?', '!', '_', '>']),
        );
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            enter_w,
            row_height,
            KeyAction::Enter,
            "↵",
            KeyType::Accent,
        ));

        keys
    }

    fn build_generic_3row(
        shift: ShiftState,
        m: &LayoutMetrics,
        r1: &[(&str, Option<&str>, &[char])],
        r2: &[(&str, Option<&str>, &[char])],
        r3: &[(&str, Option<&str>, &[char])],
        mut key_id: u32,
    ) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(40);

        // Row 1
        let r1_count = r1.len() as f32;
        let r1_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h)
                / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in r1 {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2
        let r2_count = r2.len() as f32;
        let r2_indent = if r2_count < r1_count {
            (m.total_width
                - 2.0 * m.padding_horizontal
                - (r2_count * r1_key_w + (r2_count - 1.0) * m.key_spacing_h))
                * 0.5
        } else {
            0.0
        };
        let mut r2_x = m.padding_horizontal + r2_indent.max(0.0);
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in r2 {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            r2_x += r1_key_w + m.key_spacing_h;
        }

        // Row 3: Shift + R3 letters + Backspace
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let r3_count = r3.len() as f32;
        let r3_available = m.total_width - 2.0 * m.padding_horizontal;
        let (shift_w, backspace_w, r3_letter_w) = if r3_count <= 8.0 {
            let sb_w =
                (r3_available - (r3_count * r1_key_w) - (r3_count + 1.0) * m.key_spacing_h) * 0.5;
            (sb_w.max(r1_key_w * 1.3), sb_w.max(r1_key_w * 1.3), r1_key_w)
        } else {
            let sb_w = r1_key_w * 1.45;
            let letter_w =
                (r3_available - 2.0 * sb_w - (r3_count + 1.0) * m.key_spacing_h) / r3_count;
            (sb_w, sb_w, letter_w)
        };

        // Shift
        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            shift_w,
            row_height,
            KeyAction::Shift,
            match shift {
                ShiftState::CapsLock => "⇪",
                ShiftState::Shifted => "⬆",
                ShiftState::Off => "⇧",
            },
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + shift_w + m.key_spacing_h;
        for &(ch_str, sub, alts) in r3 {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper {
                ch.to_uppercase().next().unwrap()
            } else {
                ch
            };
            let mut key = Key::new(
                key_id,
                r3_x,
                r3_y,
                r3_letter_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub {
                key = key.with_sub_label(s);
            }
            if !alts.is_empty() {
                key = key.with_alternates(alts.to_vec());
            }
            keys.push(key);
            key_id += 1;
            r3_x += r3_letter_w + m.key_spacing_h;
        }

        // Backspace
        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [?123] [🌐] [Space] [,] [.] [↵]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = shift_w;
        let enter_w = backspace_w;
        let lang_btn_w = r1_key_w * 1.15;
        let comma_w = r1_key_w;
        let dot_w = r1_key_w;

        let total_fixed =
            mode_btn_w + lang_btn_w + comma_w + dot_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            mode_btn_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Numbers),
            "?123",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            lang_btn_w,
            row_height,
            KeyAction::SwitchLanguage,
            "🌐",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            space_w,
            row_height,
            KeyAction::Space,
            "RYNK",
            KeyType::Space,
        ));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                comma_w,
                row_height,
                KeyAction::Character(','),
                ",",
                KeyType::Normal,
            )
            .with_alternates(vec![';', ':', '_', '<']),
        );
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                dot_w,
                row_height,
                KeyAction::Character('.'),
                ".",
                KeyType::Normal,
            )
            .with_alternates(vec!['?', '!', '_', '>']),
        );
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            enter_w,
            row_height,
            KeyAction::Enter,
            "↵",
            KeyType::Accent,
        ));

        keys
    }

    fn build_german(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("1"), &['1']),
            ("w", Some("2"), &['2']),
            ("e", Some("3"), &['3', 'é']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("z", Some("6"), &['6']),
            ("u", Some("7"), &['7']),
            ("i", Some("8"), &['8']),
            ("o", Some("9"), &['9']),
            ("p", Some("0"), &['0']),
            ("ü", None, &['ü']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("@"), &['@']),
            ("s", Some("#"), &['#', 'ß']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
            ("ö", None, &['ö']),
            ("ä", None, &['ä']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("y", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
            ("m", Some("!"), &['!']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1100)
    }

    fn build_french(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("1"), &['1', 'à', 'â']),
            ("z", Some("2"), &['2']),
            ("e", Some("3"), &['3', 'é', 'è', 'ê', 'ë']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("y", Some("6"), &['6']),
            ("u", Some("7"), &['7', 'ù', 'û']),
            ("i", Some("8"), &['8', 'î', 'ï']),
            ("o", Some("9"), &['9', 'ô', 'œ']),
            ("p", Some("0"), &['0']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("@"), &['@']),
            ("s", Some("#"), &['#']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
            ("m", None, &[]),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("w", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"', 'ç']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1200)
    }

    fn build_spanish(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("1"), &['1']),
            ("w", Some("2"), &['2']),
            ("e", Some("3"), &['3', 'é']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("y", Some("6"), &['6']),
            ("u", Some("7"), &['7', 'ú', 'ü']),
            ("i", Some("8"), &['8', 'í']),
            ("o", Some("9"), &['9', 'ó']),
            ("p", Some("0"), &['0']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("@"), &['@', 'á']),
            ("s", Some("#"), &['#']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
            ("ñ", None, &['ñ']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("z", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
            ("m", Some("!"), &['!', '¡', '¿']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1300)
    }

    fn build_portuguese(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("1"), &['1']),
            ("w", Some("2"), &['2']),
            ("e", Some("3"), &['3', 'é', 'ê']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("y", Some("6"), &['6']),
            ("u", Some("7"), &['7', 'ú']),
            ("i", Some("8"), &['8', 'í']),
            ("o", Some("9"), &['9', 'ó', 'ô', 'õ']),
            ("p", Some("0"), &['0']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("@"), &['@', 'á', 'à', 'â', 'ã']),
            ("s", Some("#"), &['#']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
            ("ç", None, &['ç']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("z", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
            ("m", Some("!"), &['!']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1400)
    }

    fn build_italian(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("1"), &['1']),
            ("w", Some("2"), &['2']),
            ("e", Some("3"), &['3', 'è', 'é']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("y", Some("6"), &['6']),
            ("u", Some("7"), &['7', 'ù']),
            ("i", Some("8"), &['8', 'ì']),
            ("o", Some("9"), &['9', 'ò']),
            ("p", Some("0"), &['0']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("@"), &['@', 'à']),
            ("s", Some("#"), &['#']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("z", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
            ("m", Some("!"), &['!']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1500)
    }

    fn build_turkish(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("q", Some("1"), &['1']),
            ("w", Some("2"), &['2']),
            ("e", Some("3"), &['3']),
            ("r", Some("4"), &['4']),
            ("t", Some("5"), &['5']),
            ("y", Some("6"), &['6']),
            ("u", Some("7"), &['7']),
            ("ı", Some("8"), &['8', 'i']),
            ("o", Some("9"), &['9']),
            ("p", Some("0"), &['0']),
            ("ğ", None, &['ğ']),
            ("ü", None, &['ü']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("a", Some("@"), &['@']),
            ("s", Some("#"), &['#']),
            ("d", Some("$"), &['$']),
            ("f", Some("%"), &['%']),
            ("g", Some("&"), &['&']),
            ("h", Some("*"), &['*']),
            ("j", Some("-"), &['-']),
            ("k", Some("+"), &['+']),
            ("l", Some("/"), &['/']),
            ("ş", None, &['ş']),
            ("i", None, &['i', 'ı']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("z", Some("("), &['(']),
            ("x", Some(")"), &[')']),
            ("c", Some("\""), &['"']),
            ("v", Some("'"), &['\'']),
            ("b", Some(":"), &[':']),
            ("n", Some(";"), &[';']),
            ("m", Some("!"), &['!']),
            ("ö", None, &['ö']),
            ("ç", None, &['ç']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1600)
    }

    fn build_ukrainian(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("й", Some("1"), &['1']),
            ("ц", Some("2"), &['2']),
            ("у", Some("3"), &['3']),
            ("к", Some("4"), &['4']),
            ("е", Some("5"), &['5']),
            ("н", Some("6"), &['6']),
            ("г", Some("7"), &['7', 'ґ']),
            ("ш", Some("8"), &['8']),
            ("щ", Some("9"), &['9']),
            ("з", Some("0"), &['0']),
            ("х", Some("%"), &['%']),
            ("ї", Some("="), &['ї']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("ф", Some("@"), &['@']),
            ("і", Some("#"), &['#']),
            ("в", Some("$"), &['$', '₴']),
            ("а", Some("&"), &['&']),
            ("п", Some("*"), &['*']),
            ("р", Some("-"), &['-']),
            ("о", Some("+"), &['+']),
            ("л", Some("("), &['(']),
            ("д", Some(")"), &[')']),
            ("ж", Some("/"), &['/']),
            ("є", Some("\\"), &['є']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("я", Some("~"), &['~']),
            ("ч", Some("`"), &['`']),
            ("с", Some("<"), &['<']),
            ("м", Some(">"), &['>']),
            ("и", Some("["), &['[']),
            ("т", Some("]"), &[']']),
            ("ь", Some("{"), &['{', '\'']),
            ("б", Some("}"), &['}']),
            ("ю", Some(";"), &[';']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1700)
    }

    fn build_belarusian(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("й", Some("1"), &['1']),
            ("ц", Some("2"), &['2']),
            ("у", Some("3"), &['3']),
            ("к", Some("4"), &['4']),
            ("е", Some("5"), &['5', 'ё']),
            ("н", Some("6"), &['6']),
            ("г", Some("7"), &['7']),
            ("ш", Some("8"), &['8']),
            ("ў", Some("9"), &['9']),
            ("з", Some("0"), &['0']),
            ("х", Some("%"), &['%']),
            ("'", Some("="), &['\'']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("ф", Some("@"), &['@']),
            ("ы", Some("#"), &['#']),
            ("в", Some("$"), &['$']),
            ("а", Some("&"), &['&']),
            ("п", Some("*"), &['*']),
            ("р", Some("-"), &['-']),
            ("о", Some("+"), &['+']),
            ("л", Some("("), &['(']),
            ("д", Some(")"), &[')']),
            ("ж", Some("/"), &['/']),
            ("э", Some("\\"), &['э']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("я", Some("~"), &['~']),
            ("ч", Some("`"), &['`']),
            ("с", Some("<"), &['<']),
            ("м", Some(">"), &['>']),
            ("і", Some("["), &['[']),
            ("т", Some("]"), &[']']),
            ("ь", Some("{"), &['{']),
            ("б", Some("}"), &['}']),
            ("ю", Some(";"), &[';']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1800)
    }

    fn build_kazakh(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let r1: &[(&str, Option<&str>, &[char])] = &[
            ("ә", Some("1"), &['1']),
            ("і", Some("2"), &['2']),
            ("ң", Some("3"), &['3']),
            ("ғ", Some("4"), &['4']),
            ("ү", Some("5"), &['5']),
            ("ұ", Some("6"), &['6']),
            ("қ", Some("7"), &['7']),
            ("ө", Some("8"), &['8']),
            ("һ", Some("9"), &['9']),
            ("х", Some("0"), &['0']),
            ("ъ", Some("%"), &['%']),
            ("=", Some("+"), &['+']),
        ];
        let r2: &[(&str, Option<&str>, &[char])] = &[
            ("ф", Some("@"), &['@']),
            ("ы", Some("#"), &['#']),
            ("в", Some("$"), &['$', '₸']),
            ("а", Some("&"), &['&']),
            ("п", Some("*"), &['*']),
            ("р", Some("-"), &['-']),
            ("о", Some("+"), &['+']),
            ("л", Some("("), &['(']),
            ("д", Some(")"), &[')']),
            ("ж", Some("/"), &['/']),
            ("э", Some("\\"), &['э']),
        ];
        let r3: &[(&str, Option<&str>, &[char])] = &[
            ("я", Some("~"), &['~']),
            ("ч", Some("`"), &['`']),
            ("с", Some("<"), &['<']),
            ("м", Some(">"), &['>']),
            ("и", Some("["), &['[']),
            ("т", Some("]"), &[']']),
            ("ь", Some("{"), &['{']),
            ("б", Some("}"), &['}']),
            ("ю", Some(";"), &[';']),
        ];
        Self::build_generic_3row(shift, m, r1, r2, r3, 1900)
    }

    fn build_numbers(m: &LayoutMetrics) -> Vec<Key> {
        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(36);
        let mut key_id = 200;

        let r1_symbols = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];
        let r1_count = 10.0;
        let r1_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h)
                / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;

        for &sym in &r1_symbols {
            keys.push(Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            ));
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        let r2_symbols = ["@", "#", "$", "%", "&", "-", "+", "(", ")", "/"];
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;

        for &sym in &r2_symbols {
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r1_key_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            );
            if sym == "-" {
                key = key.with_alternates(vec!['_', '—', '–']).with_sub_label("_");
            }
            keys.push(key);
            key_id += 1;
            r2_x += r1_key_w + m.key_spacing_h;
        }

        // Row 3: [=\<] * " ' : ; ! ? [Backspace]
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let more_sym_w = r1_key_w * 1.35;
        let backspace_w = r1_key_w * 1.35;
        let r3_symbols = ["*", "\"", "'", ":", ";", "!", "?"];
        let mid_count = r3_symbols.len() as f32;
        let mid_w = (m.total_width
            - 2.0 * m.padding_horizontal
            - more_sym_w
            - backspace_w
            - 8.0 * m.key_spacing_h)
            / mid_count;

        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            more_sym_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Symbols),
            "=\\<",
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + more_sym_w + m.key_spacing_h;
        for &sym in &r3_symbols {
            keys.push(Key::new(
                key_id,
                r3_x,
                r3_y,
                mid_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            ));
            key_id += 1;
            r3_x += mid_w + m.key_spacing_h;
        }

        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [ABC] [Emoji] [Space] [,] [.] [Enter]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let abc_w = more_sym_w;
        let emoji_w = r1_key_w * 1.15;
        let comma_w = r1_key_w * 1.0;
        let dot_w = r1_key_w * 1.0;
        let enter_w = backspace_w * 1.25;

        let total_fixed = abc_w + emoji_w + comma_w + dot_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            abc_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Alphabet),
            "ABC",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += abc_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            emoji_w,
            row_height,
            KeyAction::SwitchEmoji,
            "😊",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += emoji_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            space_w,
            row_height,
            KeyAction::Space,
            "RYNK",
            KeyType::Space,
        ));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                comma_w,
                row_height,
                KeyAction::Character(','),
                ",",
                KeyType::Normal,
            )
            .with_alternates(vec![';', ':', '_', '<']),
        );
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(
            Key::new(
                key_id,
                r4_x,
                r4_y,
                dot_w,
                row_height,
                KeyAction::Character('.'),
                ".",
                KeyType::Normal,
            )
            .with_alternates(vec!['!', '?', '_', '>']),
        );
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            enter_w,
            row_height,
            KeyAction::Enter,
            "↵",
            KeyType::Accent,
        ));

        keys
    }

    fn build_symbols(m: &LayoutMetrics) -> Vec<Key> {
        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(36);
        let mut key_id = 300;

        let r1_symbols = ["~", "`", "|", "_", "•", "√", "π", "÷", "×", "¶"];
        let r1_count = 10.0;
        let r1_key_w =
            (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h)
                / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;

        for &sym in &r1_symbols {
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            );
            if sym == "_" {
                key = key.with_alternates(vec!['—', '–', '∆']);
            }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        let r2_symbols = ["£", "¥", "€", "¢", "^", "°", "=", "{", "}", "\\"];
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;

        for &sym in &r2_symbols {
            keys.push(Key::new(
                key_id,
                r2_x,
                r2_y,
                r1_key_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            ));
            key_id += 1;
            r2_x += r1_key_w + m.key_spacing_h;
        }

        // Row 3: [?123] % © ® ™ ✓ < > [Backspace]
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let prev_mode_w = r1_key_w * 1.35;
        let backspace_w = r1_key_w * 1.35;
        let r3_symbols = ["%", "©", "®", "™", "✓", "<", ">"];
        let mid_count = r3_symbols.len() as f32;
        let mid_w = (m.total_width
            - 2.0 * m.padding_horizontal
            - prev_mode_w
            - backspace_w
            - 8.0 * m.key_spacing_h)
            / mid_count;

        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            prev_mode_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Numbers),
            "?123",
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + prev_mode_w + m.key_spacing_h;
        for &sym in &r3_symbols {
            keys.push(Key::new(
                key_id,
                r3_x,
                r3_y,
                mid_w,
                row_height,
                KeyAction::Character(sym.chars().next().unwrap()),
                sym,
                KeyType::Normal,
            ));
            key_id += 1;
            r3_x += mid_w + m.key_spacing_h;
        }

        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [ABC] [Emoji] [Space] [«] [»] [Enter]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let abc_w = prev_mode_w;
        let emoji_w = r1_key_w * 1.15;
        let quote_l_w = r1_key_w * 1.0;
        let quote_r_w = r1_key_w * 1.0;
        let enter_w = backspace_w * 1.25;

        let total_fixed = abc_w + emoji_w + quote_l_w + quote_r_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            abc_w,
            row_height,
            KeyAction::SwitchMode(KeyboardMode::Alphabet),
            "ABC",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += abc_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            emoji_w,
            row_height,
            KeyAction::SwitchEmoji,
            "😊",
            KeyType::Modifier,
        ));
        key_id += 1;
        r4_x += emoji_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            space_w,
            row_height,
            KeyAction::Space,
            "RYNK",
            KeyType::Space,
        ));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            quote_l_w,
            row_height,
            KeyAction::Character('«'),
            "«",
            KeyType::Normal,
        ));
        key_id += 1;
        r4_x += quote_l_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            quote_r_w,
            row_height,
            KeyAction::Character('»'),
            "»",
            KeyType::Normal,
        ));
        key_id += 1;
        r4_x += quote_r_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            enter_w,
            row_height,
            KeyAction::Enter,
            "↵",
            KeyType::Accent,
        ));

        keys
    }

    fn build_arabic(_shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("ض", Some("١"), &['١']),
                ("ص", Some("٢"), &['٢']),
                ("ث", Some("٣"), &['٣']),
                ("ق", Some("٤"), &['٤']),
                ("ف", Some("٥"), &['٥']),
                ("غ", Some("٦"), &['٦']),
                ("ع", Some("٧"), &['٧']),
                ("ه", Some("٨"), &['٨']),
                ("خ", Some("٩"), &['٩']),
                ("ح", Some("٠"), &['٠']),
                ("ج", Some("%"), &['%']),
                ("د", Some("/"), &['/']),
            ],
            &[
                ("ش", Some("@"), &['@']),
                ("س", Some("#"), &['#']),
                ("ي", Some("$"), &['$']),
                ("ب", Some("&"), &['&']),
                ("ل", Some("*"), &['*']),
                ("ا", Some("-"), &['أ', 'إ', 'آ', 'ٱ']),
                ("ت", Some("+"), &['+']),
                ("ن", Some("("), &['(']),
                ("م", Some(")"), &[')']),
                ("ك", Some(":"), &[':']),
                ("ط", Some(";"), &[';']),
            ],
            &[
                ("ئ", Some("!"), &['!']),
                ("ء", Some("?"), &['?']),
                ("ؤ", Some("\""), &['\"']),
                ("ر", Some("'"), &['\'']),
                ("لا", Some("~"), &['ل', 'ا']),
                ("ى", Some("`"), &['`']),
                ("ة", Some("^"), &['^']),
                ("و", Some("="), &['=']),
                ("ز", Some("<"), &['<']),
                ("ظ", Some(">"), &['>']),
            ],
        ];

        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(40);
        let mut key_id = 1;

        // Row 1: 12 keys
        let r1_count = rows_data[0].len() as f32;
        let r1_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h) / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in rows_data[0] {
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(ch_str.chars().next().unwrap()),
                ch_str.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2: 11 keys
        let r2_count = rows_data[1].len() as f32;
        let r2_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r2_count - 1.0) * m.key_spacing_h) / r2_count;
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in rows_data[1] {
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r2_key_w,
                row_height,
                KeyAction::Character(ch_str.chars().next().unwrap()),
                ch_str.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r2_x += r2_key_w + m.key_spacing_h;
        }

        // Row 3: 10 keys + Backspace
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let bs_w = r1_key_w * 1.4;
        let r3_count = rows_data[2].len() as f32;
        let r3_key_w = (m.total_width - 2.0 * m.padding_horizontal - bs_w - r3_count * m.key_spacing_h) / r3_count;
        let mut r3_x = m.padding_horizontal;

        for &(ch_str, sub, alts) in rows_data[2] {
            let mut key = Key::new(
                key_id,
                r3_x,
                r3_y,
                r3_key_w,
                row_height,
                KeyAction::Character(ch_str.chars().next().unwrap()),
                ch_str.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r3_x += r3_key_w + m.key_spacing_h;
        }

        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            bs_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4: [?١٢٣] [Globe] [Space] [.] [،] [Enter]
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = bs_w;
        let enter_w = bs_w;
        let lang_btn_w = r1_key_w * 1.15;
        let dot_w = r1_key_w;
        let comma_w = r1_key_w;
        let total_fixed = mode_btn_w + lang_btn_w + dot_w + comma_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;
        keys.push(Key::new(key_id, r4_x, r4_y, mode_btn_w, row_height, KeyAction::SwitchMode(KeyboardMode::Numbers), "?١٢٣", KeyType::Modifier));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, lang_btn_w, row_height, KeyAction::SwitchLanguage, "🌐", KeyType::Modifier));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, space_w, row_height, KeyAction::Space, "مسافة", KeyType::Space));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, dot_w, row_height, KeyAction::Character('.'), ".", KeyType::Normal));
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, comma_w, row_height, KeyAction::Character('،'), "،", KeyType::Normal));
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, enter_w, row_height, KeyAction::Enter, "↵", KeyType::Accent));

        keys
    }

    fn build_polish(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("q", Some("1"), &['1']),
                ("w", Some("2"), &['2']),
                ("e", Some("3"), &['ę', '3']),
                ("r", Some("4"), &['4']),
                ("t", Some("5"), &['5']),
                ("y", Some("6"), &['6']),
                ("u", Some("7"), &['7']),
                ("i", Some("8"), &['8']),
                ("o", Some("9"), &['ó', '9']),
                ("p", Some("0"), &['0']),
            ],
            &[
                ("a", Some("@"), &['ą', '@']),
                ("s", Some("#"), &['ś', '#']),
                ("d", Some("$"), &['$']),
                ("f", Some("%"), &['%']),
                ("g", Some("&"), &['&']),
                ("h", Some("-"), &['-']),
                ("j", Some("+"), &['+']),
                ("k", Some("("), &['(']),
                ("l", Some(")"), &['ł', ')']),
            ],
            &[
                ("z", Some("*"), &['ż', 'ź', '*']),
                ("x", Some("\""), &['\"']),
                ("c", Some("'"), &['ć', '\'']),
                ("v", Some(":"), &[':']),
                ("b", Some(";"), &[';']),
                ("n", Some("!"), &['ń', '!']),
                ("m", Some("?"), &['?']),
            ],
        ];

        Self::build_latin_keyboard_internal(rows_data, is_upper, shift, "spacja", m)
    }

    fn build_czech(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("q", Some("1"), &['1']),
                ("w", Some("2"), &['2']),
                ("e", Some("3"), &['ě', 'é', '3']),
                ("r", Some("4"), &['ř', '4']),
                ("t", Some("5"), &['ť', '5']),
                ("y", Some("6"), &['ý', '6']),
                ("u", Some("7"), &['ů', 'ú', '7']),
                ("i", Some("8"), &['í', '8']),
                ("o", Some("9"), &['ó', '9']),
                ("p", Some("0"), &['0']),
            ],
            &[
                ("a", Some("@"), &['á', '@']),
                ("s", Some("#"), &['š', '#']),
                ("d", Some("$"), &['ď', '$']),
                ("f", Some("%"), &['%']),
                ("g", Some("&"), &['&']),
                ("h", Some("-"), &['-']),
                ("j", Some("+"), &['+']),
                ("k", Some("("), &['(']),
                ("l", Some(")"), &[')']),
            ],
            &[
                ("z", Some("*"), &['ž', '*']),
                ("x", Some("\""), &['\"']),
                ("c", Some("'"), &['č', '\'']),
                ("v", Some(":"), &[':']),
                ("b", Some(";"), &[';']),
                ("n", Some("!"), &['ň', '!']),
                ("m", Some("?"), &['?']),
            ],
        ];

        Self::build_latin_keyboard_internal(rows_data, is_upper, shift, "mezerník", m)
    }

    fn build_romanian(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("q", Some("1"), &['1']),
                ("w", Some("2"), &['2']),
                ("e", Some("3"), &['3']),
                ("r", Some("4"), &['4']),
                ("t", Some("5"), &['ț', '5']),
                ("y", Some("6"), &['6']),
                ("u", Some("7"), &['7']),
                ("i", Some("8"), &['î', '8']),
                ("o", Some("9"), &['9']),
                ("p", Some("0"), &['0']),
            ],
            &[
                ("a", Some("@"), &['ă', 'â', '@']),
                ("s", Some("#"), &['ș', '#']),
                ("d", Some("$"), &['$']),
                ("f", Some("%"), &['%']),
                ("g", Some("&"), &['&']),
                ("h", Some("-"), &['-']),
                ("j", Some("+"), &['+']),
                ("k", Some("("), &['(']),
                ("l", Some(")"), &[')']),
            ],
            &[
                ("z", Some("*"), &['*']),
                ("x", Some("\""), &['\"']),
                ("c", Some("'"), &['\'']),
                ("v", Some(":"), &[':']),
                ("b", Some(";"), &[';']),
                ("n", Some("!"), &['!']),
                ("m", Some("?"), &['?']),
            ],
        ];

        Self::build_latin_keyboard_internal(rows_data, is_upper, shift, "spațiu", m)
    }

    fn build_hebrew(_shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("ק", Some("1"), &['1']),
                ("ר", Some("2"), &['2']),
                ("א", Some("3"), &['3']),
                ("ט", Some("4"), &['4']),
                ("ו", Some("5"), &['5']),
                ("ן", Some("6"), &['6']),
                ("ם", Some("7"), &['7']),
                ("פ", Some("8"), &['8']),
            ],
            &[
                ("ש", Some("@"), &['@']),
                ("ד", Some("#"), &['#']),
                ("ג", Some("$"), &['$']),
                ("כ", Some("%"), &['%']),
                ("ע", Some("&"), &['&']),
                ("י", Some("-"), &['-']),
                ("ח", Some("+"), &['+']),
                ("ל", Some("("), &['(']),
                ("ך", Some(")"), &[')']),
                ("ף", Some(":"), &[':']),
            ],
            &[
                ("ז", Some("*"), &['*']),
                ("ס", Some("\""), &['\"']),
                ("ב", Some("'"), &['\'']),
                ("ה", Some("?"), &['?']),
                ("נ", Some("!"), &['!']),
                ("מ", Some(";"), &[';']),
                ("צ", Some("<"), &['<']),
                ("ת", Some(">"), &['>']),
                ("ץ", Some("/"), &['/']),
            ],
        ];

        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(36);
        let mut key_id = 1;

        // Row 1
        let r1_count = rows_data[0].len() as f32;
        let r1_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h) / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in rows_data[0] {
            let mut key = Key::new(key_id, curr_x, r1_y, r1_key_w, row_height, KeyAction::Character(ch_str.chars().next().unwrap()), ch_str.to_string(), KeyType::Normal);
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2
        let r2_count = rows_data[1].len() as f32;
        let r2_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r2_count - 1.0) * m.key_spacing_h) / r2_count;
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in rows_data[1] {
            let mut key = Key::new(key_id, r2_x, r2_y, r2_key_w, row_height, KeyAction::Character(ch_str.chars().next().unwrap()), ch_str.to_string(), KeyType::Normal);
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r2_x += r2_key_w + m.key_spacing_h;
        }

        // Row 3
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let bs_w = r1_key_w * 1.3;
        let r3_count = rows_data[2].len() as f32;
        let r3_key_w = (m.total_width - 2.0 * m.padding_horizontal - bs_w - r3_count * m.key_spacing_h) / r3_count;
        let mut r3_x = m.padding_horizontal;
        for &(ch_str, sub, alts) in rows_data[2] {
            let mut key = Key::new(key_id, r3_x, r3_y, r3_key_w, row_height, KeyAction::Character(ch_str.chars().next().unwrap()), ch_str.to_string(), KeyType::Normal);
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r3_x += r3_key_w + m.key_spacing_h;
        }
        keys.push(Key::new(key_id, r3_x, r3_y, bs_w, row_height, KeyAction::Backspace, "⌫", KeyType::Modifier));
        key_id += 1;

        // Row 4
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = bs_w;
        let enter_w = bs_w;
        let lang_btn_w = r1_key_w * 1.15;
        let dot_w = r1_key_w;
        let comma_w = r1_key_w;
        let total_fixed = mode_btn_w + lang_btn_w + dot_w + comma_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;
        keys.push(Key::new(key_id, r4_x, r4_y, mode_btn_w, row_height, KeyAction::SwitchMode(KeyboardMode::Numbers), "?123", KeyType::Modifier));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, lang_btn_w, row_height, KeyAction::SwitchLanguage, "🌐", KeyType::Modifier));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, space_w, row_height, KeyAction::Space, "רווח", KeyType::Space));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, comma_w, row_height, KeyAction::Character(','), ",", KeyType::Normal).with_alternates(vec![';', ':', '_', '<']));
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, dot_w, row_height, KeyAction::Character('.'), ".", KeyType::Normal).with_alternates(vec!['!', '?', '_', '>']));
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, enter_w, row_height, KeyAction::Enter, "↵", KeyType::Accent));

        keys
    }

    fn build_korean(shift: ShiftState, m: &LayoutMetrics) -> Vec<Key> {
        let is_shifted = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                (if is_shifted { "ㅃ" } else { "ㅂ" }, Some("1"), &['1']),
                (if is_shifted { "ㅉ" } else { "ㅈ" }, Some("2"), &['2']),
                (if is_shifted { "ㄸ" } else { "ㄷ" }, Some("3"), &['3']),
                (if is_shifted { "ㄲ" } else { "ㄱ" }, Some("4"), &['4']),
                (if is_shifted { "ㅆ" } else { "ㅅ" }, Some("5"), &['5']),
                ("ㅛ", Some("6"), &['6']),
                ("ㅕ", Some("7"), &['7']),
                ("ㅑ", Some("8"), &['8']),
                (if is_shifted { "ㅒ" } else { "ㅐ" }, Some("9"), &['9']),
                (if is_shifted { "ㅖ" } else { "ㅔ" }, Some("0"), &['0']),
            ],
            &[
                ("ㅁ", Some("@"), &['@']),
                ("ㄴ", Some("#"), &['#']),
                ("ㅇ", Some("$"), &['$']),
                ("ㄹ", Some("%"), &['%']),
                ("ㅎ", Some("&"), &['&']),
                ("ㅗ", Some("-"), &['-']),
                ("ㅓ", Some("+"), &['+']),
                ("ㅏ", Some("("), &['(']),
                ("ㅣ", Some(")"), &[')']),
            ],
            &[
                ("ㅋ", Some("*"), &['*']),
                ("ㅌ", Some("\""), &['\"']),
                ("ㅊ", Some("'"), &['\'']),
                ("ㅍ", Some(":"), &[':']),
                ("ㅠ", Some(";"), &[';']),
                ("ㅜ", Some("!"), &['!']),
                ("ㅡ", Some("?"), &['?']),
            ],
        ];

        Self::build_latin_keyboard_internal(rows_data, false, shift, "간격", m)
    }

    fn build_custom_qwerty(shift: ShiftState, lang: Language, m: &LayoutMetrics) -> Vec<Key> {
        let is_upper = shift.is_uppercase();
        let rows_data: [&[KeyDef]; 3] = [
            &[
                ("q", Some("1"), &['1']),
                ("w", Some("2"), &['2']),
                ("e", Some("3"), &['3']),
                ("r", Some("4"), &['4']),
                ("t", Some("5"), &['5']),
                ("y", Some("6"), &['6']),
                ("u", Some("7"), &['7']),
                ("i", Some("8"), &['8']),
                ("o", Some("9"), &['9']),
                ("p", Some("0"), &['0']),
            ],
            &[
                ("a", Some("@"), &['@']),
                ("s", Some("#"), &['#']),
                ("d", Some("$"), &['$']),
                ("f", Some("%"), &['%']),
                ("g", Some("&"), &['&']),
                ("h", Some("-"), &['-', '_']),
                ("j", Some("+"), &['+']),
                ("k", Some("("), &['(']),
                ("l", Some(")"), &[')']),
            ],
            &[
                ("z", Some("*"), &['*']),
                ("x", Some("\""), &['\"']),
                ("c", Some("'"), &['\'']),
                ("v", Some(":"), &[':']),
                ("b", Some(";"), &[';']),
                ("n", Some("!"), &['!']),
                ("m", Some("?"), &['?']),
            ],
        ];

        Self::build_latin_keyboard_internal(rows_data, is_upper, shift, lang.space_label(), m)
    }

    fn build_latin_keyboard_internal(
        rows_data: [&[KeyDef]; 3],
        is_upper: bool,
        shift: ShiftState,
        space_text: &str,
        m: &LayoutMetrics,
    ) -> Vec<Key> {
        let num_rows = 4.0;
        let row_height = (m.key_area_height - (num_rows - 1.0) * m.key_spacing_v) / num_rows;
        let mut keys = Vec::with_capacity(36);
        let mut key_id = 1;

        // Row 1: 10 keys
        let r1_count = rows_data[0].len() as f32;
        let r1_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r1_count - 1.0) * m.key_spacing_h) / r1_count;
        let mut curr_x = m.padding_horizontal;
        let r1_y = m.key_area_top;
        for &(ch_str, sub, alts) in rows_data[0] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper { ch.to_uppercase().next().unwrap() } else { ch };
            let mut key = Key::new(
                key_id,
                curr_x,
                r1_y,
                r1_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            curr_x += r1_key_w + m.key_spacing_h;
        }

        // Row 2: 9 keys
        let r2_count = rows_data[1].len() as f32;
        let r2_key_w = (m.total_width - 2.0 * m.padding_horizontal - (r2_count - 1.0) * m.key_spacing_h) / r2_count;
        let mut r2_x = m.padding_horizontal;
        let r2_y = r1_y + row_height + m.key_spacing_v;
        for &(ch_str, sub, alts) in rows_data[1] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper { ch.to_uppercase().next().unwrap() } else { ch };
            let mut key = Key::new(
                key_id,
                r2_x,
                r2_y,
                r2_key_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r2_x += r2_key_w + m.key_spacing_h;
        }

        // Row 3: Shift + 7 letters + Backspace
        let r3_y = r2_y + row_height + m.key_spacing_v;
        let r3_letter_count = rows_data[2].len() as f32;
        let shift_backspace_w = r1_key_w * 1.30;
        let available_w = m.total_width
            - 2.0 * m.padding_horizontal
            - 2.0 * shift_backspace_w
            - (r3_letter_count + 1.0) * m.key_spacing_h;
        let r3_letter_w = available_w / r3_letter_count;

        keys.push(Key::new(
            key_id,
            m.padding_horizontal,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Shift,
            match shift {
                ShiftState::CapsLock => "⇪",
                ShiftState::Shifted => "⬆",
                ShiftState::Off => "⇧",
            },
            KeyType::Modifier,
        ));
        key_id += 1;

        let mut r3_x = m.padding_horizontal + shift_backspace_w + m.key_spacing_h;
        for &(ch_str, sub, alts) in rows_data[2] {
            let ch = ch_str.chars().next().unwrap();
            let final_ch = if is_upper { ch.to_uppercase().next().unwrap() } else { ch };
            let mut key = Key::new(
                key_id,
                r3_x,
                r3_y,
                r3_letter_w,
                row_height,
                KeyAction::Character(final_ch),
                final_ch.to_string(),
                KeyType::Normal,
            );
            if let Some(s) = sub { key = key.with_sub_label(s); }
            if !alts.is_empty() { key = key.with_alternates(alts.to_vec()); }
            keys.push(key);
            key_id += 1;
            r3_x += r3_letter_w + m.key_spacing_h;
        }

        keys.push(Key::new(
            key_id,
            r3_x,
            r3_y,
            shift_backspace_w,
            row_height,
            KeyAction::Backspace,
            "⌫",
            KeyType::Modifier,
        ));
        key_id += 1;

        // Row 4
        let r4_y = r3_y + row_height + m.key_spacing_v;
        let mode_btn_w = shift_backspace_w;
        let enter_w = shift_backspace_w;
        let lang_btn_w = r1_key_w * 1.15;
        let comma_w = r1_key_w;
        let dot_w = r1_key_w;
        let total_fixed = mode_btn_w + lang_btn_w + comma_w + dot_w + enter_w + 5.0 * m.key_spacing_h;
        let space_w = (m.total_width - 2.0 * m.padding_horizontal - total_fixed).max(60.0);

        let mut r4_x = m.padding_horizontal;
        keys.push(Key::new(key_id, r4_x, r4_y, mode_btn_w, row_height, KeyAction::SwitchMode(KeyboardMode::Numbers), "?123", KeyType::Modifier));
        key_id += 1;
        r4_x += mode_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, lang_btn_w, row_height, KeyAction::SwitchLanguage, "🌐", KeyType::Modifier));
        key_id += 1;
        r4_x += lang_btn_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, space_w, row_height, KeyAction::Space, space_text, KeyType::Space));
        key_id += 1;
        r4_x += space_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, comma_w, row_height, KeyAction::Character(','), ",", KeyType::Normal).with_alternates(vec![';', ':', '_', '<']));
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, dot_w, row_height, KeyAction::Character('.'), ".", KeyType::Normal).with_alternates(vec!['!', '?', '_', '>']));
        key_id += 1;
        r4_x += dot_w + m.key_spacing_h;

        keys.push(Key::new(key_id, r4_x, r4_y, enter_w, row_height, KeyAction::Enter, "↵", KeyType::Accent));

        keys
    }

}
