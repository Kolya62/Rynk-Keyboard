//! Binary language model file format (`<lang>.rlm`).
//!
//! Produced offline by `tools/dictgen` and loaded lazily per language on the device.
//!
//! ```text
//! header:  "RLM1" | u16 version | u16 reserved | u32 adler32(payload)
//! payload: lexicon | bigram groups | trigram groups
//!
//! lexicon: varint n, then n × [varint len, utf8 bytes, varint count, u16 freq]
//!          ids are positions in this list, ordered by descending frequency
//! groups:  varint n_groups, then per group (sorted by key):
//!          varint key delta, varint k, k × [varint word id, u8 q] (best q first)
//! ```
//!
//! Bigram keys are `ctx1` and trigram keys are `(ctx1 << 32) | ctx2`, where a context id is
//! `word id + 1` and `0` stands for the sentence start. `q` is the conditional probability
//! quantized as `round(-log10(p) * Q_SCALE)`.

use std::collections::HashMap;

pub const MAGIC: &[u8; 4] = b"RLM1";
pub const VERSION: u16 = 1;
/// Context id of the sentence start marker
pub const SENTENCE_START: u32 = 0;
/// Quantization steps per decade of probability
pub const Q_SCALE: f32 = 32.0;

/// Context id for a word id
#[inline]
pub fn ctx_id(word_id: u32) -> u32 {
    word_id + 1
}

#[inline]
pub fn trigram_key(ctx1: u32, ctx2: u32) -> u64 {
    ((ctx1 as u64) << 32) | ctx2 as u64
}

#[inline]
pub fn quantize(p: f64) -> u8 {
    if p <= 0.0 {
        return u8::MAX;
    }
    (-p.log10() * Q_SCALE as f64).round().clamp(0.0, u8::MAX as f64) as u8
}

#[inline]
pub fn dequantize_log10(q: u8) -> f32 {
    -(q as f32) / Q_SCALE
}

#[derive(Clone, Debug, PartialEq)]
pub struct WordEntry {
    /// Canonical spelling, e.g. "macOS", "Москва", "привет"
    pub word: String,
    /// Raw corpus count (pseudo-count for curated words)
    pub count: u32,
    /// Keyboard ranking frequency on the legacy 0..=2500 scale
    pub freq: u16,
}

/// N-gram table: groups of continuations keyed by context, each group sorted best-first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct NgramTable {
    keys: Vec<u64>,
    offsets: Vec<u32>,
    entries: Vec<(u32, u8)>,
}

impl NgramTable {
    /// Builds a table from `(key, word id, q)` triples in any order.
    pub fn from_triples(mut triples: Vec<(u64, u32, u8)>) -> Self {
        triples.sort_unstable_by(|a, b| a.0.cmp(&b.0).then(a.2.cmp(&b.2)).then(a.1.cmp(&b.1)));
        let mut table = Self::default();
        for (key, id, q) in triples {
            if table.keys.last() != Some(&key) {
                table.keys.push(key);
                table.offsets.push(table.entries.len() as u32);
            }
            table.entries.push((id, q));
        }
        table
    }

    /// Continuations of `key`, best (lowest q) first.
    pub fn get(&self, key: u64) -> &[(u32, u8)] {
        match self.keys.binary_search(&key) {
            Ok(i) => {
                let start = self.offsets[i] as usize;
                let end = self
                    .offsets
                    .get(i + 1)
                    .map(|&o| o as usize)
                    .unwrap_or(self.entries.len());
                &self.entries[start..end]
            }
            Err(_) => &[],
        }
    }

