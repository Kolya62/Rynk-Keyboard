//! Gesture (glide) typing decoder, in the spirit of SHARK²: a word's "ideal path" is the
//! polyline through its key centers; candidates are words whose first and last keys are near
//! the start and end of the finger path and whose keys all lie close to it; they are ranked by
//! how far the resampled ideal path is from the finger path, plus the context model.

use super::autocorrect::Autocorrect;
use super::decoder::KeyGeometry;
use super::dictionary::Dictionary;
use super::lm::WordContext;
use crate::keyboard::state::Language;

/// Points both paths are resampled to
const SAMPLES: usize = 40;
/// Candidate first/last keys: within this many key widths of the path ends, at most this many
const END_RADIUS_KEYS: f32 = 1.1;
const END_CANDIDATES: usize = 3;
/// Every key of a word must pass within this distance of the finger path
const PASS_RADIUS_KEYS: f32 = 1.0;
/// log10 penalty per key width of mean distance between the paths
const LOCATION_WEIGHT: f32 = 5.0;
const LM_WEIGHT: f32 = 1.0;
/// Longer words are not gestured
const MAX_WORD_CHARS: usize = 24;

#[derive(Clone, Debug, PartialEq)]
pub struct GestureCandidate {
    pub word: String,
    pub score: f32,
}

/// Distance in key units (x by key width, y by key height)
fn key_dist(g: &KeyGeometry, a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = (a.0 - b.0) / g.key_width.max(1.0);
    let dy = (a.1 - b.1) / g.key_height.max(1.0);
    (dx * dx + dy * dy).sqrt()
}

/// Resamples a polyline to `n` points evenly spaced along its length.
pub fn resample(points: &[(f32, f32)], n: usize) -> Vec<(f32, f32)> {
    if points.is_empty() || n == 0 {
        return Vec::new();
    }
    if points.len() == 1 {
        return vec![points[0]; n];
    }
    let seg: Vec<f32> = points
        .windows(2)
        .map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt())
        .collect();
    let total: f32 = seg.iter().sum();
    if total <= f32::EPSILON {
        return vec![points[0]; n];
    }
    let step = total / (n - 1) as f32;
    let mut out = Vec::with_capacity(n);
    let (mut i, mut acc) = (0usize, 0.0f32);
    for k in 0..n {
        let target = step * k as f32;
        while i < seg.len() - 1 && acc + seg[i] < target {
            acc += seg[i];
            i += 1;
        }
        let t = if seg[i] > 0.0 { ((target - acc) / seg[i]).clamp(0.0, 1.0) } else { 0.0 };
        let (a, b) = (points[i], points[i + 1]);
        out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
    }
    out
}

/// Keys closest to `p` within the end radius, nearest first.
fn nearby_keys(g: &KeyGeometry, p: (f32, f32)) -> Vec<char> {
    let mut keys: Vec<(char, f32)> = g
        .centers
        .iter()
        .map(|(&c, &center)| (c, key_dist(g, p, center)))
        .filter(|&(_, d)| d <= END_RADIUS_KEYS)
        .collect();
    keys.sort_by(|a, b| a.1.total_cmp(&b.1));
    keys.into_iter().take(END_CANDIDATES).map(|(c, _)| c).collect()
}

fn key_center(g: &KeyGeometry, c: char) -> Option<(f32, f32)> {
    g.centers.get(&c).or_else(|| g.centers.get(&Autocorrect::strip_diacritics(c))).copied()
}

