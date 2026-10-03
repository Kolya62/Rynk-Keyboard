pub struct Autocorrect;

impl Autocorrect {
    /// Computes Damerau-Levenshtein distance (insertions, deletions, substitutions, transpositions)
    /// Optimized with early length-difference exit and stack buffers (zero heap allocations for words <= 48 chars)
    pub fn edit_distance(s1: &str, s2: &str) -> usize {
        if s1 == s2 {
            return 0;
        }

        let mut v1_buf = ['\0'; 48];
        let mut v2_buf = ['\0'; 48];
        let mut len1 = 0;
        for (i, c) in s1.chars().enumerate() {
            if i < 48 {
                v1_buf[i] = c;
            }
            len1 += 1;
        }
        let mut len2 = 0;
        for (i, c) in s2.chars().enumerate() {
            if i < 48 {
                v2_buf[i] = c;
            }
            len2 += 1;
        }

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let diff = (len1 as isize - len2 as isize).unsigned_abs();
        if diff > 2 {
            return diff;
        }

        if len1 <= 48 && len2 <= 48 {
            let width = len2 + 1;
            let mut d = [0usize; 49 * 49];

            for i in 0..=len1 {
                d[i * width] = i;
            }
            for (j, item) in d.iter_mut().enumerate().take(len2 + 1) {
                *item = j;
            }

            for i in 1..=len1 {
                for j in 1..=len2 {
                    let cost = if v1_buf[i - 1] == v2_buf[j - 1] { 0 } else { 1 };

                    let del = d[(i - 1) * width + j] + 1;
                    let ins = d[i * width + (j - 1)] + 1;
                    let sub = d[(i - 1) * width + (j - 1)] + cost;

                    let mut val = del.min(ins).min(sub);

                    if i > 1
                        && j > 1
                        && v1_buf[i - 1] == v2_buf[j - 2]
                        && v1_buf[i - 2] == v2_buf[j - 1]
                    {
                        let trans = d[(i - 2) * width + (j - 2)] + 1;
                        val = val.min(trans);
                    }

                    d[i * width + j] = val;
                }
            }

            d[len1 * width + len2]
        } else {
            // Fallback for unusually long input strings
            let v1: Vec<char> = s1.chars().collect();
            let v2: Vec<char> = s2.chars().collect();
            let width = len2 + 1;
            let mut d = vec![0usize; (len1 + 1) * width];

            for i in 0..=len1 {
                d[i * width] = i;
            }
            for (j, item) in d.iter_mut().enumerate().take(len2 + 1) {
                *item = j;
            }

            for i in 1..=len1 {
                for j in 1..=len2 {
                    let cost = if v1[i - 1] == v2[j - 1] { 0 } else { 1 };
                    let del = d[(i - 1) * width + j] + 1;
                    let ins = d[i * width + (j - 1)] + 1;
                    let sub = d[(i - 1) * width + (j - 1)] + cost;
                    let mut val = del.min(ins).min(sub);

                    if i > 1 && j > 1 && v1[i - 1] == v2[j - 2] && v1[i - 2] == v2[j - 1] {
                        val = val.min(d[(i - 2) * width + (j - 2)] + 1);
                    }
                    d[i * width + j] = val;
                }
            }
            d[len1 * width + len2]
        }
    }

    /// Computes match score combining edit distance, length similarity, and frequency
    pub fn score_candidate(input: &str, candidate: &str, frequency: u32) -> f32 {
        if input.is_empty() || candidate.is_empty() {
            return 0.0;
        }

        let dist = Self::edit_distance(input, candidate);
        let input_len = input.chars().count();
        let cand_len = candidate.chars().count();
        let max_len = input_len.max(cand_len) as f32;
        if max_len == 0.0 {
            return 0.0;
        }

        // Distance filtering: never suggest words with large edit distance
        if dist > 2 {
            return 0.0;
        }
        if dist == 2 {
            if input_len < 3 || cand_len < 3 {
                return 0.0;
            }
            // For short words (len <= 4), only allow distance 2 if frequency is high and first or last matches
            if input_len <= 4 && cand_len <= 4 {
                let first_matches = input.chars().next() == candidate.chars().next();
                let last_matches = input.chars().last() == candidate.chars().last();
                if !(frequency >= 200 && (first_matches || last_matches)) {
                    return 0.0;
                }
            } else if frequency < 50 {
                return 0.0;
            }
        }

        let dist_base = match dist {
            0 => 10.0,
            1 => 3.0,
            2 => 1.5,
            _ => 0.0,
        };

        let len_diff = (input_len as f32 - cand_len as f32).abs();
        let len_penalty = len_diff * 0.35;

        let first_bonus = if input.chars().next() == candidate.chars().next() {
            0.5
        } else {
            0.0
        };
        let prefix_bonus = if candidate.starts_with(input) {
            0.6
        } else {
            0.0
        };

        // Meaningful frequency weight: normalized + log component
        let freq_weight = (frequency as f32).min(1500.0) / 1500.0 * 1.5;

        dist_base - len_penalty + first_bonus + prefix_bonus + freq_weight
    }

    /// Determines if a candidate is a high-confidence autocorrect replacement on spacebar
    pub fn is_confident_correction(typed: &str, candidate: &str, frequency: u32) -> bool {
        let t = typed.trim().to_lowercase();
        let c = candidate.trim().to_lowercase();
        if t == c {
            return false;
        }
        let t_len = t.chars().count();
        let c_len = c.chars().count();
        if t_len < 3 || c_len < 3 {
            return false;
        }

        // 1. Quick typo lookup: if typed is a known common typo and candidate matches, 100% confident
        if let Some(quick) = crate::prediction::typos::get_quick_correction(&t) {
            if quick.to_lowercase() == c {
                return true;
            }
        }

        // If candidate is a pure prefix extension of typed, do not auto-complete on space
        if c.starts_with(&t) {
            return false;
        }

        let dist = Self::edit_distance(&t, &c);
        let first_matches = t.chars().next() == c.chars().next();
        let last_matches = t.chars().last() == c.chars().last();

        if dist == 1 {
            // Single character substitution, insertion, deletion, or transposition.
            // If first letter matches: confident if frequency >= 50.
            if first_matches && frequency >= 50 {
                return true;
            }
            // If first letter differs (e.g., "зделал" -> "сделал", "ашибка" -> "ошибка"):
            // confident if word length >= 4 and frequency >= 50.
            if !first_matches && (t_len >= 4 || c_len >= 4) && frequency >= 50 {
                return true;
            }
        }

        if dist == 2 {
            // Allow distance 2:
            // For words >= 5 chars, if first matches and frequency >= 150 (e.g. "севодня" -> "сегодня", "вопще" -> "вообще")
            if (t_len >= 5 || c_len >= 5) && first_matches && frequency >= 150 {
                return true;
            }
            // For words >= 4 chars, if frequency >= 300 and at least first or last matches (e.g. "каво" -> "кого")
            if (t_len >= 4 && c_len >= 4) && (first_matches || last_matches) && frequency >= 300 {
                return true;
            }
        }

        false
    }
}
