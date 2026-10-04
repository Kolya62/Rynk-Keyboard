//! Korean Hangul Syllable Composition Engine (2-Set / 두벌식)
//!
//! Handles real-time Hangul syllable composition from Jamo keystrokes,
//! compound vowels (e.g. ㅗ + ㅏ = ㅘ), compound final consonants (e.g. ㅂ + ㅅ = ㅄ),
//! syllable splitting when vowels follow final consonants (e.g. 밥 + ㅏ = 바 + 바),
//! and step-by-step syllable decomposition on Backspace.

// Unicode ranges:
// Hangul Compatibility Jamo: 0x3131..=0x3163
// Hangul Syllables: 0xAC00..=0xD7A3

const HANGUL_BASE: u32 = 0xAC00;
const NUM_JUNGS: u32 = 21;
const NUM_JONGS: u32 = 28;

// Choseong table (19)
const CHOSEONG: [char; 19] = [
    'ㄱ', 'ㄲ', 'ㄴ', 'ㄷ', 'ㄸ', 'ㄹ', 'ㅁ', 'ㅂ', 'ㅃ', 'ㅅ',
    'ㅆ', 'ㅇ', 'ㅈ', 'ㅉ', 'ㅊ', 'ㅋ', 'ㅌ', 'ㅍ', 'ㅎ',
];

// Jungseong table (21)
const JUNGSEONG: [char; 21] = [
    'ㅏ', 'ㅐ', 'ㅑ', 'ㅒ', 'ㅓ', 'ㅔ', 'ㅕ', 'ㅖ', 'ㅗ', 'ㅘ',
    'ㅙ', 'ㅚ', 'ㅛ', 'ㅜ', 'ㅝ', 'ㅞ', 'ㅟ', 'ㅠ', 'ㅡ', 'ㅢ', 'ㅣ',
];

// Jongseong table (28) - index 0 is None
const JONGSEONG: [Option<char>; 28] = [
    None,
    Some('ㄱ'), Some('ㄲ'), Some('ㄳ'), Some('ㄴ'), Some('ㄵ'), Some('ㄶ'),
    Some('ㄷ'), Some('ㄹ'), Some('ㄺ'), Some('ㄻ'), Some('ㄼ'), Some('ㄽ'),
    Some('ㄾ'), Some('ㄿ'), Some('ㅀ'), Some('ㅁ'), Some('ㅂ'), Some('ㅄ'),
    Some('ㅅ'), Some('ㅆ'), Some('ㅇ'), Some('ㅈ'), Some('ㅊ'), Some('ㅋ'),
    Some('ㅌ'), Some('ㅍ'), Some('ㅎ'),
];

fn choseong_index(ch: char) -> Option<usize> {
    CHOSEONG.iter().position(|&c| c == ch)
}

fn jungseong_index(ch: char) -> Option<usize> {
    JUNGSEONG.iter().position(|&c| c == ch)
}

fn jongseong_index(ch: char) -> Option<usize> {
    for (i, opt) in JONGSEONG.iter().enumerate() {
        if let Some(c) = opt {
            if *c == ch {
                return Some(i);
            }
        }
    }
    None
}

/// Combines two vowels into a compound vowel if valid
fn combine_vowels(v1: char, v2: char) -> Option<char> {
    match (v1, v2) {
        ('ㅗ', 'ㅏ') => Some('ㅘ'),
        ('ㅗ', 'ㅐ') => Some('ㅙ'),
        ('ㅗ', 'ㅣ') => Some('ㅚ'),
        ('ㅜ', 'ㅓ') => Some('ㅝ'),
        ('ㅜ', 'ㅔ') => Some('ㅞ'),
        ('ㅜ', 'ㅣ') => Some('ㅟ'),
        ('ㅡ', 'ㅣ') => Some('ㅢ'),
        _ => None,
    }
}