impl Dictionary {
    /// Words the finger path most likely spells, best first.
    pub fn decode_gesture(
        &self,
        lang: Language,
        ctx: &WordContext,
        path: &[(f32, f32)],
        geometry: &KeyGeometry,
        limit: usize,
    ) -> Vec<GestureCandidate> {
        if path.len() < 2 || geometry.is_empty() {
            return Vec::new();
        }
        let user = resample(path, SAMPLES);
        let starts = nearby_keys(geometry, path[0]);
        let ends = nearby_keys(geometry, *path.last().unwrap());
        let lexicon = self.get_lexicon(lang);
        let entries = lexicon.entries();

        let mut found: Vec<GestureCandidate> = Vec::new();
        let mut centers: Vec<(f32, f32)> = Vec::with_capacity(MAX_WORD_CHARS);
        for &s in &starts {
            let mut buf = [0u8; 4];
            let prefix: &str = s.encode_utf8(&mut buf);
            let lo = entries.partition_point(|e| e.lower < prefix);
            for e in entries[lo..].iter().take_while(|e| e.lower.starts_with(prefix)) {
                let Some(last) = e.lower.chars().last() else { continue };
                if !ends.contains(&last) && !ends.contains(&Autocorrect::strip_diacritics(last)) {
                    continue;
                }
                centers.clear();
                let mut ok = true;
                for c in e.lower.chars() {
                    match key_center(geometry, c) {
                        Some(p) => {
                            // A double letter is a single key on the path
                            if centers.last() != Some(&p) {
                                centers.push(p);
                            }
                        }
                        None => {
                            ok = false;
                            break;
                        }
                    }
                }
                if !ok || centers.len() < 2 || centers.len() > MAX_WORD_CHARS {
                    continue;
                }
                // Every key must lie near the finger path
                if !centers
                    .iter()
                    .all(|&c| user.iter().any(|&u| key_dist(geometry, u, c) <= PASS_RADIUS_KEYS))
                {
                    continue;
                }
                let ideal = resample(&centers, SAMPLES);
                let location: f32 =
                    user.iter().zip(&ideal).map(|(&u, &i)| key_dist(geometry, u, i)).sum::<f32>() / SAMPLES as f32;
                let removed = self.removed_words.contains(e.lower)
                    || (!self.profanity_enabled && self.profanity.contains(e.lower));
                if removed {
                    continue;
                }
                let score = -LOCATION_WEIGHT * location + LM_WEIGHT * self.context_log10(lang, ctx, e.canonical);
                found.push(GestureCandidate { word: e.canonical.to_string(), score });
            }
        }
        found.sort_by(|a, b| b.score.total_cmp(&a.score));
        found.truncate(limit);
        found
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prediction::lm_data::{LanguageModelData, WordEntry};

    /// QWERTY rows with 100×150 keys
    fn qwerty() -> KeyGeometry {
        let mut g = KeyGeometry { key_width: 100.0, key_height: 150.0, ..Default::default() };
        for (row, letters) in ["qwertyuiop", "asdfghjkl", "zxcvbnm"].iter().enumerate() {
            for (i, c) in letters.chars().enumerate() {
                let x = 50.0 + i as f32 * 100.0 + row as f32 * 50.0;
                g.centers.insert(c, (x, 75.0 + row as f32 * 150.0));
            }
        }
        g
    }

    fn dict(words: &[&str]) -> Dictionary {
        let words = words
            .iter()
            .map(|w| WordEntry { word: w.to_string(), count: 1, freq: 1500 })
            .collect();
        let mut d = Dictionary::new();
        d.install_model(Language::Ukrainian, LanguageModelData { words, ..Default::default() });
        d
    }

    /// Finger path through a word's keys with a sideways wobble
    fn swipe(g: &KeyGeometry, word: &str, wobble: f32) -> Vec<(f32, f32)> {
        let pts: Vec<(f32, f32)> = word.chars().map(|c| g.centers[&c]).collect();
        resample(&pts, 60)
            .into_iter()
            .enumerate()
            .map(|(i, (x, y))| (x + (i as f32 * 0.7).sin() * wobble, y + (i as f32 * 1.3).cos() * wobble))
            .collect()
    }

    #[test]
    fn resample_is_even() {
        let r = resample(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0)], 5);
        assert_eq!(r.len(), 5);
        assert_eq!(r[0], (0.0, 0.0));
        assert_eq!(r[2], (10.0, 0.0));
        assert_eq!(r[4], (10.0, 10.0));
    }

    #[test]
    fn decodes_words_from_paths() {
        let g = qwerty();
        let d = dict(&["hello", "help", "hell", "world", "word", "the", "they", "toy", "try"]);
        for word in ["hello", "world", "the", "try"] {
            let got = d.decode_gesture(Language::Ukrainian, &WordContext::default(), &swipe(&g, word, 25.0), &g, 3);
            assert_eq!(got.first().map(|c| c.word.as_str()), Some(word), "{word}: {got:?}");
        }
    }
}
