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
pub static AR_WORDS_RAW: &str = include_str!("data/ar_words.txt");
pub static PL_WORDS_RAW: &str = include_str!("data/pl_words.txt");
pub static CS_WORDS_RAW: &str = include_str!("data/cs_words.txt");
pub static RO_WORDS_RAW: &str = include_str!("data/ro_words.txt");
pub static NL_WORDS_RAW: &str = include_str!("data/nl_words.txt");
pub static SV_WORDS_RAW: &str = include_str!("data/sv_words.txt");
pub static NO_WORDS_RAW: &str = include_str!("data/no_words.txt");
pub static DA_WORDS_RAW: &str = include_str!("data/da_words.txt");
pub static FI_WORDS_RAW: &str = include_str!("data/fi_words.txt");
pub static EL_WORDS_RAW: &str = include_str!("data/el_words.txt");
pub static HE_WORDS_RAW: &str = include_str!("data/he_words.txt");
pub static FA_WORDS_RAW: &str = include_str!("data/fa_words.txt");
pub static UR_WORDS_RAW: &str = include_str!("data/ur_words.txt");
pub static HI_WORDS_RAW: &str = include_str!("data/hi_words.txt");
pub static BN_WORDS_RAW: &str = include_str!("data/bn_words.txt");
pub static ID_WORDS_RAW: &str = include_str!("data/id_words.txt");
pub static MS_WORDS_RAW: &str = include_str!("data/ms_words.txt");
pub static VI_WORDS_RAW: &str = include_str!("data/vi_words.txt");
pub static TH_WORDS_RAW: &str = include_str!("data/th_words.txt");
pub static HU_WORDS_RAW: &str = include_str!("data/hu_words.txt");
pub static BG_WORDS_RAW: &str = include_str!("data/bg_words.txt");
pub static SR_WORDS_RAW: &str = include_str!("data/sr_words.txt");
pub static HR_WORDS_RAW: &str = include_str!("data/hr_words.txt");
pub static SK_WORDS_RAW: &str = include_str!("data/sk_words.txt");
pub static SL_WORDS_RAW: &str = include_str!("data/sl_words.txt");
pub static LT_WORDS_RAW: &str = include_str!("data/lt_words.txt");
pub static LV_WORDS_RAW: &str = include_str!("data/lv_words.txt");
pub static ET_WORDS_RAW: &str = include_str!("data/et_words.txt");
pub static KA_WORDS_RAW: &str = include_str!("data/ka_words.txt");
pub static HY_WORDS_RAW: &str = include_str!("data/hy_words.txt");
pub static AZ_WORDS_RAW: &str = include_str!("data/az_words.txt");
pub static UZ_WORDS_RAW: &str = include_str!("data/uz_words.txt");
pub static TG_WORDS_RAW: &str = include_str!("data/tg_words.txt");
pub static KY_WORDS_RAW: &str = include_str!("data/ky_words.txt");
pub static TK_WORDS_RAW: &str = include_str!("data/tk_words.txt");
pub static MN_WORDS_RAW: &str = include_str!("data/mn_words.txt");
pub static TL_WORDS_RAW: &str = include_str!("data/tl_words.txt");
pub static SQ_WORDS_RAW: &str = include_str!("data/sq_words.txt");
pub static BS_WORDS_RAW: &str = include_str!("data/bs_words.txt");
pub static MK_WORDS_RAW: &str = include_str!("data/mk_words.txt");
pub static IS_WORDS_RAW: &str = include_str!("data/is_words.txt");
pub static GA_WORDS_RAW: &str = include_str!("data/ga_words.txt");
pub static CY_WORDS_RAW: &str = include_str!("data/cy_words.txt");
pub static EU_WORDS_RAW: &str = include_str!("data/eu_words.txt");
pub static CA_WORDS_RAW: &str = include_str!("data/ca_words.txt");
pub static GL_WORDS_RAW: &str = include_str!("data/gl_words.txt");
pub static AF_WORDS_RAW: &str = include_str!("data/af_words.txt");
pub static SW_WORDS_RAW: &str = include_str!("data/sw_words.txt");
pub static HA_WORDS_RAW: &str = include_str!("data/ha_words.txt");
pub static YO_WORDS_RAW: &str = include_str!("data/yo_words.txt");
pub static IG_WORDS_RAW: &str = include_str!("data/ig_words.txt");
pub static ZU_WORDS_RAW: &str = include_str!("data/zu_words.txt");
pub static EO_WORDS_RAW: &str = include_str!("data/eo_words.txt");
pub static LA_WORDS_RAW: &str = include_str!("data/la_words.txt");
pub static TA_WORDS_RAW: &str = include_str!("data/ta_words.txt");
pub static TE_WORDS_RAW: &str = include_str!("data/te_words.txt");
pub static MR_WORDS_RAW: &str = include_str!("data/mr_words.txt");
pub static GU_WORDS_RAW: &str = include_str!("data/gu_words.txt");
pub static KN_WORDS_RAW: &str = include_str!("data/kn_words.txt");
pub static ML_WORDS_RAW: &str = include_str!("data/ml_words.txt");
pub static PA_WORDS_RAW: &str = include_str!("data/pa_words.txt");
pub static NE_WORDS_RAW: &str = include_str!("data/ne_words.txt");
pub static SI_WORDS_RAW: &str = include_str!("data/si_words.txt");
pub static MY_WORDS_RAW: &str = include_str!("data/my_words.txt");
pub static KM_WORDS_RAW: &str = include_str!("data/km_words.txt");
pub static AM_WORDS_RAW: &str = include_str!("data/am_words.txt");
pub static SO_WORDS_RAW: &str = include_str!("data/so_words.txt");
pub static KU_WORDS_RAW: &str = include_str!("data/ku_words.txt");
pub static MT_WORDS_RAW: &str = include_str!("data/mt_words.txt");
pub static KO_WORDS_RAW: &str = include_str!("data/ko_words.txt");
pub static JA_WORDS_RAW: &str = include_str!("data/ja_words.txt");
pub static ZH_CN_WORDS_RAW: &str = include_str!("data/zh_cn_words.txt");
pub static ZH_TW_WORDS_RAW: &str = include_str!("data/zh_tw_words.txt");
pub static ZH_HK_WORDS_RAW: &str = include_str!("data/zh_hk_words.txt");
pub static PROFANITY_RAW: &str = include_str!("data/profanity.txt");
pub static RU_BIGRAMS_RAW: &str = include_str!("data/ru_bigrams.txt");
pub static EN_BIGRAMS_RAW: &str = include_str!("data/en_bigrams.txt");
pub static MULTI_BIGRAMS_RAW: &str = include_str!("data/multi_bigrams.txt");

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
    pub tries: HashMap<Language, Trie>,
    pub word_lists: HashMap<Language, Vec<(&'static str, u32)>>,
    pub candidate_buckets: HashMap<Language, CandidateBucketMap>,
    pub bigrams: HashMap<Language, HashMap<String, Vec<String>>>,
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

fn get_raw_words_for_lang(lang: Language) -> &'static str {
    match lang {
        Language::Russian => RU_WORDS_RAW,
        Language::English => EN_WORDS_RAW,
        Language::German => DE_WORDS_RAW,
        Language::French => FR_WORDS_RAW,
        Language::Spanish => ES_WORDS_RAW,
        Language::Portuguese => PT_WORDS_RAW,
        Language::Italian => IT_WORDS_RAW,
        Language::Turkish => TR_WORDS_RAW,
        Language::Ukrainian => UK_WORDS_RAW,
        Language::Belarusian => BE_WORDS_RAW,
        Language::Kazakh => KK_WORDS_RAW,
        Language::Arabic => AR_WORDS_RAW,
        Language::Polish => PL_WORDS_RAW,
        Language::Czech => CS_WORDS_RAW,
        Language::Romanian => RO_WORDS_RAW,
        Language::Dutch => NL_WORDS_RAW,
        Language::Swedish => SV_WORDS_RAW,
        Language::Norwegian => NO_WORDS_RAW,
        Language::Danish => DA_WORDS_RAW,
        Language::Finnish => FI_WORDS_RAW,
        Language::Greek => EL_WORDS_RAW,
        Language::Hebrew => HE_WORDS_RAW,
        Language::Persian => FA_WORDS_RAW,
        Language::Urdu => UR_WORDS_RAW,
        Language::Hindi => HI_WORDS_RAW,
        Language::Bengali => BN_WORDS_RAW,
        Language::Indonesian => ID_WORDS_RAW,
        Language::Malay => MS_WORDS_RAW,
        Language::Vietnamese => VI_WORDS_RAW,
        Language::Thai => TH_WORDS_RAW,
        Language::Hungarian => HU_WORDS_RAW,
        Language::Bulgarian => BG_WORDS_RAW,
        Language::Serbian => SR_WORDS_RAW,
        Language::Croatian => HR_WORDS_RAW,
        Language::Slovak => SK_WORDS_RAW,
        Language::Slovenian => SL_WORDS_RAW,
        Language::Lithuanian => LT_WORDS_RAW,
        Language::Latvian => LV_WORDS_RAW,
        Language::Estonian => ET_WORDS_RAW,
        Language::Georgian => KA_WORDS_RAW,
        Language::Armenian => HY_WORDS_RAW,
        Language::Azerbaijani => AZ_WORDS_RAW,
        Language::Uzbek => UZ_WORDS_RAW,
        Language::Tajik => TG_WORDS_RAW,
        Language::Kyrgyz => KY_WORDS_RAW,
        Language::Turkmen => TK_WORDS_RAW,
        Language::Mongolian => MN_WORDS_RAW,
        Language::Tagalog => TL_WORDS_RAW,
        Language::Albanian => SQ_WORDS_RAW,
        Language::Bosnian => BS_WORDS_RAW,
        Language::Macedonian => MK_WORDS_RAW,
        Language::Icelandic => IS_WORDS_RAW,
        Language::Irish => GA_WORDS_RAW,
        Language::Welsh => CY_WORDS_RAW,
        Language::Basque => EU_WORDS_RAW,
        Language::Catalan => CA_WORDS_RAW,
        Language::Galician => GL_WORDS_RAW,
        Language::Afrikaans => AF_WORDS_RAW,
        Language::Swahili => SW_WORDS_RAW,
        Language::Hausa => HA_WORDS_RAW,
        Language::Yoruba => YO_WORDS_RAW,
        Language::Igbo => IG_WORDS_RAW,
        Language::Zulu => ZU_WORDS_RAW,
        Language::Esperanto => EO_WORDS_RAW,
        Language::Latin => LA_WORDS_RAW,
        Language::Tamil => TA_WORDS_RAW,
        Language::Telugu => TE_WORDS_RAW,
        Language::Marathi => MR_WORDS_RAW,
        Language::Gujarati => GU_WORDS_RAW,
        Language::Kannada => KN_WORDS_RAW,
        Language::Malayalam => ML_WORDS_RAW,
        Language::Punjabi => PA_WORDS_RAW,
        Language::Nepali => NE_WORDS_RAW,
        Language::Sinhala => SI_WORDS_RAW,
        Language::Burmese => MY_WORDS_RAW,
        Language::Khmer => KM_WORDS_RAW,
        Language::Amharic => AM_WORDS_RAW,
        Language::Somali => SO_WORDS_RAW,
        Language::Kurdish => KU_WORDS_RAW,
        Language::Maltese => MT_WORDS_RAW,
        Language::Korean => KO_WORDS_RAW,
        Language::Japanese => JA_WORDS_RAW,
        Language::ChineseSimplified => ZH_CN_WORDS_RAW,
        Language::ChineseTraditional => ZH_TW_WORDS_RAW,
        Language::Cantonese => ZH_HK_WORDS_RAW,
    }
}

