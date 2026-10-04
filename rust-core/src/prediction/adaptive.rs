//! Words and word pairs learned from what the user types, stored on the device only.
//!
//! File format (`adaptive_dict.bin`):
//! ```text
//! "RYNK" | u16 version | u32 adler32(payload) | payload
//! v2 payload: u32 n, n × [u16 len, utf8, u32 freq, u8 language id (255 = unknown)]
//!             u32 n, n × [u16 len, utf8 key, u16 k, k × [u16 len, utf8, u32 count]]
//! v1 payload: same without language ids and counts (bigram lists were most-recent-first)
//! ```

use super::lm_data::adler32;
use crate::keyboard::state::Language;
use std::collections::HashMap;

const MAX_LEARNED_WORDS: usize = 2000;
const MAX_LEARNED_BIGRAM_KEYS: usize = 1500;
const MAX_NEXT_WORDS: usize = 8;
const FORMAT_VERSION: u16 = 2;
const UNKNOWN_LANGUAGE: u8 = u8::MAX;

#[derive(Clone, Debug)]
pub struct AdaptiveDictionary {
    /// Learned word -> ranking frequency on the 0..=2500 scale
    pub learned_words: HashMap<String, u32>,
    /// Language a learned word was typed in
    pub word_languages: HashMap<String, Language>,
    /// Word -> following words with counts, most frequent first
    pub learned_bigrams: HashMap<String, Vec<(String, u32)>>,
    pub enabled: bool,
    pub is_dirty: bool,
}

impl Default for AdaptiveDictionary {
    fn default() -> Self {
        Self {
            learned_words: HashMap::new(),
            word_languages: HashMap::new(),
            learned_bigrams: HashMap::new(),
            enabled: true,
            is_dirty: false,
        }
    }
}

impl AdaptiveDictionary {
    pub fn learn_word(&mut self, word: &str) {
        self.learn_word_in(word, None);
    }

    pub fn learn_word_in(&mut self, word: &str, lang: Option<Language>) {
        if !self.enabled {
            return;
        }
        let trimmed = word.trim().to_lowercase();
        if trimmed.chars().count() < 2 {
            return;
        }
        if self.learned_words.len() >= MAX_LEARNED_WORDS && !self.learned_words.contains_key(&trimmed) {
            self.decay_and_prune();
        }
        if let Some(lang) = lang {
            self.word_languages.insert(trimmed.clone(), lang);
        }
        let entry = self.learned_words.entry(trimmed).or_insert(100);
        *entry = (*entry + 25).min(2500);
        self.is_dirty = true;
    }

    pub fn learn_bigram(&mut self, w1: &str, w2: &str) {
        if !self.enabled {
            return;
        }
        let k = w1.trim().to_lowercase();
        let v = w2.trim().to_lowercase();
        if k.is_empty() || v.is_empty() {
            return;
        }
        if self.learned_bigrams.len() >= MAX_LEARNED_BIGRAM_KEYS && !self.learned_bigrams.contains_key(&k) {
            self.decay_bigrams();
        }
        let list = self.learned_bigrams.entry(k).or_default();
        let pos = match list.iter().position(|(w, _)| *w == v) {
            Some(p) => {
                list[p].1 = list[p].1.saturating_add(1);
                p
            }
            None => {
                if list.len() >= MAX_NEXT_WORDS {
                    list.pop();
                }
                list.push((v, 1));
                list.len() - 1
            }
        };
        // Keep most frequent first; among equal counts the most recent wins
        let mut i = pos;
        while i > 0 && list[i - 1].1 <= list[i].1 {
            list.swap(i - 1, i);
            i -= 1;
        }
        self.is_dirty = true;
    }

    /// Learned continuations of `word` with their share of the word's learned continuations.
    pub fn next_words(&self, word: &str) -> Vec<(&str, f32)> {
        let Some(list) = self.learned_bigrams.get(&word.trim().to_lowercase()) else {
            return Vec::new();
        };
        let total: u32 = list.iter().map(|(_, c)| c).sum();
        list.iter()
            .map(|(w, c)| (w.as_str(), *c as f32 / total.max(1) as f32))
            .collect()
    }

    /// Total count of learned continuations of `word`.
    pub fn continuation_total(&self, word: &str) -> u32 {
        self.learned_bigrams
            .get(&word.trim().to_lowercase())
            .map(|l| l.iter().map(|(_, c)| c).sum())
            .unwrap_or(0)
    }

    /// How often `next` was typed after `word`, if ever.
    pub fn bigram_count(&self, word: &str, next: &str) -> Option<u32> {
        let next = next.to_lowercase();
        self.learned_bigrams
            .get(&word.trim().to_lowercase())?
            .iter()
            .find(|(w, _)| *w == next)
            .map(|(_, c)| *c)
    }

