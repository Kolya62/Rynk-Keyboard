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

pub struct LayoutBuilder;

impl LayoutBuilder {
    pub fn build_layout(
        mode: KeyboardMode,
        language: Language,
        shift_state: ShiftState,
        metrics: &LayoutMetrics,
    ) -> Vec<Key> {
        match mode {
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
            },
            KeyboardMode::Numbers => Self::build_numbers(metrics),
            KeyboardMode::Symbols => Self::build_symbols(metrics),
            KeyboardMode::Emoji => Vec::new(),
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
                ("р", Some("-"), &['-']),
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
            .with_alternates(vec![';', ':', '<']),
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
            .with_alternates(vec!['?', '!', '>']),
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
                ("j", Some("-"), &['-']),
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
            .with_alternates(vec![';', ':', '<']),
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
            .with_alternates(vec!['?', '!', '>']),
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
            .with_alternates(vec![';', ':', '<']),
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
            .with_alternates(vec!['?', '!', '>']),
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

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            comma_w,
            row_height,
            KeyAction::Character(','),
            ",",
            KeyType::Normal,
        ));
        key_id += 1;
        r4_x += comma_w + m.key_spacing_h;

        keys.push(Key::new(
            key_id,
            r4_x,
            r4_y,
            dot_w,
            row_height,
            KeyAction::Character('.'),
            ".",
            KeyType::Normal,
        ));
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

        let r1_symbols = ["~", "`", "|", "•", "√", "π", "÷", "×", "¶", "∆"];
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
}
