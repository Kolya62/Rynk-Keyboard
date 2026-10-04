//! Prints the letters typeable on a language's layout: `cargo run -p dictgen --example typeable -- bg el`
use rynk_core::keyboard::key::{KeyAction, KeyboardMode};
use rynk_core::keyboard::layout::{LayoutBuilder, LayoutMetrics};
use rynk_core::keyboard::state::{Language, ShiftState};

fn main() {
    let metrics = LayoutMetrics::new(1080.0, 800.0, 2.75);
    for code in std::env::args().skip(1) {
        let lang = Language::from_code(&code).expect("language code");
        let mut keys = String::new();
        let mut alts = String::new();
        for key in
            LayoutBuilder::build_layout(KeyboardMode::Alphabet, lang, ShiftState::Off, &metrics)
        {
            if let KeyAction::Character(c) = key.action {
                keys.push(c);
            }
            alts.extend(key.alternate_chars.iter());
        }
        println!("{code}: keys={keys} alts={alts}");
    }
}