/// Decomposes a compound vowel into (first, second)
fn decompose_vowel(v: char) -> Option<(char, char)> {
    match v {
        'ㅘ' => Some(('ㅗ', 'ㅏ')),
        'ㅙ' => Some(('ㅗ', 'ㅐ')),
        'ㅚ' => Some(('ㅗ', 'ㅣ')),
        'ㅝ' => Some(('ㅜ', 'ㅓ')),
        'ㅞ' => Some(('ㅜ', 'ㅔ')),
        'ㅟ' => Some(('ㅜ', 'ㅣ')),
        'ㅢ' => Some(('ㅡ', 'ㅣ')),
        _ => None,
    }
}

/// Combines two consonants into a compound Jongseong
fn combine_jongseong(j1: char, j2: char) -> Option<char> {
    match (j1, j2) {
        ('ㄱ', 'ㅅ') => Some('ㄳ'),
        ('ㄴ', 'ㅈ') => Some('ㄵ'),
        ('ㄴ', 'ㅎ') => Some('ㄶ'),
        ('ㄹ', 'ㄱ') => Some('ㄺ'),
        ('ㄹ', 'ㅁ') => Some('ㄻ'),
        ('ㄹ', 'ㅂ') => Some('ㄼ'),
        ('ㄹ', 'ㅅ') => Some('ㄽ'),
        ('ㄹ', 'ㅌ') => Some('ㄾ'),
        ('ㄹ', 'ㅍ') => Some('ㄿ'),
        ('ㄹ', 'ㅎ') => Some('ㅀ'),
        ('ㅂ', 'ㅅ') => Some('ㅄ'),
        _ => None,
    }
}

/// Decomposes a compound Jongseong into (first, second)
fn decompose_jongseong(j: char) -> Option<(char, char)> {
    match j {
        'ㄳ' => Some(('ㄱ', 'ㅅ')),
        'ㄵ' => Some(('ㄴ', 'ㅈ')),
        'ㄶ' => Some(('ㄴ', 'ㅎ')),
        'ㄺ' => Some(('ㄹ', 'ㄱ')),
        'ㄻ' => Some(('ㄹ', 'ㅁ')),
        'ㄼ' => Some(('ㄹ', 'ㅂ')),
        'ㄽ' => Some(('ㄹ', 'ㅅ')),
        'ㄾ' => Some(('ㄹ', 'ㅌ')),
        'ㄿ' => Some(('ㄹ', 'ㅍ')),
        'ㅀ' => Some(('ㄹ', 'ㅎ')),
        'ㅄ' => Some(('ㅂ', 'ㅅ')),
        _ => None,
    }
}

