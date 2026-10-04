//! Parses the text before the cursor into what the prediction engine needs: the word being
//! typed (if the cursor sits right after letters) and up to two preceding words of the same
//! sentence.

/// Characters that end a sentence for context purposes
fn is_sentence_end(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '…' | '\n' | '¡' | '¿' | '。' | '！' | '？' | '؟' | '।')
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || is_mark(c)
}

/// Combining marks are part of words in Indic and other scripts
fn is_mark(c: char) -> bool {
    matches!(c as u32, 0x0300..=0x036F | 0x0900..=0x0DFF | 0x0E31..=0x0E4E | 0x1000..=0x109F | 0x17B4..=0x17D3)
        && !c.is_alphanumeric()
}

/// Inner apostrophes and hyphens join word parts: "don't", "по-моему"
fn is_joiner(c: char) -> bool {
    matches!(c, '\'' | '’' | '-')
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditorContext {
    /// Letters directly before the cursor (the word being typed), empty after a space
    pub partial_word: String,
    /// Word before the current one
    pub prev1: Option<String>,
    /// Word before `prev1`
    pub prev2: Option<String>,
    /// The oldest word in the context window starts a sentence (or the window is empty and the
    /// current word starts one)
    pub sentence_start: bool,
    /// The text ends with whitespace
    pub ends_with_space: bool,
}

impl EditorContext {
    pub fn parse(text_before_cursor: &str) -> Self {
        let chars: Vec<char> = text_before_cursor.chars().collect();
        let mut i = chars.len();

        let partial_end = i;
        i = word_start(&chars, i);
        let partial_word: String = chars[i..partial_end].iter().collect();

        let mut words: Vec<String> = Vec::with_capacity(2);
        let mut sentence_start = false;
        loop {
            // Skip separators (spaces, commas, quotes) until a word or a sentence boundary
            while i > 0 && !is_word_char(chars[i - 1]) {
                if is_sentence_end(chars[i - 1]) {
                    sentence_start = true;
                    break;
                }
                i -= 1;
            }
            if sentence_start || i == 0 {
                // Start of text counts as a sentence start
                sentence_start = true;
                break;
            }
            if words.len() == 2 {
                break;
            }
            let end = i;
            i = word_start(&chars, i);
            words.push(chars[i..end].iter().collect());
        }

        Self {
            partial_word,
            prev1: words.first().cloned(),
            prev2: words.get(1).cloned(),
            sentence_start,
            ends_with_space: chars.last().is_some_and(|c| c.is_whitespace()),
        }
    }
}

/// Start index of the word ending at `end` (exclusive), or `end` if no word ends there.
fn word_start(chars: &[char], end: usize) -> usize {
    let mut i = end;
    while i > 0 {
        let c = chars[i - 1];
        let inner_joiner = is_joiner(c) && i < end && i >= 2 && is_word_char(chars[i - 2]);
        if !is_word_char(c) && !inner_joiner {
            break;
        }
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_word_and_previous_words() {
        let c = EditorContext::parse("Я сегодня иду в мага");
        assert_eq!(c.partial_word, "мага");
        assert_eq!(c.prev1.as_deref(), Some("в"));
        assert_eq!(c.prev2.as_deref(), Some("иду"));
        assert!(!c.sentence_start);
        assert!(!c.ends_with_space);
    }

    #[test]
    fn after_space_there_is_no_partial_word() {
        let c = EditorContext::parse("thank ");
        assert_eq!(c.partial_word, "");
        assert_eq!(c.prev1.as_deref(), Some("thank"));
        assert_eq!(c.prev2, None);
        assert!(c.sentence_start, "'thank' starts the text");
        assert!(c.ends_with_space);
    }

    #[test]
    fn sentence_boundary_stops_the_context() {
        let c = EditorContext::parse("Всё хорошо. Как де");
        assert_eq!(c.partial_word, "де");
        assert_eq!(c.prev1.as_deref(), Some("Как"));
        assert_eq!(c.prev2, None);
        assert!(c.sentence_start);

        let c = EditorContext::parse("Done! ");
        assert_eq!(c.prev1, None);
        assert!(c.sentence_start);
    }

    #[test]
    fn punctuation_inside_a_sentence_is_skipped() {
        let c = EditorContext::parse("ну, по-моему, don't ");
        assert_eq!(c.prev1.as_deref(), Some("don't"));
        assert_eq!(c.prev2.as_deref(), Some("по-моему"));
        assert!(!c.sentence_start);
    }

    #[test]
    fn empty_text_is_a_sentence_start() {
        let c = EditorContext::parse("");
        assert_eq!(c, EditorContext { sentence_start: true, ..Default::default() });
    }
}