    pub fn remove_word(&mut self, word: &str) {
        let w = word.trim().to_lowercase();
        self.learned_words.remove(&w);
        self.word_languages.remove(&w);
        self.learned_bigrams.remove(&w);
        for list in self.learned_bigrams.values_mut() {
            list.retain(|(n, _)| *n != w);
        }
        self.is_dirty = true;
    }

    pub fn clear(&mut self) {
        self.learned_words.clear();
        self.word_languages.clear();
        self.learned_bigrams.clear();
        self.is_dirty = true;
    }

    fn decay_and_prune(&mut self) {
        let mut sorted: Vec<(String, u32)> = self.learned_words.drain().collect();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.1));
        let keep_count = (MAX_LEARNED_WORDS * 9) / 10;
        for (w, freq) in sorted.into_iter().take(keep_count) {
            self.learned_words.insert(w, ((freq * 9) / 10).max(50));
        }
        let words = &self.learned_words;
        self.word_languages.retain(|w, _| words.contains_key(w));
    }

    /// Halves all pair counts, forgetting pairs seen once; if nothing was freed, drops the
    /// least used key.
    fn decay_bigrams(&mut self) {
        for list in self.learned_bigrams.values_mut() {
            for e in list.iter_mut() {
                e.1 /= 2;
            }
            list.retain(|e| e.1 > 0);
        }
        self.learned_bigrams.retain(|_, l| !l.is_empty());
        if self.learned_bigrams.len() >= MAX_LEARNED_BIGRAM_KEYS {
            if let Some(k) = self
                .learned_bigrams
                .iter()
                .min_by_key(|(_, l)| l.iter().map(|e| e.1).sum::<u32>())
                .map(|(k, _)| k.clone())
            {
                self.learned_bigrams.remove(&k);
            }
        }
    }

    pub fn serialize_binary(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(1024);
        payload.extend_from_slice(&(self.learned_words.len() as u32).to_le_bytes());
        for (w, freq) in &self.learned_words {
            put_str(&mut payload, w);
            payload.extend_from_slice(&freq.to_le_bytes());
            let lang = self.word_languages.get(w).map(|l| l.to_id() as u8).unwrap_or(UNKNOWN_LANGUAGE);
            payload.push(lang);
        }
        payload.extend_from_slice(&(self.learned_bigrams.len() as u32).to_le_bytes());
        for (k, list) in &self.learned_bigrams {
            put_str(&mut payload, k);
            payload.extend_from_slice(&(list.len() as u16).to_le_bytes());
            for (v, count) in list {
                put_str(&mut payload, v);
                payload.extend_from_slice(&count.to_le_bytes());
            }
        }

        let mut buf = Vec::with_capacity(payload.len() + 10);
        buf.extend_from_slice(b"RYNK");
        buf.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        buf.extend_from_slice(&adler32(&payload).to_le_bytes());
        buf.extend_from_slice(&payload);
        buf
    }

    /// Replaces the contents with a saved state (v1 or v2). Returns false and keeps the current
    /// contents if the data is corrupt.
    pub fn deserialize_binary(&mut self, data: &[u8]) -> bool {
        if data.len() < 10 || &data[0..4] != b"RYNK" {
            return false;
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        let checksum = u32::from_le_bytes([data[6], data[7], data[8], data[9]]);
        let payload = &data[10..];
        if adler32(payload) != checksum || !(1..=FORMAT_VERSION).contains(&version) {
            return false;
        }
        match parse_payload(payload, version) {
            Some((words, langs, bigrams)) => {
                self.learned_words = words;
                self.word_languages = langs;
                self.learned_bigrams = bigrams;
                // A migrated v1 file is rewritten in the current format on the next save
                self.is_dirty = version != FORMAT_VERSION;
                true
            }
            None => false,
        }
    }
}

type Parsed = (HashMap<String, u32>, HashMap<String, Language>, HashMap<String, Vec<(String, u32)>>);

