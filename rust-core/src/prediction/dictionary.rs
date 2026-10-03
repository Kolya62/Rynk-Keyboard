use super::trie::Trie;
use crate::keyboard::state::Language;
use std::collections::{HashMap, HashSet};

pub static RU_WORDS_RAW: &str = include_str!("data/ru_words.txt");
pub static EN_WORDS_RAW: &str = include_str!("data/en_words.txt");
pub static DE_WORDS_RAW: &str = include_str!("data/de_words.txt");
pub static FR_WORDS_RAW: &str = include_str!("data/fr_words.txt");
pub static ES_WORDS_RAW: &str = include_str!("data/es_words.txt");
pub static PT_WORDS_RAW: &str = include_str!("data/pt_words.txt");
pub static IT_WORDS_RAW: &str = include_str!("data/it_words.txt");
pub static TR_WORDS_RAW: &str = include_str!("data/tr_words.txt");
pub static UK_WORDS_RAW: &str = include_str!("data/uk_words.txt");
pub static BE_WORDS_RAW: &str = include_str!("data/be_words.txt");
pub static KK_WORDS_RAW: &str = include_str!("data/kk_words.txt");
pub static PROFANITY_RAW: &str = include_str!("data/profanity.txt");
pub static RU_BIGRAMS_RAW: &str = include_str!("data/ru_bigrams.txt");
pub static EN_BIGRAMS_RAW: &str = include_str!("data/en_bigrams.txt");

const NUM_LANGUAGES: usize = 11;

const MAX_LEARNED_WORDS: usize = 2000;
const MAX_LEARNED_BIGRAMS: usize = 600;

#[derive(Clone, Debug)]
pub struct AdaptiveDictionary {
    pub learned_words: HashMap<String, u32>,
    pub learned_bigrams: HashMap<String, Vec<String>>,
    pub enabled: bool,
    pub is_dirty: bool,
}

impl Default for AdaptiveDictionary {
    fn default() -> Self {
        Self {
            learned_words: HashMap::new(),
            learned_bigrams: HashMap::new(),
            enabled: true,
            is_dirty: false,
        }
    }
}

