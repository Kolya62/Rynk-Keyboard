//! Compact per-language word list with prefix completion.
//!
//! Replaces the node-per-character trie, whose per-node completion caches do not scale to
//! 100k+ word dictionaries. Words live in one sorted array (by lowercase form); a prefix is a
//! contiguous range found by binary search. Very short prefixes match huge ranges, so their
//! top completions are precomputed.

use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::{Arc, Mutex, MutexGuard};

/// Multiply-rotate hasher (FxHash): the search does tens of map operations per expanded state,
/// and the default SipHash dominated the decoding time.
#[derive(Default, Clone, Copy)]
pub struct FxHasher(u64);

impl Hasher for FxHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.write_u64(b as u64);
        }
    }
    fn write_u64(&mut self, v: u64) {
        self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
    }
    fn write_u32(&mut self, v: u32) {
        self.write_u64(v as u64);
    }
    fn write_usize(&mut self, v: usize) {
        self.write_u64(v as u64);
    }
}

pub type FastMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

/// Child ranges of a lexicon range: (next letter, lo, hi)
pub type ChildRanges = Arc<Vec<(char, u32, u32)>>;

/// Prefixes up to this many chars get precomputed top completions
const PRECOMPUTED_PREFIX_CHARS: usize = 2;
const PRECOMPUTED_TOP: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct LexEntry {
    pub lower: &'static str,
    pub canonical: &'static str,
    pub freq: u32,
    /// Word id in the language model, `u32::MAX` for words without one
    pub model_id: u32,
}

/// Immutable word list built from a language model, shared by every dictionary in the process.
#[derive(Default)]
pub struct LexiconBase {
    /// Sorted by `lower`; one entry per lowercase form
    entries: Vec<LexEntry>,
    top_by_prefix: HashMap<String, Vec<u32>>,
    /// Model word id -> entry index
    by_model_id: Vec<u32>,
    /// Spelling decoder cache, see `decoder::Decoder`
    children_cache: Mutex<FastMap<(u32, u32, u32), ChildRanges>>,
}

/// A shared base word list plus this dictionary's runtime additions.
#[derive(Default)]
pub struct Lexicon {
    base: Arc<LexiconBase>,
    /// Runtime additions and frequency boosts (user / learned words), keyed by lowercase
    overlay: HashMap<String, (String, u32)>,
    /// The overlay as sorted entries for the spelling decoder
    overlay_entries: Vec<LexEntry>,
}

impl LexiconBase {
    /// Builds from `(canonical spelling, freq)` in model-id order. Strings are leaked, so
    /// callers must build each language once per process (see `Dictionary`'s model cache).
    pub fn from_words(words: Vec<(String, u32)>) -> Self {
        let mut canon_blob = String::with_capacity(words.iter().map(|w| w.0.len()).sum());
        let mut lower_blob = String::with_capacity(canon_blob.capacity());
        let mut spans = Vec::with_capacity(words.len());
        for (w, _) in &words {
            let c0 = canon_blob.len();
            canon_blob.push_str(w);
            let l0 = lower_blob.len();
            lower_blob.push_str(&w.to_lowercase());
            spans.push((c0, canon_blob.len(), l0, lower_blob.len()));
        }
        let canon_blob: &'static str = Box::leak(canon_blob.into_boxed_str());
        let lower_blob: &'static str = Box::leak(lower_blob.into_boxed_str());

        let mut entries: Vec<LexEntry> = words
            .iter()
            .zip(&spans)
            .enumerate()
            .map(|(id, ((_, freq), &(c0, c1, l0, l1)))| LexEntry {
                lower: &lower_blob[l0..l1],
                canonical: &canon_blob[c0..c1],
                freq: *freq,
                model_id: id as u32,
            })
            .collect();

        // One entry per lowercase form: keep the most frequent spelling
        entries.sort_by(|a, b| a.lower.cmp(b.lower).then(b.freq.cmp(&a.freq)));
        entries.dedup_by(|later, kept| later.lower == kept.lower);

        let mut by_model_id = vec![u32::MAX; words.len()];
        for (i, e) in entries.iter().enumerate() {
            by_model_id[e.model_id as usize] = i as u32;
        }
        // Spellings that lost the dedup still resolve to their lowercase entry
        for (id, w) in words.iter().enumerate() {
            if by_model_id[id] == u32::MAX {
                let lower = w.0.to_lowercase();
                if let Ok(i) = entries.binary_search_by(|e| e.lower.cmp(lower.as_str())) {
                    by_model_id[id] = i as u32;
                }
            }
        }

        let mut top_by_prefix: HashMap<String, Vec<u32>> = HashMap::new();
        for (i, e) in entries.iter().enumerate() {
            let mut prefix = String::new();
            for ch in e.lower.chars().take(PRECOMPUTED_PREFIX_CHARS) {
                prefix.push(ch);
                top_by_prefix.entry(prefix.clone()).or_default().push(i as u32);
            }
        }
        for list in top_by_prefix.values_mut() {
            list.sort_unstable_by(|&a, &b| entries[b as usize].freq.cmp(&entries[a as usize].freq));
            list.truncate(PRECOMPUTED_TOP);
            list.shrink_to_fit();
        }

        Self {
            entries,
            top_by_prefix,
            by_model_id,
            children_cache: Mutex::default(),
        }
    }

