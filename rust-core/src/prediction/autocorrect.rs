pub struct Autocorrect;

impl Autocorrect {
    /// Strips common diacritics and accents to base Latin / Cyrillic characters.
    /// This enables effortless typing in Romanian, Polish, Czech, French, German, Spanish, Turkish, etc.
    #[inline]
    pub fn strip_diacritics(c: char) -> char {
        match c {
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
            'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ė' | 'ę' | 'ě' => 'e',
            'ì' | 'í' | 'î' | 'ï' | 'ī' | 'į' => 'i',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ō' | 'ő' | 'ø' => 'o',
            'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' | 'ű' | 'ų' => 'u',
            'ý' | 'ÿ' => 'y',
            'ç' | 'ć' | 'č' | 'ĉ' | 'ċ' => 'c',
            'ď' | 'đ' => 'd',
            'ğ' | 'ġ' => 'g',
            'ł' => 'l',
            'ñ' | 'ń' | 'ň' | 'ņ' => 'n',
            'ř' | 'ŕ' => 'r',
            'š' | 'ś' | 'ş' | 'ș' => 's',
            'ť' | 'ț' => 't',
            'ž' | 'ź' | 'ż' => 'z',
            'ё' => 'е',
            'і' => 'и',
            'ї' => 'и',
            'ў' => 'у',
            other => other,
        }
    }

    /// Checks if two characters are adjacent neighbors on QWERTY or ЙЦУКЕН layouts
    pub fn is_layout_neighbor(c1: char, c2: char) -> bool {
        let n1 = Self::strip_diacritics(c1);
        let n2 = Self::strip_diacritics(c2);
        if n1 == n2 {
            return true;
        }

        // QWERTY adjacency
        let qwerty_adj = match n1 {
            'q' => "wa",
            'w' => "qesa",
            'e' => "wrsd",
            'r' => "etfd",
            't' => "rygf",
            'y' => "tuhg",
            'u' => "yijh",
            'i' => "uokj",
            'o' => "iplk",
            'p' => "ol",
            'a' => "qwsz",
            's' => "weadzx",
            'd' => "ersfxc",
            'f' => "rtdgcv",
            'g' => "tyfhvb",
            'h' => "yugjbn",
            'j' => "uihknm",
            'k' => "iojlm",
            'l' => "opk",
            'z' => "asx",
            'x' => "zsdc",
            'c' => "xdfv",
            'v' => "cfgb",
            'b' => "vghn",
            'n' => "bhjm",
            'm' => "njk",
            _ => "",
        };
        if qwerty_adj.contains(n2) {
            return true;
        }

        // ЙЦУКЕН adjacency
        let cyr_adj = match n1 {
            'й' => "цф",
            'ц' => "йуыф",
            'у' => "цквы",
            'к' => "уеав",
            'е' => "кнпа",
            'н' => "егрп",
            'г' => "ншор",
            'ш' => "гщло",
            'щ' => "шздл",
            'з' => "щхжд",
            'х' => "зэж",
            'ф' => "йцыя",
            'ы' => "цуфвяч",
            'в' => "укыачс",
            'а' => "кевпсм",
            'п' => "енарим",
            'р' => "нгпоит",
            'о' => "гшрлт",
            'л' => "шщодьб",
            'д' => "щзлжбю",
            'ж' => "зхдэю",
            'э' => "хж",
            'я' => "фыч",
            'ч' => "яывс",
            'с' => "чвам",
            'м' => "сапи",
            'и' => "мпрот",
            'т' => "ироь",
            'ь' => "толб",
            'б' => "ьлдю",
            'ю' => "бдж",
            _ => "",
        };
        cyr_adj.contains(n2)
    }

    /// Calculates substitution cost between two characters:
    /// 0.0 = identical
    /// 0.1 = exact base letter differing only by accent / diacritic (e.g. a vs ă, o vs ó, e vs ě)
    /// 0.5 = physical keyboard layout neighbor
    /// 1.0 = distinct letters
    #[inline]
    pub fn char_sub_cost(c1: char, c2: char) -> f32 {
        if c1 == c2 {
            0.0
        } else if Self::strip_diacritics(c1) == Self::strip_diacritics(c2) {
            0.1
        } else if Self::is_layout_neighbor(c1, c2) {
            0.5
        } else {
            1.0
        }
    }

    /// Computes weighted Damerau-Levenshtein distance incorporating
    /// diacritic equivalence, keyboard adjacency, and transpositions.
    pub fn edit_distance(s1: &str, s2: &str) -> usize {
        Self::weighted_edit_distance(s1, s2).round() as usize
    }

