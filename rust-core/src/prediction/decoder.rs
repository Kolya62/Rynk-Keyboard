//! Noisy-channel spelling decoder.
//!
//! Finds dictionary words that could have produced what the user typed: every touch is scored
//! against each key with a 2D Gaussian around the key center (so a touch on the border between
//! two keys is cheap to reinterpret), and letters can be omitted, added or swapped at a cost.
//! Costs are -log10 probabilities; the search walks the sorted lexicon like a trie, best-first.

use super::autocorrect::Autocorrect;
use super::lexicon::{LexEntry, Lexicon};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use super::lexicon::{ChildRanges, FastMap};

/// Missing letter (user skipped it)
const INSERT_COST: f32 = 2.0;
/// Missing second letter of a double ("helo" -> "hello")
const DOUBLE_INSERT_COST: f32 = 0.9;
/// Extra letter (user typed one too many)
const DELETE_COST: f32 = 2.0;
/// Extra repeat of the previous letter ("helllo")
const DOUBLE_DELETE_COST: f32 = 0.9;
const TRANSPOSE_COST: f32 = 1.6;
/// Substitution costs when there is no touch position
const DIACRITIC_COST: f32 = 0.3;
const PHONETIC_COST: f32 = 1.2;
const NEIGHBOR_COST: f32 = 1.3;
const OTHER_SUB_COST: f32 = 2.8;
/// Touch noise as a fraction of the key size
const SIGMA_X: f32 = 0.5;
const SIGMA_Y: f32 = 0.5;
const MAX_SPATIAL_COST: f32 = 6.0;
/// Completing a word the user has not finished typing
const COMPLETION_BASE_COST: f32 = 0.6;
const COMPLETION_CHAR_COST: f32 = 0.25;
const COMPLETION_SCAN_LIMIT: usize = 2000;
const MAX_EXPANSIONS: usize = 20_000;
const BEAM_PER_POSITION: usize = 60;
/// Offset separating lazy insertion states from regular ones in the visited map
const PENDING_KEY: usize = 1 << 20;
/// Search stops this far (log10) above the cheapest candidate found
const RELATIVE_MARGIN: f32 = 2.5;
/// Cached child ranges per language before the cache is reset
const CHILDREN_CACHE_LIMIT: usize = 200_000;
const MAX_COMPLETION_STATES: usize = 12;
/// Substitutions at least this expensive count against the edit budget
const FAR_SUBSTITUTION_COST: f32 = 1.5;

/// Key centers of the current letter layout
#[derive(Clone, Debug, Default)]
pub struct KeyGeometry {
    pub centers: HashMap<char, (f32, f32)>,
    pub key_width: f32,
    pub key_height: f32,
}

impl KeyGeometry {
    pub fn is_empty(&self) -> bool {
        self.centers.is_empty()
    }

    /// -log10 of the Gaussian touch likelihood for `key`, up to a constant; `None` if the
    /// letter is not on the layout.
    fn spatial_cost(&self, key: char, point: (f32, f32)) -> Option<f32> {
        let &(cx, cy) = self
            .centers
            .get(&key)
            .or_else(|| self.centers.get(&Autocorrect::strip_diacritics(key)))?;
        let dx = (point.0 - cx) / (SIGMA_X * self.key_width);
        let dy = (point.1 - cy) / (SIGMA_Y * self.key_height);
        Some(0.5 * (dx * dx + dy * dy) * std::f32::consts::LOG10_E)
    }
}

/// One typed letter, with the touch position when it came from a tap
#[derive(Clone, Copy, Debug)]
pub struct Observation {
    pub ch: char,
    pub point: Option<(f32, f32)>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Candidate {
    /// Canonical spelling
    pub word: &'static str,
    pub freq: u32,
    /// -log10 P(typed | word)
    pub cost: f32,
    /// Number of letters added after the last typed one (0 = a full-word correction)
    pub completed_chars: usize,
}

pub struct Decoder<'a> {
    entries: &'a [LexEntry],
    obs: &'a [Observation],
    geometry: Option<&'a KeyGeometry>,
    /// Substitution cost per (observation, letter)
    sub_cache: FastMap<(usize, char), f32>,
    /// Child ranges per (range, byte offset)
    children_cache: ChildCache<'a>,
}

