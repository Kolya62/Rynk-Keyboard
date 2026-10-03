use super::key::KeyboardMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftState {
    Off,
    Shifted,
    CapsLock,
}

impl ShiftState {
    pub fn is_uppercase(&self) -> bool {
        matches!(self, ShiftState::Shifted | ShiftState::CapsLock)
    }

    pub fn cycle_on_tap(&self) -> Self {
        match self {
            ShiftState::Off => ShiftState::Shifted,
            ShiftState::Shifted => ShiftState::Off,
            ShiftState::CapsLock => ShiftState::Off,
        }
    }

    pub fn on_char_typed(&self) -> Self {
        match self {
            ShiftState::Shifted => ShiftState::Off,
            other => *other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Russian,
    English,
    German,
    French,
    Spanish,
    Portuguese,
    Italian,
    Turkish,
    Ukrainian,
    Belarusian,
    Kazakh,
}

impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Russian => "Русский",
            Language::English => "English",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Portuguese => "Português",
            Language::Italian => "Italiano",
            Language::Turkish => "Türkçe",
            Language::Ukrainian => "Українська",
            Language::Belarusian => "Беларуская",
            Language::Kazakh => "Қазақша",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Language::Russian => "RU",
            Language::English => "EN",
            Language::German => "DE",
            Language::French => "FR",
            Language::Spanish => "ES",
            Language::Portuguese => "PT",
            Language::Italian => "IT",
            Language::Turkish => "TR",
            Language::Ukrainian => "UK",
            Language::Belarusian => "BE",
            Language::Kazakh => "KK",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Russian => "ru",
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Turkish => "tr",
            Language::Ukrainian => "uk",
            Language::Belarusian => "be",
            Language::Kazakh => "kk",
        }
    }

    pub fn to_id(&self) -> i32 {
        match self {
            Language::Russian => 0,
            Language::English => 1,
            Language::German => 2,
            Language::French => 3,
            Language::Spanish => 4,
            Language::Portuguese => 5,
            Language::Italian => 6,
            Language::Turkish => 7,
            Language::Ukrainian => 8,
            Language::Belarusian => 9,
            Language::Kazakh => 10,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            1 => Language::English,
            2 => Language::German,
            3 => Language::French,
            4 => Language::Spanish,
            5 => Language::Portuguese,
            6 => Language::Italian,
            7 => Language::Turkish,
            8 => Language::Ukrainian,
            9 => Language::Belarusian,
            10 => Language::Kazakh,
            _ => Language::Russian,
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "ru" => Some(Language::Russian),
            "en" => Some(Language::English),
            "de" => Some(Language::German),
            "fr" => Some(Language::French),
            "es" => Some(Language::Spanish),
            "pt" => Some(Language::Portuguese),
            "it" => Some(Language::Italian),
            "tr" => Some(Language::Turkish),
            "uk" => Some(Language::Ukrainian),
            "be" => Some(Language::Belarusian),
            "kk" => Some(Language::Kazakh),
            _ => None,
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Russian => Language::English,
            Language::English => Language::Russian,
            other => *other,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyboardOutputEvent {
    CommitText(String),
    DeleteSurroundingText { before: u32, after: u32 },
    SendKeyEvent(i32),
    PerformHaptic(HapticFeedbackType),
    SwitchInputMethod,
    OpenSettings,
    MoveCursor(i32),
    DeleteWord,
    HideKeyboard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticFeedbackType {
    KeyTick,
    KeyClick,
    KeyHeavyClick,
    LongPress,
}

#[derive(Clone, Debug)]
pub struct KeyboardState {
    pub language: Language,
    pub mode: KeyboardMode,
    pub shift_state: ShiftState,
    pub composing_text: String,
    pub last_committed_word: String,
    pub last_autocorrect_original: Option<String>,
    pub last_autocorrect_replacement: Option<String>,
    pub rejected_autocorrect_word: Option<String>,
    pub last_char_was_space: bool,
    pub clipboard_preview: Option<String>,
    pub output_events: Vec<KeyboardOutputEvent>,
    pub space_swipe_dx: f32,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            language: Language::Russian,
            mode: KeyboardMode::Alphabet,
            shift_state: ShiftState::Off,
            composing_text: String::with_capacity(64),
            last_committed_word: String::with_capacity(32),
            last_autocorrect_original: None,
            last_autocorrect_replacement: None,
            rejected_autocorrect_word: None,
            last_char_was_space: false,
            clipboard_preview: None,
            output_events: Vec::with_capacity(16),
            space_swipe_dx: 0.0,
        }
    }
}

impl KeyboardState {
    pub fn push_event(&mut self, event: KeyboardOutputEvent) {
        self.output_events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<KeyboardOutputEvent> {
        std::mem::take(&mut self.output_events)
    }
}