    pub fn entries(&self) -> &[LexEntry] {
        &self.entries
    }

    fn find(&self, lower: &str) -> Option<&LexEntry> {
        self.entries
            .binary_search_by(|e| e.lower.cmp(lower))
            .ok()
            .map(|i| &self.entries[i])
    }
}

impl Lexicon {
    pub fn new(base: Arc<LexiconBase>) -> Self {
        Self {
            base,
            overlay: HashMap::new(),
            overlay_entries: Vec::new(),
        }
    }

    pub fn from_words(words: Vec<(String, u32)>) -> Self {
        Self::new(Arc::new(LexiconBase::from_words(words)))
    }

    pub fn len(&self) -> usize {
        self.base.entries.len()
    }

    /// Decoder cache of the shared base word list (held for one decoding pass).
    pub fn children_cache(&self) -> MutexGuard<'_, FastMap<(u32, u32, u32), ChildRanges>> {
        self.base.children_cache.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn is_empty(&self) -> bool {
        self.base.entries.is_empty() && self.overlay.is_empty()
    }

    pub fn entries(&self) -> &[LexEntry] {
        &self.base.entries
    }

    fn find(&self, lower: &str) -> Option<&LexEntry> {
        self.base.find(lower)
    }

    /// Adds a word or raises its frequency (user dictionary, adaptive learning).
    pub fn insert(&mut self, word: &str, freq: u32) {
        if word.is_empty() {
            return;
        }
        let lower = word.to_lowercase();
        let canonical = self
            .find(&lower)
            .map(|e| e.canonical.to_string())
            .unwrap_or_else(|| word.to_string());
        let is_new = !self.overlay.contains_key(&lower);
        let slot = self.overlay.entry(lower.clone()).or_insert((canonical, 0));
        slot.1 = slot.1.max(freq);
        let (canonical, freq) = (slot.0.clone(), slot.1);
        match self.overlay_entries.binary_search_by(|e| e.lower.cmp(lower.as_str())) {
            Ok(i) => self.overlay_entries[i].freq = freq,
            Err(i) => {
                debug_assert!(is_new);
                // Leaked like the base strings; the overlay is bounded by the user's vocabulary
                self.overlay_entries.insert(
                    i,
                    LexEntry {
                        lower: Box::leak(lower.into_boxed_str()),
                        canonical: Box::leak(canonical.into_boxed_str()),
                        freq,
                        model_id: u32::MAX,
                    },
                );
            }
        }
    }

    /// User and learned words as sorted entries (searched by the decoder next to the base).
    pub fn overlay_entries(&self) -> &[LexEntry] {
        &self.overlay_entries
    }

    pub fn contains(&self, word: &str) -> bool {
        let lower = word.to_lowercase();
        self.overlay.contains_key(&lower) || self.find(&lower).is_some()
    }

    pub fn get_frequency(&self, word: &str) -> Option<u32> {
        let lower = word.to_lowercase();
        let base = self.find(&lower).map(|e| e.freq);
        let over = self.overlay.get(&lower).map(|o| o.1);
        base.max(over)
    }