/// The base word list's cache is shared with later calls (consecutive keystrokes explore
/// mostly the same part of the lexicon); the small user overlay gets a throwaway one.
enum ChildCache<'a> {
    Shared(std::sync::MutexGuard<'a, FastMap<(u32, u32, u32), ChildRanges>>),
    Local(FastMap<(u32, u32, u32), ChildRanges>),
}

impl ChildCache<'_> {
    fn map(&mut self) -> &mut FastMap<(u32, u32, u32), ChildRanges> {
        match self {
            ChildCache::Shared(g) => g,
            ChildCache::Local(m) => m,
        }
    }
}

#[derive(PartialEq)]
struct State {
    cost: f32,
    lo: usize,
    hi: usize,
    /// Prefix length in bytes and in chars
    off: usize,
    depth: usize,
    /// Observations consumed
    i: usize,
    /// Last letter of the prefix (cheaper double-letter insertion)
    last: Option<char>,
    /// Insertions, deletions and swaps so far (touch substitutions are not counted)
    edits: u8,
    /// Lazy "a letter was skipped here" state: expands into one insertion per child when
    /// popped, instead of pushing ~30 insertion states that the beam would mostly discard
    insert_pending: bool,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap on cost
        other.cost.total_cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Decoder<'a> {
    pub fn new(lexicon: &'a Lexicon, obs: &'a [Observation], geometry: Option<&'a KeyGeometry>) -> Self {
        Self {
            entries: lexicon.entries(),
            obs,
            geometry: geometry.filter(|g| !g.is_empty()),
            sub_cache: FastMap::default(),
            children_cache: ChildCache::Shared(lexicon.children_cache()),
        }
    }

    /// Decoder over the lexicon's user/learned words
    pub fn for_overlay(lexicon: &'a Lexicon, obs: &'a [Observation], geometry: Option<&'a KeyGeometry>) -> Self {
        Self {
            entries: lexicon.overlay_entries(),
            obs,
            geometry: geometry.filter(|g| !g.is_empty()),
            sub_cache: FastMap::default(),
            children_cache: ChildCache::Local(FastMap::default()),
        }
    }

    /// Edit operations allowed for an input of `n` letters
    fn max_edits(n: usize) -> u8 {
        match n {
            0..=4 => 1,
            5..=8 => 2,
            _ => 3,
        }
    }

    fn sub_cost(&mut self, i: usize, c: char) -> f32 {
        if let Some(&v) = self.sub_cache.get(&(i, c)) {
            return v;
        }
        let o = self.obs[i];
        let typed = o.ch;
        let v = if c == typed {
            0.0
        } else {
            let text_cost = if Autocorrect::strip_diacritics(c) == Autocorrect::strip_diacritics(typed) {
                DIACRITIC_COST
            } else if Autocorrect::is_phonetic_confusion(c, typed) {
                PHONETIC_COST
            } else if Autocorrect::is_layout_neighbor(c, typed) {
                NEIGHBOR_COST
            } else {
                OTHER_SUB_COST
            };
            let spatial = match (self.geometry, o.point) {
                (Some(g), Some(p)) => match (g.spatial_cost(c, p), g.spatial_cost(typed, p)) {
                    (Some(sc), Some(st)) => {
                        let diacritic = if g.centers.contains_key(&c) { 0.0 } else { DIACRITIC_COST };
                        Some((sc - st).clamp(0.0, MAX_SPATIAL_COST) + diacritic)
                    }
                    _ => None,
                },
                _ => None,
            };
            // Spelling confusions (phonetic, diacritics) stay possible whatever the touch says
            match spatial {
                Some(s) => s.min(if text_cost < NEIGHBOR_COST { text_cost } else { MAX_SPATIAL_COST }),
                None => text_cost,
            }
        };
        self.sub_cache.insert((i, c), v);
        v
    }