impl AdaptiveDictionary {
    pub fn learn_word(&mut self, word: &str) {
        if !self.enabled {
            return;
        }
        let trimmed = word.trim().to_lowercase();
        if trimmed.chars().count() < 2 {
            return;
        }
        if self.learned_words.len() >= MAX_LEARNED_WORDS
            && !self.learned_words.contains_key(&trimmed)
        {
            self.decay_and_prune();
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
        if self.learned_bigrams.len() >= MAX_LEARNED_BIGRAMS
            && !self.learned_bigrams.contains_key(&k)
        {
            if let Some(first_key) = self.learned_bigrams.keys().next().cloned() {
                self.learned_bigrams.remove(&first_key);
            }
        }
        let list = self.learned_bigrams.entry(k).or_default();
        if let Some(pos) = list.iter().position(|x| x == &v) {
            list.remove(pos);
        }
        list.insert(0, v);
        if list.len() > 6 {
            list.pop();
        }
        self.is_dirty = true;
    }

    pub fn clear(&mut self) {
        self.learned_words.clear();
        self.learned_bigrams.clear();
        self.is_dirty = true;
    }

    fn decay_and_prune(&mut self) {
        let mut sorted: Vec<(String, u32)> = self.learned_words.drain().collect();
        sorted.sort_by_key(|a| std::cmp::Reverse(a.1));
        let keep_count = (MAX_LEARNED_WORDS * 9) / 10;
        for (w, mut freq) in sorted.into_iter().take(keep_count) {
            freq = (freq * 9) / 10;
            self.learned_words.insert(w, freq.max(50));
        }
    }

    pub fn serialize_binary(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(1024);
        buf.extend_from_slice(b"RYNK");
        buf.extend_from_slice(&1u16.to_le_bytes());
        buf.extend_from_slice(&(self.learned_words.len() as u32).to_le_bytes());
        for (w, freq) in &self.learned_words {
            let bytes = w.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(bytes);
            buf.extend_from_slice(&freq.to_le_bytes());
        }
        buf.extend_from_slice(&(self.learned_bigrams.len() as u32).to_le_bytes());
        for (k, list) in &self.learned_bigrams {
            let k_bytes = k.as_bytes();
            buf.extend_from_slice(&(k_bytes.len() as u16).to_le_bytes());
            buf.extend_from_slice(k_bytes);
            buf.extend_from_slice(&(list.len() as u16).to_le_bytes());
            for v in list {
                let v_bytes = v.as_bytes();
                buf.extend_from_slice(&(v_bytes.len() as u16).to_le_bytes());
                buf.extend_from_slice(v_bytes);
            }
        }
        buf
    }

    pub fn deserialize_binary(&mut self, data: &[u8]) -> bool {
        if data.len() < 10 || &data[0..4] != b"RYNK" {
            return false;
        }
        let mut offset = 4;
        let _version = u16::from_le_bytes([data[offset], data[offset + 1]]);
        offset += 2;

        if offset + 4 > data.len() {
            return false;
        }
        let word_count = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        self.learned_words.clear();
        for _ in 0..word_count.min(MAX_LEARNED_WORDS) {
            if offset + 2 > data.len() {
                break;
            }
            let w_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
            offset += 2;
            if offset + w_len + 4 > data.len() {
                break;
            }
            if let Ok(w) = std::str::from_utf8(&data[offset..offset + w_len]) {
                offset += w_len;
                let freq = u32::from_le_bytes([
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    data[offset + 3],
                ]);
                offset += 4;
                self.learned_words.insert(w.to_string(), freq);
            } else {
                offset += w_len + 4;
            }
        }

        if offset + 4 <= data.len() {
            let bigram_count = u32::from_le_bytes([
                data[offset],
                data[offset + 1],
                data[offset + 2],
                data[offset + 3],
            ]) as usize;
            offset += 4;
            self.learned_bigrams.clear();
            for _ in 0..bigram_count.min(MAX_LEARNED_BIGRAMS) {
                if offset + 2 > data.len() {
                    break;
                }
                let k_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
                offset += 2;
                if offset + k_len + 2 > data.len() {
                    break;
                }
                let k_opt = std::str::from_utf8(&data[offset..offset + k_len])
                    .ok()
                    .map(|s| s.to_string());
                offset += k_len;
                let next_count = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
                offset += 2;
                let mut list = Vec::with_capacity(next_count.min(6));
                for _ in 0..next_count {
                    if offset + 2 > data.len() {
                        break;
                    }
                    let v_len = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
                    offset += 2;
                    if offset + v_len > data.len() {
                        break;
                    }
                    if let Ok(v) = std::str::from_utf8(&data[offset..offset + v_len]) {
                        if list.len() < 6 {
                            list.push(v.to_string());
                        }
                    }
                    offset += v_len;
                }
                if let Some(k) = k_opt {
                    self.learned_bigrams.insert(k, list);
                }
            }
        }
        self.is_dirty = false;
        true
    }
}

pub type CandidateBucketMap = HashMap<(char, usize), Vec<(&'static str, u32)>>;

pub struct Dictionary {
    pub tries: [Trie; NUM_LANGUAGES],
    pub word_lists: [Vec<(&'static str, u32)>; NUM_LANGUAGES],
    pub candidate_buckets: [CandidateBucketMap; NUM_LANGUAGES],
    pub ru_bigrams: HashMap<String, Vec<String>>,
    pub en_bigrams: HashMap<String, Vec<String>>,
    pub profanity: HashSet<&'static str>,
    pub profanity_enabled: bool,
    pub user_dict: HashMap<String, u32>,
    pub adaptive_dict: AdaptiveDictionary,
    pub removed_words: HashSet<String>,
}

impl Default for Dictionary {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_words(raw: &'static str, cap: usize) -> Vec<(&'static str, u32)> {
    let mut words = Vec::with_capacity(cap);
    for line in raw.lines() {
        let mut parts = line.split(':');
        if let (Some(w), Some(f_str)) = (parts.next(), parts.next()) {
            if let Ok(freq) = f_str.parse::<u32>() {
                words.push((w.trim(), freq));
            }
        }
    }
    words
}

impl Dictionary {
    pub fn new() -> Self {
        let ru_words = parse_words(RU_WORDS_RAW, 50000);
        let en_words = parse_words(EN_WORDS_RAW, 25000);
        let de_words = parse_words(DE_WORDS_RAW, 2000);
        let fr_words = parse_words(FR_WORDS_RAW, 2000);
        let es_words = parse_words(ES_WORDS_RAW, 2000);
        let pt_words = parse_words(PT_WORDS_RAW, 2000);
        let it_words = parse_words(IT_WORDS_RAW, 2000);
        let tr_words = parse_words(TR_WORDS_RAW, 2000);
        let uk_words = parse_words(UK_WORDS_RAW, 2000);
        let be_words = parse_words(BE_WORDS_RAW, 2000);
        let kk_words = parse_words(KK_WORDS_RAW, 2000);

        let mut profanity = HashSet::with_capacity(200);
        for line in PROFANITY_RAW.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                profanity.insert(trimmed);
            }
        }

        let mut ru_bigrams = HashMap::with_capacity(100);
        for line in RU_BIGRAMS_RAW.lines() {
            let mut parts = line.split(':');
            if let (Some(w), Some(nexts)) = (parts.next(), parts.next()) {
                let list: Vec<String> = nexts.split(',').map(|s| s.trim().to_string()).collect();
                ru_bigrams.insert(w.to_string(), list);
            }
        }

        let mut en_bigrams = HashMap::with_capacity(100);
        for line in EN_BIGRAMS_RAW.lines() {
            let mut parts = line.split(':');
            if let (Some(w), Some(nexts)) = (parts.next(), parts.next()) {
                let list: Vec<String> = nexts.split(',').map(|s| s.trim().to_string()).collect();
                en_bigrams.insert(w.to_string(), list);
            }
        }

        let word_lists = [
            ru_words, en_words, de_words, fr_words, es_words, pt_words, it_words, tr_words,
            uk_words, be_words, kk_words,
        ];

        let mut tries: [Trie; NUM_LANGUAGES] = Default::default();
        let mut candidate_buckets: [CandidateBucketMap; NUM_LANGUAGES] = Default::default();

        for (i, list) in word_lists.iter().enumerate() {
            for &(w, freq) in list {
                tries[i].insert(w, freq);
                let c0 = w
                    .chars()
                    .next()
                    .unwrap_or('\0')
                    .to_lowercase()
                    .next()
                    .unwrap_or('\0');
                let len = w.chars().count();
                candidate_buckets[i]
                    .entry((c0, len))
                    .or_default()
                    .push((w, freq));
            }
        }

        Self {
            tries,
            word_lists,
            candidate_buckets,
            ru_bigrams,
            en_bigrams,
            profanity,
            profanity_enabled: false,
            user_dict: HashMap::new(),
            adaptive_dict: AdaptiveDictionary::default(),
            removed_words: HashSet::new(),
        }
    }

    pub fn set_profanity_enabled(&mut self, enabled: bool) {
        self.profanity_enabled = enabled;
    }

    pub fn get_trie(&self, lang: Language) -> &Trie {
        let idx = (lang.to_id() as usize).min(NUM_LANGUAGES - 1);
        &self.tries[idx]
    }

    pub fn get_word_list(&self, lang: Language) -> &[(&'static str, u32)] {
        let idx = (lang.to_id() as usize).min(NUM_LANGUAGES - 1);
        &self.word_lists[idx]
    }

    pub fn get_fuzzy_candidates(&self, query: &str, lang: Language) -> Vec<(&'static str, u32)> {
        let clean = query.trim().to_lowercase();
        let query_len = clean.chars().count();
        if query_len == 0 {
            return Vec::new();
        }

        let idx = (lang.to_id() as usize).min(NUM_LANGUAGES - 1);
        let buckets = &self.candidate_buckets[idx];

        let min_len = query_len.saturating_sub(2).max(1);
        let max_len = query_len + 2;

        let mut chars = clean.chars();
        let c0 = chars.next();
        let c1 = chars.next();

        let mut candidates = Vec::with_capacity(128);
        let mut seen = HashSet::with_capacity(128);

        for len in min_len..=max_len {
            if let Some(c) = c0 {
                if let Some(list) = buckets.get(&(c, len)) {
                    for &(w, freq) in list {
                        if seen.insert(w) {
                            candidates.push((w, freq));
                        }
                    }
                }
            }
            if let Some(c) = c1 {
                if let Some(list) = buckets.get(&(c, len)) {
                    for &(w, freq) in list {
                        if seen.insert(w) {
                            candidates.push((w, freq));
                        }
                    }
                }
            }
        }

        candidates
    }

    pub fn contains_word(&self, word: &str, _is_ru: bool) -> bool {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() || self.removed_words.contains(&clean) {
            return false;
        }
        if self.user_dict.contains_key(&clean)
            || self.adaptive_dict.learned_words.contains_key(&clean)
        {
            return true;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return true;
        }
        for trie in &self.tries {
            if trie.contains(&clean) {
                return true;
            }
        }
        false
    }

    pub fn contains_word_for_lang(&self, word: &str, lang: Language) -> bool {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() || self.removed_words.contains(&clean) {
            return false;
        }
        if self.user_dict.contains_key(&clean)
            || self.adaptive_dict.learned_words.contains_key(&clean)
        {
            return true;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return true;
        }
        self.get_trie(lang).contains(&clean)
    }

    pub fn get_word_frequency(&self, word: &str, is_ru: bool) -> u32 {
        let lang = if is_ru {
            Language::Russian
        } else {
            Language::English
        };
        self.get_word_frequency_for_lang(word, lang)
    }

    pub fn get_word_frequency_for_lang(&self, word: &str, lang: Language) -> u32 {
        let clean = word.trim().to_lowercase();
        if self.removed_words.contains(&clean) {
            return 0;
        }
        if let Some(&f) = self.user_dict.get(&clean) {
            return f;
        }
        if let Some(&f) = self.adaptive_dict.learned_words.get(&clean) {
            return f;
        }
        if self.profanity_enabled && self.profanity.contains(clean.as_str()) {
            return 1000;
        }
        if let Some(f) = self.get_trie(lang).get_frequency(&clean) {
            return f;
        }
        // Fallback check in Russian or English trie
        if let Some(f) = self.tries[0].get_frequency(&clean) {
            return f;
        }
        if let Some(f) = self.tries[1].get_frequency(&clean) {
            return f;
        }
        0
    }

    pub fn add_user_word(&mut self, word: &str, is_ru: bool) {
        let clean = word.trim().to_lowercase();
        if clean.is_empty() {
            return;
        }
        self.removed_words.remove(&clean);
        self.user_dict.insert(clean.clone(), 2500);
        let lang_idx = if is_ru { 0 } else { 1 };
        self.tries[lang_idx].insert(&clean, 2500);
    }

    pub fn remove_user_word(&mut self, word: &str) {
        let clean = word.trim().to_lowercase();
        self.user_dict.remove(&clean);
        self.adaptive_dict.learned_words.remove(&clean);
        self.adaptive_dict.is_dirty = true;
        self.removed_words.insert(clean);
    }

    pub fn get_user_words(&self) -> Vec<String> {
        let mut words: Vec<String> = self.user_dict.keys().cloned().collect();
        words.sort();
        words
    }

    pub fn learn_word(&mut self, word: &str, is_ru: bool) {
        let trimmed = word.trim().to_lowercase();
        if trimmed.chars().count() < 2 || self.removed_words.contains(&trimmed) {
            return;
        }

        self.adaptive_dict.learn_word(&trimmed);
        if let Some(&freq) = self.adaptive_dict.learned_words.get(&trimmed) {
            let lang_idx = if is_ru { 0 } else { 1 };
            self.tries[lang_idx].insert(&trimmed, freq);
        }
    }

    pub fn learn_bigram(&mut self, w1: &str, w2: &str) {
        self.adaptive_dict.learn_bigram(w1, w2);
    }

    pub fn get_context_predictions(&self, last_word: &str, lang: Language) -> Vec<String> {
        let k = last_word.trim().to_lowercase();
        if k.is_empty() {
            return Vec::new();
        }

        let mut res = Vec::with_capacity(6);

        // 1. Check user learned bigrams first
        if let Some(user_nexts) = self.adaptive_dict.learned_bigrams.get(&k) {
            for w in user_nexts {
                if !res.contains(w) && !self.removed_words.contains(w) {
                    res.push(w.clone());
                }
            }
        }

        // 2. Builtin bigrams (Russian or English)
        let bigram_map = if lang == Language::Russian {
            &self.ru_bigrams
        } else {
            &self.en_bigrams
        };
        if let Some(nexts) = bigram_map.get(&k) {
            for w in nexts {
                if !res.contains(w) && !self.removed_words.contains(w) {
                    res.push(w.clone());
                }
            }
        }

        res
    }
}