    /// Quantized probability of `word_id` after `key`, if present.
    pub fn q(&self, key: u64, word_id: u32) -> Option<u8> {
        self.get(key).iter().find(|e| e.0 == word_id).map(|e| e.1)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn write(&self, out: &mut Vec<u8>) {
        write_varint(out, self.keys.len() as u64);
        let mut prev = 0u64;
        for (i, &key) in self.keys.iter().enumerate() {
            write_varint(out, key - prev);
            prev = key;
            let start = self.offsets[i] as usize;
            let end = self.offsets.get(i + 1).map(|&o| o as usize).unwrap_or(self.entries.len());
            write_varint(out, (end - start) as u64);
            for &(id, q) in &self.entries[start..end] {
                write_varint(out, id as u64);
                out.push(q);
            }
        }
    }

    fn read(r: &mut Reader, n_words: u32) -> Option<Self> {
        let n_groups = r.varint()? as usize;
        let mut table = Self {
            keys: Vec::with_capacity(n_groups),
            offsets: Vec::with_capacity(n_groups),
            entries: Vec::new(),
        };
        let mut key = 0u64;
        for _ in 0..n_groups {
            key = key.checked_add(r.varint()?)?;
            let k = r.varint()? as usize;
            table.keys.push(key);
            table.offsets.push(table.entries.len() as u32);
            for _ in 0..k {
                let id = u32::try_from(r.varint()?).ok()?;
                let q = r.u8()?;
                if id >= n_words {
                    return None;
                }
                table.entries.push((id, q));
            }
        }
        Some(table)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LanguageModelData {
    pub words: Vec<WordEntry>,
    pub bigrams: NgramTable,
    pub trigrams: NgramTable,
}

impl LanguageModelData {
    pub fn encode(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(self.words.len() * 12 + self.bigrams.len() * 3);
        write_varint(&mut payload, self.words.len() as u64);
        for w in &self.words {
            let bytes = w.word.as_bytes();
            write_varint(&mut payload, bytes.len() as u64);
            payload.extend_from_slice(bytes);
            write_varint(&mut payload, w.count as u64);
            payload.extend_from_slice(&w.freq.to_le_bytes());
        }
        self.bigrams.write(&mut payload);
        self.trigrams.write(&mut payload);

        let mut out = Vec::with_capacity(payload.len() + 12);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&adler32(&payload).to_le_bytes());
        out.extend_from_slice(&payload);
        out
    }

    /// Parses a model file; returns `None` on any corruption instead of panicking.
    pub fn decode(data: &[u8]) -> Option<Self> {
        if data.len() < 12 || &data[0..4] != MAGIC {
            return None;
        }
        let version = u16::from_le_bytes([data[4], data[5]]);
        if version != VERSION {
            return None;
        }
        let checksum = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let payload = &data[12..];
        if adler32(payload) != checksum {
            return None;
        }

        let mut r = Reader { data: payload, pos: 0 };
        let n_words = r.varint()?;
        let n_words = u32::try_from(n_words).ok()?;
        let mut words = Vec::with_capacity((n_words as usize).min(payload.len()));
        for _ in 0..n_words {
            let len = r.varint()? as usize;
            let word = std::str::from_utf8(r.bytes(len)?).ok()?.to_string();
            let count = u32::try_from(r.varint()?).ok()?;
            let freq = u16::from_le_bytes([r.u8()?, r.u8()?]);
            words.push(WordEntry { word, count, freq });
        }
        let bigrams = NgramTable::read(&mut r, n_words)?;
        let trigrams = NgramTable::read(&mut r, n_words)?;
        Some(Self {
            words,
            bigrams,
            trigrams,
        })
    }

    /// Map from lowercase word to id (first, i.e. most frequent, spelling wins).
    pub fn lowercase_index(&self) -> HashMap<String, u32> {
        let mut map = HashMap::with_capacity(self.words.len());
        for (i, w) in self.words.iter().enumerate() {
            map.entry(w.word.to_lowercase()).or_insert(i as u32);
        }
        map
    }
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn u8(&mut self) -> Option<u8> {
        let b = *self.data.get(self.pos)?;
        self.pos += 1;
        Some(b)
    }

    fn bytes(&mut self, len: usize) -> Option<&'a [u8]> {
        let end = self.pos.checked_add(len)?;
        let slice = self.data.get(self.pos..end)?;
        self.pos = end;
        Some(slice)
    }

    fn varint(&mut self) -> Option<u64> {
        let mut value = 0u64;
        for shift in (0..64).step_by(7) {
            let b = self.u8()?;
            value |= ((b & 0x7F) as u64) << shift;
            if b & 0x80 == 0 {
                return Some(value);
            }
        }
        None
    }
}

fn write_varint(out: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        out.push((v as u8) | 0x80);
        v >>= 7;
    }
    out.push(v as u8);
}

pub fn adler32(data: &[u8]) -> u32 {
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> LanguageModelData {
        let words = vec![
            WordEntry { word: "как".into(), count: 900, freq: 2400 },
            WordEntry { word: "дела".into(), count: 300, freq: 1800 },
            WordEntry { word: "Москва".into(), count: 120, freq: 1200 },
        ];
        let bigrams = NgramTable::from_triples(vec![
            (ctx_id(0) as u64, 1, quantize(0.4)),
            (ctx_id(0) as u64, 2, quantize(0.01)),
            (SENTENCE_START as u64, 0, quantize(0.2)),
        ]);
        let trigrams = NgramTable::from_triples(vec![(trigram_key(SENTENCE_START, ctx_id(0)), 1, quantize(0.5))]);
        LanguageModelData { words, bigrams, trigrams }
    }

    #[test]
    fn roundtrip() {
        let model = sample();
        let decoded = LanguageModelData::decode(&model.encode()).expect("valid model");
        assert_eq!(decoded, model);
        assert_eq!(decoded.bigrams.get(ctx_id(0) as u64)[0].0, 1, "best continuation first");
        assert_eq!(decoded.bigrams.q(ctx_id(0) as u64, 2), Some(quantize(0.01)));
        assert_eq!(decoded.trigrams.get(trigram_key(SENTENCE_START, ctx_id(0))).len(), 1);
        assert_eq!(decoded.lowercase_index().get("москва"), Some(&2));
    }

    #[test]
    fn corrupted_data_is_rejected() {
        let mut bytes = sample().encode();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        assert!(LanguageModelData::decode(&bytes).is_none());
        assert!(LanguageModelData::decode(&bytes[..10]).is_none());
        assert!(LanguageModelData::decode(b"").is_none());
    }

    #[test]
    fn quantization_is_monotonic() {
        assert!(quantize(0.5) < quantize(0.05));
        assert_eq!(quantize(1.0), 0);
        assert!((dequantize_log10(quantize(0.01)) - (-2.0)).abs() < 0.05);
    }
}