    /// Floating-point weighted distance for high-precision autocorrect ranking
    pub fn weighted_edit_distance(s1: &str, s2: &str) -> f32 {
        if s1 == s2 {
            return 0.0;
        }

        let v1: Vec<char> = s1.chars().collect();
        let v2: Vec<char> = s2.chars().collect();
        let len1 = v1.len();
        let len2 = v2.len();

        if len1 == 0 {
            return len2 as f32;
        }
        if len2 == 0 {
            return len1 as f32;
        }

        // Quick check for diacritic-only match
        if len1 == len2 {
            let mut all_diacritic = true;
            let mut diacritic_cost = 0.0;
            for i in 0..len1 {
                let cost = Self::char_sub_cost(v1[i], v2[i]);
                if cost > 0.1 {
                    all_diacritic = false;
                    break;
                }
                diacritic_cost += cost;
            }
            if all_diacritic {
                return diacritic_cost;
            }
        }

        // Morphology check: recognized prefix alternation or suffix/inflection match
        if let Some(prefix_cost) = crate::prediction::morphology::Morphology::analyze_prefix_match(s1, s2) {
            return prefix_cost;
        }
        if let Some(suffix_cost) = crate::prediction::morphology::Morphology::analyze_suffix_and_ending(s1, s2) {
            return suffix_cost;
        }

        let width = len2 + 1;
        let mut d = vec![0.0f32; (len1 + 1) * width];

        for i in 0..=len1 {
            d[i * width] = i as f32;
        }
        for j in 0..=len2 {
            d[j] = j as f32;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let c1 = v1[i - 1];
                let c2 = v2[j - 1];
                let cost = Self::char_sub_cost(c1, c2);

                // Double letter cost reduction (e.g. helo -> hello, aparat -> apparat)
                let del_cost = if i > 1 && v1[i - 1] == v1[i - 2] { 0.4 } else { 1.0 };
                let ins_cost = if j > 1 && v2[j - 1] == v2[j - 2] { 0.4 } else { 1.0 };

                let del = d[(i - 1) * width + j] + del_cost;
                let ins = d[i * width + (j - 1)] + ins_cost;
                let sub = d[(i - 1) * width + (j - 1)] + cost;

                let mut val = del.min(ins).min(sub);

                // Transposition
                if i > 1 && j > 1 && c1 == v2[j - 2] && v1[i - 2] == c2 {
                    let trans = d[(i - 2) * width + (j - 2)] + 0.7;
                    val = val.min(trans);
                }

                d[i * width + j] = val;
            }
        }

        d[len1 * width + len2]
    }

    /// Computes match score combining weighted edit distance, length similarity, and frequency
    pub fn score_candidate(input: &str, candidate: &str, frequency: u32) -> f32 {
        if input.is_empty() || candidate.is_empty() {
            return 0.0;
        }

        let dist = Self::weighted_edit_distance(input, candidate);
        let input_len = input.chars().count();
        let cand_len = candidate.chars().count();

        let morph_match = crate::prediction::morphology::Morphology::analyze_prefix_match(input, candidate).is_some()
            || crate::prediction::morphology::Morphology::analyze_suffix_and_ending(input, candidate).is_some();

        // Distance filtering: never suggest words with large edit distance unless morphology matches
        if !morph_match {
            if dist > 2.5 {
                return 0.0;
            }
            if dist > 1.8 && (input_len < 4 || cand_len < 4) {
                return 0.0;
            }
        }

        let dist_base = (10.0 - dist * 3.5).max(0.0);

        let len_diff = (input_len as f32 - cand_len as f32).abs();
        let len_penalty = len_diff * 0.3;

        let first_matches = input.chars().next() == candidate.chars().next()
            || Self::strip_diacritics(input.chars().next().unwrap_or('\0'))
                == Self::strip_diacritics(candidate.chars().next().unwrap_or('\0'))
            || morph_match;
        let first_bonus = if first_matches { 0.8 } else { 0.0 };

        let prefix_bonus = if candidate.starts_with(input) {
            1.0
        } else {
            0.0
        };

        let morph_bonus = if morph_match { 3.0 } else { 0.0 };

        let freq_weight = (frequency as f32).min(2000.0) / 2000.0 * 2.0;

        dist_base - len_penalty + first_bonus + prefix_bonus + morph_bonus + freq_weight
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
        if t_len < 2 || c_len < 2 {
            return false;
        }

        // 1. Quick typo lookup
        if let Some(quick) = crate::prediction::typos::get_quick_correction(&t) {
            if quick.to_lowercase() == c {
                return true;
            }
        }

        // 2. Exact match when stripping diacritics (e.g. "buna" -> "bună", "dziekuje" -> "dziękuję", "uber" -> "über")
        let t_norm: String = t.chars().map(Self::strip_diacritics).collect();
        let c_norm: String = c.chars().map(Self::strip_diacritics).collect();
        if t_norm == c_norm && frequency >= 50 {
            return true;
        }

        // 3. Morphology: Prefix or Suffix / Inflectional Ending match
        if crate::prediction::morphology::Morphology::analyze_prefix_match(&t, &c).is_some()
            || crate::prediction::morphology::Morphology::analyze_suffix_and_ending(&t, &c).is_some()
        {
            if frequency >= 40 {
                return true;
            }
        }

        // If candidate is a pure prefix extension of typed, do not auto-complete on space
        if c.starts_with(&t) {
            return false;
        }

        let dist = Self::weighted_edit_distance(&t, &c);
        let first_matches = t.chars().next() == c.chars().next()
            || Self::strip_diacritics(t.chars().next().unwrap_or('\0'))
                == Self::strip_diacritics(c.chars().next().unwrap_or('\0'));
        let last_matches = t.chars().last() == c.chars().last();

        if dist <= 1.0 {
            if (first_matches || last_matches) && frequency >= 50 {
                return true;
            }
            if (t_len >= 4 || c_len >= 4) && frequency >= 100 {
                return true;
            }
        }

        if dist <= 1.6 {
            if (t_len >= 4 && c_len >= 4) && first_matches && frequency >= 150 {
                return true;
            }
            if (t_len >= 5 || c_len >= 5) && frequency >= 250 {
                return true;
            }
        }

        false
    }
}