impl Dictionary {
    pub fn new() -> Self {
        let mut profanity = HashSet::with_capacity(200);
        for line in PROFANITY_RAW.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                profanity.insert(trimmed);
            }
        }

        let mut bigrams: HashMap<Language, HashMap<String, Vec<String>>> = HashMap::new();

        // 1. Russian bigrams
        let ru_map = bigrams.entry(Language::Russian).or_default();
        for line in RU_BIGRAMS_RAW.lines() {
            let mut parts = line.split(':');
            if let (Some(w), Some(nexts)) = (parts.next(), parts.next()) {
                let list: Vec<String> = nexts.split(',').map(|s| s.trim().to_string()).collect();
                ru_map.insert(w.to_string(), list);
            }
        }

        // 2. English bigrams
        let en_map = bigrams.entry(Language::English).or_default();
        for line in EN_BIGRAMS_RAW.lines() {
            let mut parts = line.split(':');
            if let (Some(w), Some(nexts)) = (parts.next(), parts.next()) {
                let list: Vec<String> = nexts.split(',').map(|s| s.trim().to_string()).collect();
                en_map.insert(w.to_string(), list);
            }
        }

        // 3. Multi-language bigrams
        for line in MULTI_BIGRAMS_RAW.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.split(':');
            if let (Some(code), Some(w), Some(nexts)) = (parts.next(), parts.next(), parts.next()) {
                if let Some(lang) = Language::from_code(code) {
                    let map = bigrams.entry(lang).or_default();
                    let list: Vec<String> = nexts.split(',').map(|s| s.trim().to_string()).collect();
                    map.insert(w.to_string(), list);
                }
            }
        }

        let mut dict = Self {
            tries: HashMap::new(),
            word_lists: HashMap::new(),
            candidate_buckets: HashMap::new(),
            bigrams,
            profanity,
            profanity_enabled: true,
            user_dict: HashMap::new(),
            adaptive_dict: AdaptiveDictionary::default(),
            removed_words: HashSet::new(),
        };

        // Always load Russian and English as primary base languages
        dict.ensure_language_loaded(Language::Russian);
        dict.ensure_language_loaded(Language::English);

        dict
    }

    pub fn ensure_language_loaded(&mut self, lang: Language) {
        if self.tries.contains_key(&lang) {
            return;
        }

        let raw = get_raw_words_for_lang(lang);
        let cap = match lang {
            Language::Russian => 45000,
            Language::English => 22000,
            Language::Turkish
            | Language::Kazakh
            | Language::Polish
            | Language::German
            | Language::Czech
            | Language::Azerbaijani => 5500,
            Language::Korean
            | Language::Japanese
            | Language::ChineseSimplified
            | Language::ChineseTraditional
            | Language::Cantonese => 3500,
            _ => 2500,
        };
        let words = parse_words(raw, cap);

        let mut trie = Trie::default();
        let mut buckets: CandidateBucketMap = HashMap::new();

        for &(w, freq) in &words {
            trie.insert(w, freq);
            let c0 = w
                .chars()
                .next()
                .unwrap_or('\0')
                .to_lowercase()
                .next()
                .unwrap_or('\0');
            let len = w.chars().count();
            buckets.entry((c0, len)).or_default().push((w, freq));
        }

        self.tries.insert(lang, trie);
        self.word_lists.insert(lang, words);
        self.candidate_buckets.insert(lang, buckets);
    }

    pub fn ensure_languages_loaded(&mut self, langs: &[Language]) {
        for &lang in langs {
            self.ensure_language_loaded(lang);
        }
    }

    pub fn set_profanity_enabled(&mut self, enabled: bool) {
        self.profanity_enabled = enabled;
    }

    pub fn get_trie(&self, lang: Language) -> &Trie {
        if let Some(trie) = self.tries.get(&lang) {
            trie
        } else if let Some(trie) = self.tries.get(&Language::Russian) {
            trie
        } else {
            self.tries.values().next().expect("At least one trie loaded")
        }
    }

    pub fn get_word_list(&self, lang: Language) -> &[(&'static str, u32)] {
        if let Some(list) = self.word_lists.get(&lang) {
            list
        } else if let Some(list) = self.word_lists.get(&Language::Russian) {
            list
        } else {
            &[]
        }
    }

    pub fn get_fuzzy_candidates(&self, query: &str, lang: Language) -> Vec<(&'static str, u32)> {
        let clean = query.trim().to_lowercase();
        let query_len = clean.chars().count();
        if query_len == 0 {
            return Vec::new();
        }

        let buckets_opt = self.candidate_buckets.get(&lang).or_else(|| self.candidate_buckets.get(&Language::Russian));
        let buckets = match buckets_opt {
            Some(b) => b,
            None => return Vec::new(),
        };

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

        // Prefix variants check (e.g. зделал -> check 'с', unpossible -> check 'i', etc.)
        let alt_c0 = match c0 {
            Some('з') => Some('с'),
            Some('с') => Some('з'),
            Some('u') if clean.starts_with("un") => Some('i'),
            Some('i') if clean.starts_with("in") || clean.starts_with("im") => Some('u'),
            Some('d') if clean.starts_with("dis") => Some('m'),
            Some('m') if clean.starts_with("mis") => Some('d'),
            _ => None,
        };
        if let Some(alt) = alt_c0 {
            for len in min_len..=max_len {
                if let Some(list) = buckets.get(&(alt, len)) {
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
        for trie in self.tries.values() {
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
        if let Some(trie) = self.tries.get(&Language::Russian) {
            if let Some(f) = trie.get_frequency(&clean) {
                return f;
            }
        }
        if let Some(trie) = self.tries.get(&Language::English) {
            if let Some(f) = trie.get_frequency(&clean) {
                return f;
            }
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
        let lang = if is_ru { Language::Russian } else { Language::English };
        self.ensure_language_loaded(lang);
        if let Some(trie) = self.tries.get_mut(&lang) {
            trie.insert(&clean, 2500);
        }
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
            let lang = if is_ru { Language::Russian } else { Language::English };
            self.ensure_language_loaded(lang);
            if let Some(trie) = self.tries.get_mut(&lang) {
                trie.insert(&trimmed, freq);
            }
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

        // 2. Builtin bigrams for this language
        if let Some(map) = self.bigrams.get(&lang) {
            if let Some(nexts) = map.get(&k) {
                for w in nexts {
                    if !res.contains(w) && !self.removed_words.contains(w) {
                        res.push(w.clone());
                    }
                }
            }
        }

        // 3. Preposition context predictions from Morphology engine
        let prep_preds = crate::prediction::morphology::Morphology::get_preposition_context_predictions(&k, lang);
        for &pw in prep_preds {
            let s = pw.to_string();
            if !res.contains(&s) && !self.removed_words.contains(&s) {
                res.push(s);
            }
        }

        // 3. Fallback to English/Russian bigrams if not found
        if res.is_empty() {
            let fallback_map = if lang == Language::Russian {
                self.bigrams.get(&Language::English)
            } else {
                self.bigrams.get(&Language::Russian)
            };
            if let Some(map) = fallback_map {
                if let Some(nexts) = map.get(&k) {
                    for w in nexts {
                        if !res.contains(w) && !self.removed_words.contains(w) {
                            res.push(w.clone());
                        }
                    }
                }
            }
        }

        res
    }
}