fn compose_syllable(cho: char, jung: char, jong: Option<char>) -> Option<char> {
    let cho_idx = choseong_index(cho)? as u32;
    let jung_idx = jungseong_index(jung)? as u32;
    let jong_idx = match jong {
        Some(j) => jongseong_index(j)? as u32,
        None => 0,
    };

    let code = HANGUL_BASE + (cho_idx * NUM_JUNGS + jung_idx) * NUM_JONGS + jong_idx;
    char::from_u32(code)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HangulAction {
    /// Simply append character (e.g. isolated consonant, vowel, or first letter)
    Commit(char),
    /// Replace previous character on screen with composed syllable (delete 1, insert char)
    Replace(char),
    /// Split: replace previous character on screen with first, and commit second
    Split(char, char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HangulBackspaceResult {
    /// Replace previous composed syllable with decomposed state
    Replace(char),
    /// Completely delete the character
    Delete,
    /// Composer had no active state, normal backspace should execute
    None,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum ComposerState {
    #[default]
    Empty,
    Cho(char),
    ChoJung(char, char),
    ChoJungJong(char, char, char),
}

#[derive(Clone, Debug, Default)]
pub struct HangulComposer {
    state: ComposerState,
}

impl HangulComposer {
    pub fn new() -> Self {
        Self {
            state: ComposerState::Empty,
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != ComposerState::Empty
    }

    pub fn reset(&mut self) {
        self.state = ComposerState::Empty;
    }

    pub fn is_hangul_jamo(ch: char) -> bool {
        choseong_index(ch).is_some() || jungseong_index(ch).is_some() || jongseong_index(ch).is_some()
    }

    pub fn feed_jamo(&mut self, jamo: char) -> HangulAction {
        let is_vowel = jungseong_index(jamo).is_some();
        let is_consonant = choseong_index(jamo).is_some();

        match self.state {
            ComposerState::Empty => {
                if is_consonant {
                    self.state = ComposerState::Cho(jamo);
                    HangulAction::Commit(jamo)
                } else {
                    // Isolated vowel
                    self.state = ComposerState::Empty;
                    HangulAction::Commit(jamo)
                }
            }

            ComposerState::Cho(c) => {
                if is_vowel {
                    if let Some(syl) = compose_syllable(c, jamo, None) {
                        self.state = ComposerState::ChoJung(c, jamo);
                        HangulAction::Replace(syl)
                    } else {
                        self.state = ComposerState::Empty;
                        HangulAction::Commit(jamo)
                    }
                } else {
                    // Consonant after consonant: commit previous, start new
                    self.state = ComposerState::Cho(jamo);
                    HangulAction::Commit(jamo)
                }
            }

            ComposerState::ChoJung(c, v) => {
                if is_vowel {
                    // Check if vowels combine into compound vowel
                    if let Some(comp_v) = combine_vowels(v, jamo) {
                        if let Some(syl) = compose_syllable(c, comp_v, None) {
                            self.state = ComposerState::ChoJung(c, comp_v);
                            return HangulAction::Replace(syl);
                        }
                    }
                    // Cannot combine vowels: commit current syllable, start isolated vowel
                    self.state = ComposerState::Empty;
                    HangulAction::Commit(jamo)
                } else if is_consonant {
                    // Check if consonant can be Jongseong
                    if jongseong_index(jamo).is_some() {
                        if let Some(syl) = compose_syllable(c, v, Some(jamo)) {
                            self.state = ComposerState::ChoJungJong(c, v, jamo);
                            return HangulAction::Replace(syl);
                        }
                    }
                    // Cannot be Jongseong: start new syllable
                    self.state = ComposerState::Cho(jamo);
                    HangulAction::Commit(jamo)
                } else {
                    self.state = ComposerState::Empty;
                    HangulAction::Commit(jamo)
                }
            }

            ComposerState::ChoJungJong(c, v, j) => {
                if is_vowel {
                    // Vowel after final consonant: final consonant moves to become initial of next syllable!
                    if let Some((first_j, second_j)) = decompose_jongseong(j) {
                        // Compound Jongseong: first remains as Jongseong, second becomes Choseong
                        let prev_syl = compose_syllable(c, v, Some(first_j)).unwrap();
                        let next_syl = compose_syllable(second_j, jamo, None).unwrap();
                        self.state = ComposerState::ChoJung(second_j, jamo);
                        HangulAction::Split(prev_syl, next_syl)
                    } else {
                        // Single Jongseong: current syllable loses Jongseong, j becomes Choseong
                        let prev_syl = compose_syllable(c, v, None).unwrap();
                        let next_syl = compose_syllable(j, jamo, None).unwrap();
                        self.state = ComposerState::ChoJung(j, jamo);
                        HangulAction::Split(prev_syl, next_syl)
                    }
                } else if is_consonant {
                    // Try to form compound Jongseong
                    if let Some(comp_j) = combine_jongseong(j, jamo) {
                        if let Some(syl) = compose_syllable(c, v, Some(comp_j)) {
                            self.state = ComposerState::ChoJungJong(c, v, comp_j);
                            return HangulAction::Replace(syl);
                        }
                    }
                    // Cannot combine: start new Choseong
                    self.state = ComposerState::Cho(jamo);
                    HangulAction::Commit(jamo)
                } else {
                    self.state = ComposerState::Empty;
                    HangulAction::Commit(jamo)
                }
            }
        }
    }

    pub fn feed_backspace(&mut self) -> HangulBackspaceResult {
        match self.state {
            ComposerState::Empty => HangulBackspaceResult::None,

            ComposerState::Cho(_) => {
                self.state = ComposerState::Empty;
                HangulBackspaceResult::Delete
            }

            ComposerState::ChoJung(c, v) => {
                if let Some((first_v, _)) = decompose_vowel(v) {
                    // Decompose compound vowel back to single vowel
                    let syl = compose_syllable(c, first_v, None).unwrap();
                    self.state = ComposerState::ChoJung(c, first_v);
                    HangulBackspaceResult::Replace(syl)
                } else {
                    // Revert back to Choseong
                    self.state = ComposerState::Cho(c);
                    HangulBackspaceResult::Replace(c)
                }
            }

            ComposerState::ChoJungJong(c, v, j) => {
                if let Some((first_j, _)) = decompose_jongseong(j) {
                    // Decompose compound Jongseong back to first Jongseong
                    let syl = compose_syllable(c, v, Some(first_j)).unwrap();
                    self.state = ComposerState::ChoJungJong(c, v, first_j);
                    HangulBackspaceResult::Replace(syl)
                } else {
                    // Revert to syllable without Jongseong
                    let syl = compose_syllable(c, v, None).unwrap();
                    self.state = ComposerState::ChoJung(c, v);
                    HangulBackspaceResult::Replace(syl)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hangul_composition_basic() {
        let mut composer = HangulComposer::new();

        // 1. 'ㅎ' -> Commit('ㅎ')
        assert_eq!(composer.feed_jamo('ㅎ'), HangulAction::Commit('ㅎ'));

        // 2. 'ㅏ' -> Replace('하')
        assert_eq!(composer.feed_jamo('ㅏ'), HangulAction::Replace('하'));

        // 3. 'ㄴ' -> Replace('한')
        assert_eq!(composer.feed_jamo('ㄴ'), HangulAction::Replace('한'));

        // 4. 'ㄱ' -> Commit('ㄱ')
        assert_eq!(composer.feed_jamo('ㄱ'), HangulAction::Commit('ㄱ'));

        // 5. 'ㅡ' -> Replace('그')
        assert_eq!(composer.feed_jamo('ㅡ'), HangulAction::Replace('그'));

        // 6. 'ㄹ' -> Replace('글')
        assert_eq!(composer.feed_jamo('ㄹ'), HangulAction::Replace('글'));
    }

    #[test]
    fn test_hangul_split_on_vowel() {
        let mut composer = HangulComposer::new();

        // Type 'ㅂ', 'ㅏ', 'ㅂ' -> '밥'
        composer.feed_jamo('ㅂ');
        composer.feed_jamo('ㅏ');
        assert_eq!(composer.feed_jamo('ㅂ'), HangulAction::Replace('밥'));

        // Type 'ㅏ' -> Splits '밥' + 'ㅏ' into '바' + '바'
        assert_eq!(composer.feed_jamo('ㅏ'), HangulAction::Split('바', '바'));
    }

    #[test]
    fn test_hangul_backspace_decomposition() {
        let mut composer = HangulComposer::new();

        // Compose '한'
        composer.feed_jamo('ㅎ');
        composer.feed_jamo('ㅏ');
        composer.feed_jamo('ㄴ');

        // Backspace once -> '하'
        assert_eq!(composer.feed_backspace(), HangulBackspaceResult::Replace('하'));

        // Backspace twice -> 'ㅎ'
        assert_eq!(composer.feed_backspace(), HangulBackspaceResult::Replace('ㅎ'));

        // Backspace third time -> Delete
        assert_eq!(composer.feed_backspace(), HangulBackspaceResult::Delete);

        // Fourth time -> None (empty)
        assert_eq!(composer.feed_backspace(), HangulBackspaceResult::None);
    }

    #[test]
    fn test_compound_jongseong() {
        let mut composer = HangulComposer::new();

        // Compose '닭' ('ㄷ' + 'ㅏ' + 'ㄹ' + 'ㄱ')
        composer.feed_jamo('ㄷ');
        composer.feed_jamo('ㅏ');
        assert_eq!(composer.feed_jamo('ㄹ'), HangulAction::Replace('달'));
        assert_eq!(composer.feed_jamo('ㄱ'), HangulAction::Replace('닭'));

        // Backspace decomposes '닭' -> '달'
        assert_eq!(composer.feed_backspace(), HangulBackspaceResult::Replace('달'));
    }
}
