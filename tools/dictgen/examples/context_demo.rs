//! Shows suggestions for text typed in context:
//! `cargo run --release -p dictgen --example context_demo -- ru "я живу в мос"`
use rynk_core::keyboard::state::Language;
use rynk_core::keyboard::KeyboardEngine;

fn main() {
    let mut args = std::env::args().skip(1);
    let lang = Language::from_code(&args.next().expect("language code")).expect("known language");
    let mut engine = KeyboardEngine::new(1080.0, 800.0, 2.75);
    engine.set_enabled_languages(vec![lang]);
    engine.set_language(lang);
    for text in args {
        engine.set_editor_context(&text, false);
        engine.update_suggestions();
        println!("{text:?} -> {:?}", engine.cached_suggestions);
    }
}
