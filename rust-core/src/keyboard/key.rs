#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyAction {
    Character(char),
    Shift,
    Backspace,
    Enter,
    SwitchMode(KeyboardMode),
    SwitchLanguage,
    SwitchEmoji,
    Space,
    Settings,
    HideKeyboard,
    SwitchInputMethod,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardMode {
    Alphabet,
    Numbers,
    Symbols,
    Emoji,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyType {
    Normal,
    Modifier,
    Accent,
    Space,
    Icon,
}

#[derive(Clone, Debug)]
pub struct Key {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    // Invisible touch hit-box expanding over gaps and borders
    pub hit_x: f32,
    pub hit_y: f32,
    pub hit_width: f32,
    pub hit_height: f32,
    pub action: KeyAction,
    pub label: String,
    pub sub_label: Option<String>,
    pub alternate_chars: Vec<char>,
    pub key_type: KeyType,
    pub is_pressed: bool,
    pub press_time_ms: u64,
}

impl Key {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        action: KeyAction,
        label: impl Into<String>,
        key_type: KeyType,
    ) -> Self {
        Self {
            id,
            x,
            y,
            width,
            height,
            hit_x: x,
            hit_y: y,
            hit_width: width,
            hit_height: height,
            action,
            label: label.into(),
            sub_label: None,
            alternate_chars: Vec::new(),
            key_type,
            is_pressed: false,
            press_time_ms: 0,
        }
    }

    pub fn with_sub_label(mut self, sub: impl Into<String>) -> Self {
        self.sub_label = Some(sub.into());
        self
    }

    pub fn with_alternates(mut self, alts: Vec<char>) -> Self {
        self.alternate_chars = alts;
        self
    }

    pub fn with_hit_box(mut self, hx: f32, hy: f32, hw: f32, hh: f32) -> Self {
        self.hit_x = hx;
        self.hit_y = hy;
        self.hit_width = hw;
        self.hit_height = hh;
        self
    }

    pub fn set_hit_box(&mut self, hx: f32, hy: f32, hw: f32, hh: f32) {
        self.hit_x = hx;
        self.hit_y = hy;
        self.hit_width = hw;
        self.hit_height = hh;
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.hit_x
            && px <= (self.hit_x + self.hit_width)
            && py >= self.hit_y
            && py <= (self.hit_y + self.hit_height)
    }

    pub fn contains_visual(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= (self.x + self.width) && py >= self.y && py <= (self.y + self.height)
    }

    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width * 0.5, self.y + self.height * 0.5)
    }
}