fn parse_payload(payload: &[u8], version: u16) -> Option<Parsed> {
    let mut r = Reader { data: payload, pos: 0 };
    let n_words = r.u32()? as usize;
    let mut words = HashMap::with_capacity(n_words.min(MAX_LEARNED_WORDS));
    let mut langs = HashMap::new();
    for _ in 0..n_words {
        let w = r.string()?;
        let freq = r.u32()?;
        if version >= 2 {
            let lang = r.u8()?;
            if lang != UNKNOWN_LANGUAGE {
                langs.insert(w.clone(), Language::from_id(lang as i32));
            }
        }
        if words.len() < MAX_LEARNED_WORDS {
            words.insert(w, freq);
        }
    }

    let n_keys = r.u32()? as usize;
    let mut bigrams = HashMap::with_capacity(n_keys.min(MAX_LEARNED_BIGRAM_KEYS));
    for _ in 0..n_keys {
        let k = r.string()?;
        let n = r.u16()? as usize;
        let mut list = Vec::with_capacity(n.min(MAX_NEXT_WORDS));
        for i in 0..n {
            let v = r.string()?;
            // v1 lists were ordered most recent first; keep that order as descending counts
            let count = if version >= 2 { r.u32()? } else { (n - i) as u32 };
            if list.len() < MAX_NEXT_WORDS {
                list.push((v, count));
            }
        }
        list.sort_by_key(|e| std::cmp::Reverse(e.1));
        if bigrams.len() < MAX_LEARNED_BIGRAM_KEYS && !list.is_empty() {
            bigrams.insert(k, list);
        }
    }
    Some((words, langs, bigrams))
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    let bytes = &s.as_bytes()[..s.len().min(u16::MAX as usize)];
    out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
    out.extend_from_slice(bytes);
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(n)?;
        let s = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|b| b[0])
    }
    fn u16(&mut self) -> Option<u16> {
        self.take(2).map(|b| u16::from_le_bytes([b[0], b[1]]))
    }
    fn u32(&mut self) -> Option<u32> {
        self.take(4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    fn string(&mut self) -> Option<String> {
        let len = self.u16()? as usize;
        std::str::from_utf8(self.take(len)?).ok().map(str::to_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A v1 file as written by the previous app version
    fn v1_bytes() -> Vec<u8> {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1u32.to_le_bytes());
        put_str(&mut payload, "альтушка");
        payload.extend_from_slice(&150u32.to_le_bytes());
        payload.extend_from_slice(&1u32.to_le_bytes());
        put_str(&mut payload, "привет");
        payload.extend_from_slice(&2u16.to_le_bytes());
        put_str(&mut payload, "мир");
        put_str(&mut payload, "всем");
        let mut buf = b"RYNK".to_vec();
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&adler32(&payload).to_le_bytes());
        buf.extend_from_slice(&payload);
        buf
    }

    #[test]
    fn v1_files_are_migrated() {
        let mut d = AdaptiveDictionary::default();
        assert!(d.deserialize_binary(&v1_bytes()));
        assert_eq!(d.learned_words.get("альтушка"), Some(&150));
        assert_eq!(d.next_words("привет")[0].0, "мир", "most recent v1 entry stays first");
        assert!(d.is_dirty, "migrated data is rewritten as v2");
    }

    #[test]
    fn v2_roundtrip_keeps_counts_and_languages() {
        let mut d = AdaptiveDictionary::default();
        d.learn_word_in("Rynk", Some(Language::English));
        for _ in 0..3 {
            d.learn_bigram("доброе", "утро");
        }
        d.learn_bigram("доброе", "дело");
        let mut restored = AdaptiveDictionary::default();
        assert!(restored.deserialize_binary(&d.serialize_binary()));
        assert_eq!(restored.word_languages.get("rynk"), Some(&Language::English));
        assert_eq!(restored.bigram_count("доброе", "утро"), Some(3));
        let next = restored.next_words("доброе");
        assert_eq!(next[0].0, "утро");
        assert!((next[0].1 - 0.75).abs() < 1e-6);
    }

    #[test]
    fn frequent_pairs_rank_first_and_recent_breaks_ties() {
        let mut d = AdaptiveDictionary::default();
        d.learn_bigram("как", "дела");
        d.learn_bigram("как", "ты");
        assert_eq!(d.next_words("как")[0].0, "ты", "most recent among equals");
        d.learn_bigram("как", "дела");
        assert_eq!(d.next_words("как")[0].0, "дела");
    }

    #[test]
    fn corrupt_data_keeps_current_state() {
        let mut d = AdaptiveDictionary::default();
        d.learn_word("слово");
        let mut bytes = v1_bytes();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        assert!(!d.deserialize_binary(&bytes));
        assert!(d.learned_words.contains_key("слово"));
    }

    #[test]
    fn removing_a_word_forgets_its_pairs() {
        let mut d = AdaptiveDictionary::default();
        d.learn_word("опечатко");
        d.learn_bigram("эта", "опечатко");
        d.remove_word("опечатко");
        assert!(d.learned_words.is_empty());
        assert!(d.next_words("эта").is_empty());
    }
}