    /// Canonical spelling of a known word ("москва" -> "Москва").
    pub fn canonical(&self, word: &str) -> Option<&str> {
        let lower = word.to_lowercase();
        if let Some(e) = self.find(&lower) {
            return Some(e.canonical);
        }
        self.overlay.get(&lower).map(|o| o.0.as_str())
    }

    /// Canonical spelling for a model word id.
    pub fn word_by_model_id(&self, id: u32) -> Option<&'static str> {
        let idx = *self.base.by_model_id.get(id as usize)?;
        self.base.entries.get(idx as usize).map(|e| e.canonical)
    }

    /// Model word id for a word, if it is in the base vocabulary.
    pub fn model_id(&self, word: &str) -> Option<u32> {
        self.find(&word.to_lowercase()).map(|e| e.model_id)
    }

    /// Most frequent words starting with `prefix` (case-insensitive), as canonical spellings.
    pub fn find_completions(&self, prefix: &str, limit: usize) -> Vec<(String, u32)> {
        if prefix.is_empty() || limit == 0 {
            return Vec::new();
        }
        let lower = prefix.to_lowercase();
        let mut found: Vec<(&str, &str, u32)> = Vec::with_capacity(limit * 2);

        if lower.chars().count() <= PRECOMPUTED_PREFIX_CHARS {
            if let Some(list) = self.base.top_by_prefix.get(&lower) {
                for &i in list {
                    let e = &self.base.entries[i as usize];
                    found.push((e.lower, e.canonical, e.freq));
                }
            }
        } else {
            let entries = &self.base.entries;
            let start = entries.partition_point(|e| e.lower < lower.as_str());
            for e in entries[start..].iter().take_while(|e| e.lower.starts_with(lower.as_str())) {
                found.push((e.lower, e.canonical, e.freq));
            }
        }

        let mut results: Vec<(String, u32)> = Vec::with_capacity(found.len() + 4);
        let mut seen: Vec<&str> = Vec::with_capacity(found.len());
        for (l, canonical, freq) in found {
            // Overlay boosts take precedence over base frequencies
            let f = self.overlay.get(l).map(|o| o.1.max(freq)).unwrap_or(freq);
            results.push((canonical.to_string(), f));
            seen.push(l);
        }
        // Overlay words: new words, and boosted base words outside a precomputed top list
        for (l, (canonical, f)) in &self.overlay {
            if l.starts_with(lower.as_str()) && !seen.contains(&l.as_str()) {
                let f = self.find(l).map(|e| e.freq.max(*f)).unwrap_or(*f);
                results.push((canonical.clone(), f));
            }
        }
        results.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        results.truncate(limit);
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex() -> Lexicon {
        Lexicon::from_words(vec![
            ("привет".into(), 2400),
            ("приветствую".into(), 900),
            ("при".into(), 2000),
            ("Москва".into(), 1500),
            ("москва".into(), 100),
            ("macOS".into(), 800),
        ])
    }

    #[test]
    fn completions_are_ranked_and_case_insensitive() {
        let l = lex();
        let c = l.find_completions("прив", 5);
        assert_eq!(c[0].0, "привет");
        assert_eq!(c[1].0, "приветствую");
        // Short prefixes use the precomputed table
        assert_eq!(l.find_completions("п", 1)[0].0, "привет");
        assert_eq!(l.find_completions("MAC", 2)[0].0, "macOS");
    }

    #[test]
    fn most_frequent_spelling_wins() {
        let l = lex();
        assert_eq!(l.find_completions("моск", 3), vec![("Москва".to_string(), 1500)]);
        assert_eq!(l.get_frequency("МОСКВА"), Some(1500));
        // Both spellings map to the same entry
        assert_eq!(l.word_by_model_id(4), Some("Москва"));
        assert_eq!(l.model_id("москва"), Some(3));
    }

    #[test]
    fn overlay_adds_and_boosts_words() {
        let mut l = lex();
        assert!(!l.contains("альтушка"));
        l.insert("альтушка", 2500);
        assert!(l.contains("Альтушка"));
        assert_eq!(l.find_completions("альт", 3)[0].0, "альтушка");

        l.insert("приветствую", 2500);
        assert_eq!(l.find_completions("прив", 1)[0].0, "приветствую");
        assert_eq!(l.get_frequency("приветствую"), Some(2500));
    }
}