    /// Next letter of `e` after `off` bytes
    fn next_char(e: &LexEntry, off: usize) -> Option<char> {
        e.lower.get(off..).and_then(|s| s.chars().next())
    }

    /// Child ranges of [lo, hi) at byte offset `off`: (letter, lo, hi)
    fn children(&mut self, lo: usize, hi: usize, off: usize) -> ChildRanges {
        let key = (lo as u32, hi as u32, off as u32);
        if let Some(c) = self.children_cache.map().get(&key) {
            return c.clone();
        }
        if self.children_cache.map().len() >= CHILDREN_CACHE_LIMIT {
            self.children_cache.map().clear();
        }
        let c: ChildRanges = std::sync::Arc::new(
            self.compute_children(lo, hi, off).into_iter().map(|(c, l, h)| (c, l as u32, h as u32)).collect(),
        );
        self.children_cache.map().insert(key, c.clone());
        c
    }

    fn compute_children(&self, lo: usize, hi: usize, off: usize) -> Vec<(char, usize, usize)> {
        let mut out = Vec::new();
        let mut j = lo;
        // The entry equal to the prefix itself sorts first
        while j < hi && self.entries[j].lower.len() == off {
            j += 1;
        }
        while j < hi {
            let Some(c) = Self::next_char(&self.entries[j], off) else { break };
            let mut buf = [0u8; 4];
            let cs: &str = c.encode_utf8(&mut buf);
            let len = self.entries[j..hi].partition_point(|e| e.lower[off..].starts_with(cs));
            out.push((c, j, j + len));
            j += len.max(1);
        }
        out
    }

    fn child(&mut self, lo: usize, hi: usize, off: usize, c: char) -> Option<(usize, usize)> {
        self.children(lo, hi, off).iter().find(|ch| ch.0 == c).map(|ch| (ch.1 as usize, ch.2 as usize))
    }

    /// Words reachable within `max_cost`, cheapest first. With `completions`, words longer than
    /// the input are offered too, as completions of the typed prefix.
    pub fn decode(mut self, max_cost: f32, limit: usize, completions: bool) -> Vec<Candidate> {
        let n = self.obs.len();
        let max_edits = Self::max_edits(n);
        // Completions are collected for the cheapest few ways to explain the whole input only
        let mut completion_states = 0;
        let mut found: FastMap<usize, Candidate> = FastMap::default();
        let mut best_seen: FastMap<(usize, usize, usize), f32> = FastMap::default();
        let mut heap = BinaryHeap::new();
        heap.push(State {
            cost: 0.0,
            lo: 0,
            hi: self.entries.len(),
            off: 0,
            depth: 0,
            i: 0,
            last: None,
            edits: 0,
            insert_pending: false,
        });
        let mut expansions = 0;
        // Beam: the cheapest states per input position are expanded, the rest dropped
        let mut expanded_at = vec![0usize; n + 1];

        let push = |heap: &mut BinaryHeap<State>, best: &mut FastMap<(usize, usize, usize), f32>, st: State| {
            if st.cost > max_cost {
                return;
            }
            let k = (st.lo, st.hi, st.i + if st.insert_pending { PENDING_KEY } else { 0 });
            if best.get(&k).is_some_and(|&c| c <= st.cost) {
                return;
            }
            best.insert(k, st.cost);
            heap.push(st);
        };

        while let Some(s) = heap.pop() {
            // Candidates far costlier than the best one found are hundreds of times less likely
            let bound = found.values().map(|c| c.cost).fold(max_cost, |b, c| b.min(c + RELATIVE_MARGIN));
            if s.cost > bound || expansions >= MAX_EXPANSIONS || found.len() >= limit * 2 {
                break;
            }
            expansions += 1;
            let key = (s.lo, s.hi, s.i + if s.insert_pending { PENDING_KEY } else { 0 });
            if best_seen.get(&key).is_some_and(|&c| c < s.cost) {
                continue;
            }
            if !s.insert_pending {
                if expanded_at[s.i] >= BEAM_PER_POSITION {
                    continue;
                }
                expanded_at[s.i] += 1;
            }

            if s.insert_pending {
                // The pending cost assumed the cheap double-letter case; others pay the rest
                let children = self.children(s.lo, s.hi, s.off);
                for &(c, lo, hi) in children.iter() {
                    let (lo, hi) = (lo as usize, hi as usize);
                    let extra = if s.last == Some(c) { 0.0 } else { INSERT_COST - DOUBLE_INSERT_COST };
                    push(&mut heap, &mut best_seen, State {
                        cost: s.cost + extra,
                        lo,
                        hi,
                        off: s.off + c.len_utf8(),
                        depth: s.depth + 1,
                        i: s.i,
                        last: Some(c),
                        edits: s.edits,
                        insert_pending: false,
                    });
                }
                continue;
            }

            if s.i == n {
                // The whole input is explained: the prefix itself may be a word
                if s.lo < s.hi && self.entries[s.lo].lower.len() == s.off {
                    found.entry(s.lo).or_insert_with(|| candidate(&self.entries[s.lo], s.cost, 0));
                }
                if completions && s.depth > 0 && completion_states < MAX_COMPLETION_STATES {
                    completion_states += 1;
                    self.collect_completions(&s, max_cost, &mut found);
                }
            }



            let can_edit = s.edits < max_edits;
            // Extra typed letter: skip the observation
            if s.i < n && can_edit {
                let doubled = s.i > 0 && self.obs[s.i].ch == self.obs[s.i - 1].ch;
                let c = if doubled { DOUBLE_DELETE_COST } else { DELETE_COST };
                push(&mut heap, &mut best_seen, State { cost: s.cost + c, i: s.i + 1, edits: s.edits + 1, ..s });
            }

            // Letter the user skipped (before the end of the input)
            if can_edit && (s.i < n || !completions) {
                push(&mut heap, &mut best_seen, State {
                    cost: s.cost + DOUBLE_INSERT_COST,
                    edits: s.edits + 1,
                    insert_pending: true,
                    ..s
                });
            }

            let children = self.children(s.lo, s.hi, s.off);
            for &(c, lo, hi) in children.iter() {
                let (lo, hi) = (lo as usize, hi as usize);
                let off = s.off + c.len_utf8();
                // Typed letter matches / was a neighbor
                if s.i < n {
                    let sc = self.sub_cost(s.i, c);
                    // Near misses are free to repeat; a far substitution is an edit like a typo, and
                    // with a known touch position it is not worth exploring at all
                    let far = sc >= FAR_SUBSTITUTION_COST;
                    if !far || (can_edit && self.obs[s.i].point.is_none()) {
                        push(&mut heap, &mut best_seen, State {
                            cost: s.cost + sc, lo, hi, off, depth: s.depth + 1, i: s.i + 1, last: Some(c),
                            edits: s.edits + far as u8,
                            insert_pending: false,
                        });
                    }
                    // Two typed letters swapped
                    if can_edit && s.i + 1 < n && c != self.obs[s.i].ch {
                        let c2 = self.obs[s.i].ch;
                        let first = self.sub_cost(s.i + 1, c);
                        if first < NEIGHBOR_COST {
                            if let Some((lo2, hi2)) = self.child(lo, hi, off, c2) {
                                push(&mut heap, &mut best_seen, State {
                                    cost: s.cost + first + TRANSPOSE_COST,
                                    lo: lo2,
                                    hi: hi2,
                                    off: off + c2.len_utf8(),
                                    depth: s.depth + 2,
                                    i: s.i + 2,
                                    last: Some(c2),
                                    edits: s.edits + 1,
                                    insert_pending: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        let mut out: Vec<Candidate> = found.into_values().collect();
        out.sort_by(|a, b| a.cost.total_cmp(&b.cost).then(b.freq.cmp(&a.freq)));
        out.truncate(limit);
        out
    }

    /// Words of the current range that continue past the input, most frequent first.
    fn collect_completions(&self, s: &State, max_cost: f32, found: &mut FastMap<usize, Candidate>) {
        // Top few by frequency without sorting the range
        const TOP: usize = 6;
        let mut top: [(u32, usize); TOP] = [(0, usize::MAX); TOP];
        for j in s.lo..s.hi.min(s.lo + COMPLETION_SCAN_LIMIT) {
            let e = &self.entries[j];
            if e.lower.len() <= s.off || e.freq <= top[TOP - 1].0 {
                continue;
            }
            let mut k = TOP - 1;
            while k > 0 && top[k - 1].0 < e.freq {
                top[k] = top[k - 1];
                k -= 1;
            }
            top[k] = (e.freq, j);
        }
        for &(_, j) in top.iter().filter(|t| t.1 != usize::MAX) {
            let e = &self.entries[j];
            let extra = e.lower[s.off..].chars().count();
            let cost = s.cost + COMPLETION_BASE_COST + COMPLETION_CHAR_COST * extra as f32;
            if cost <= max_cost && found.get(&j).is_none_or(|c| c.cost > cost) {
                found.insert(j, candidate(e, cost, extra));
            }
        }
    }
}

fn candidate(e: &LexEntry, cost: f32, completed_chars: usize) -> Candidate {
    Candidate { word: e.canonical, freq: e.freq, cost, completed_chars }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lexicon() -> Lexicon {
        Lexicon::from_words(
            ["привет", "привести", "прицел", "hello", "help", "the", "thank", "thanks", "tank", "them"]
                .iter()
                .map(|w| (w.to_string(), 1000))
                .collect(),
        )
    }

    fn typed(s: &str) -> Vec<Observation> {
        s.chars().map(|ch| Observation { ch, point: None }).collect()
    }

    fn words(c: &[Candidate]) -> Vec<&str> {
        c.iter().map(|c| c.word).collect()
    }

    #[test]
    fn finds_exact_and_edited_words() {
        let lex = lexicon();
        let obs = typed("hello");
        let c = Decoder::new(&lex, &obs, None).decode(3.0, 5, false);
        assert_eq!(c[0].word, "hello");
        assert_eq!(c[0].cost, 0.0);

        // Missing letter, double letter, swap
        let obs = typed("thnk");
        assert!(words(&Decoder::new(&lex, &obs, None).decode(3.0, 5, false)).contains(&"thank"));
        let obs = typed("helo");
        assert_eq!(Decoder::new(&lex, &obs, None).decode(3.0, 5, false)[0].word, "hello");
        let obs = typed("teh");
        assert_eq!(Decoder::new(&lex, &obs, None).decode(3.0, 5, false)[0].word, "the");
    }

    #[test]
    fn touch_position_decides_between_neighbors() {
        let lex = lexicon();
        let mut g = KeyGeometry { key_width: 100.0, key_height: 150.0, ..Default::default() };
        // A row "a s d" and the letters of "tank"/"thank" far away
        for (i, c) in ['t', 'h', 'a', 'n', 'k', 'e', 'm'].iter().enumerate() {
            g.centers.insert(*c, (i as f32 * 100.0 + 50.0, 75.0));
        }
        // Typed "tank", but the 'a' touch landed right at the border with 'h'
        let mut obs = typed("tank");
        obs[1].point = Some((140.0, 75.0));
        let c = Decoder::new(&lex, &obs, Some(&g)).decode(4.0, 5, false);
        let thank = c.iter().find(|c| c.word == "thank").expect("thank reachable");
        assert!(thank.cost < 3.0, "border touch makes 'h' plausible: {}", thank.cost);
    }

    #[test]
    fn completions_extend_a_prefix() {
        let lex = lexicon();
        let obs = typed("прив");
        let c = Decoder::new(&lex, &obs, None).decode(3.0, 5, true);
        assert!(c.iter().any(|c| c.word == "привет" && c.completed_chars == 2));
        // Without completions only full words of similar length come back
        let c = Decoder::new(&lex, &obs, None).decode(3.0, 5, false);
        assert!(c.iter().all(|c| c.completed_chars == 0));
    }
}
