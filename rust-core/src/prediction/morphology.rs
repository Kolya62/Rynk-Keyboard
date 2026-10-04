use crate::keyboard::state::Language;

/// High-performance morphology, affix, and grammatical context engine
/// Supporting prefixes (приставки), suffixes (суффиксы), inflectional endings (окончания),
/// and prepositions (предлоги) across all 80+ supported languages.
pub struct Morphology;

impl Morphology {
    // =========================================================================
    // 1. PREPOSITIONS & CONTEXT AGREEMENT (ПРЕДЛОГИ И КОНТЕКСТ)
    // =========================================================================

    /// Checks if a token is a recognized preposition in the target language
    pub fn is_preposition(word: &str, lang: Language) -> bool {
        let w = word.trim().to_lowercase();
        match lang {
            Language::Russian | Language::Belarusian | Language::Ukrainian => matches!(
                w.as_str(),
                "в" | "во" | "на" | "с" | "со" | "из" | "изо" | "к" | "ко" | "по" | "у" | "за"
                    | "о" | "об" | "обо" | "под" | "подо" | "над" | "надо" | "от" | "ото" | "до"
                    | "перед" | "передо" | "через" | "при" | "без" | "безо" | "для" | "ради"
                    | "около" | "после" | "кроме" | "вместо" | "между" | "про" | "сквозь"
                    | "вопреки" | "согласно" | "благодаря" | "насчет" | "навстречу"
            ),
            Language::English => matches!(
                w.as_str(),
                "in" | "on" | "at" | "to" | "for" | "with" | "by" | "from" | "of" | "about"
                    | "into" | "through" | "after" | "over" | "between" | "out" | "against"
                    | "during" | "without" | "before" | "under" | "around" | "among" | "upon"
                    | "towards" | "toward" | "across" | "behind" | "beyond" | "within"
            ),
            Language::German => matches!(
                w.as_str(),
                "in" | "an" | "auf" | "zu" | "mit" | "für" | "von" | "bei" | "nach" | "aus"
                    | "um" | "über" | "unter" | "vor" | "ohne" | "durch" | "gegen" | "zwischen"
                    | "seit" | "ab" | "trotz" | "wegen" | "während"
            ),
            Language::French => matches!(
                w.as_str(),
                "de" | "à" | "dans" | "sur" | "pour" | "avec" | "par" | "en" | "sous" | "chez"
                    | "vers" | "sans" | "entre" | "après" | "avant" | "contre" | "pendant"
                    | "depuis" | "selon" | "malgré"
            ),
            Language::Spanish => matches!(
                w.as_str(),
                "de" | "a" | "en" | "por" | "para" | "con" | "sin" | "sobre" | "entre" | "hasta"
                    | "desde" | "hacia" | "bajo" | "contra" | "según" | "durante" | "mediante"
            ),
            Language::Italian => matches!(
                w.as_str(),
                "di" | "a" | "da" | "in" | "con" | "su" | "per" | "tra" | "fra" | "sopra" | "sotto"
            ),
            Language::Portuguese => matches!(
                w.as_str(),
                "de" | "a" | "em" | "para" | "por" | "com" | "sem" | "sob" | "sobre" | "entre" | "até"
            ),
            Language::Polish => matches!(
                w.as_str(),
                "w" | "we" | "na" | "z" | "ze" | "do" | "od" | "po" | "o" | "u" | "za" | "dla"
                    | "pod" | "nad" | "przed" | "przez" | "przy" | "bez" | "między"
            ),
            Language::Czech => matches!(
                w.as_str(),
                "v" | "ve" | "na" | "s" | "se" | "do" | "od" | "po" | "o" | "u" | "za" | "pro"
                    | "pod" | "nad" | "před" | "přes" | "při" | "bez" | "mezi"
            ),
            Language::Romanian => matches!(
                w.as_str(),
                "de" | "la" | "în" | "pe" | "cu" | "pentru" | "prin" | "din" | "spre" | "fără" | "sub"
            ),
            Language::Turkish => matches!(
                w.as_str(),
                "için" | "ile" | "gibi" | "kadar" | "sonra" | "önce" | "göre" | "karşı"
            ),
            _ => {
                // Universal short preposition check (1-3 chars common particles/prepositions)
                w.len() <= 3
            }
        }
    }

    /// Returns high-probability contextual continuations when user is idle right after a preposition
    pub fn get_preposition_context_predictions(prep: &str, lang: Language) -> &'static [&'static str] {
        if !Self::is_preposition(prep, lang) {
            return &[];
        }
        let p = prep.trim().to_lowercase();
        match lang {
            Language::Russian | Language::Belarusian | Language::Ukrainian => match p.as_str() {
                "в" | "во" => &["том", "этом", "городе", "России", "начале", "школе", "порядке", "конце", "магазине", "доме"],
                "на" => &["самом", "работе", "сайте", "улице", "тему", "месте", "связи", "следующий", "днях", "пути"],
                "с" | "со" => &["вами", "тобой", "ним", "ней", "друзьями", "удовольствием", "начала", "новым", "каждым", "днем"],
                "к" | "ко" => &["вам", "нам", "нему", "ней", "сожалению", "примеру", "выводу", "врачу", "концу", "дому"],
                "по" => &["поводу", "факту", "телефону", "работе", "сути", "дороге", "всему", "утрам", "делу", "закону"],
                "для" => &["вас", "меня", "нас", "этого", "детей", "работы", "дома", "жизни", "себя", "всех"],
                "из" => &["за", "дома", "города", "этого", "них", "жизни", "окна", "школы", "России", "него"],
                "о" | "об" | "обо" => &["том", "этом", "тебе", "мне", "нас", "жизни", "любви", "работе", "семье", "детях"],
                "до" => &["конца", "свидания", "сих", "утра", "вечера", "слез", "дома", "утра", "зимы", "встречи"],
                "за" => &["счет", "то", "что", "тебя", "меня", "нас", "вас", "день", "работу", "рулем"],
                "у" => &["нас", "меня", "вас", "него", "нее", "них", "дома", "врача", "друзей", "двери"],
                "от" => &["вас", "меня", "души", "имени", "начала", "сердца", "работы", "дома", "него", "боли"],
                "под" => &["рукой", "контролем", "видом", "названием", "вопросом", "защитой", "дождем", "руку"],
                "без" => &["проблем", "сомнения", "труда", "внимания", "тебя", "меня", "всяких", "паники", "слов"],
                "после" => &["этого", "работы", "обеда", "уроков", "школы", "того", "того как", "смерти"],
                "через" => &["день", "час", "неделю", "год", "минуту", "дорогу", "несколько", "реку"],
                "при" => &["этом", "встрече", "жизни", "помощи", "условии", "работе", "себе", "входе"],
                _ => &[],
            },
            Language::English => match p.as_str() {
                "to" => &["be", "do", "have", "get", "go", "see", "make", "you", "me", "the"],
                "in" => &["the", "a", "my", "this", "case", "fact", "order", "touch", "front", "time"],
                "on" => &["the", "my", "time", "your", "this", "board", "top", "track", "line", "demand"],
                "with" => &["you", "me", "us", "them", "the", "him", "her", "my", "pleasure", "friends"],
                "for" => &["you", "me", "us", "the", "your", "this", "more", "instance", "now", "ever"],
                "of" => &["the", "course", "all", "my", "this", "these", "our", "them", "life", "us"],
                "at" => &["the", "all", "least", "home", "once", "work", "first", "night", "last", "times"],
                "from" => &["the", "you", "my", "here", "home", "now", "scratch", "this", "where", "day"],
                "by" => &["the", "way", "far", "then", "chance", "default", "myself", "law", "side", "now"],
                "about" => &["it", "the", "this", "that", "you", "what", "how", "time", "life", "work"],
                "after" => &["that", "all", "the", "work", "school", "hours", "dinner", "lunch", "years"],
                "before" => &["that", "the", "you", "going", "bedtime", "leaving", "long", "now", "sunrise"],
                "between" => &["the", "two", "us", "them", "you", "both", "these", "different"],
                "without" => &["any", "you", "hesitation", "doubt", "fear", "delay", "warning", "thinking"],
                _ => &[],
            },
            Language::German => match p.as_str() {
                "in" => &["der", "die", "das", "den", "dem", "einem", "einer", "diesem", "meiner"],
                "mit" => &["dem", "der", "den", "dir", "mir", "uns", "ihnen", "Freude", "Sicherheit"],
                "zu" => &["Hause", "sein", "tun", "haben", "der", "dem", "den", "Ende"],
                "für" => &["dich", "mich", "uns", "Sie", "die", "das", "den", "immer", "alle"],
                "auf" => &["der", "die", "das", "jeden", "Wiedersehen", "dem", "den", "meine"],
                "von" => &["der", "dem", "mir", "dir", "uns", "ihnen", "hier", "Anfang"],
                "nach" => &["Hause", "der", "dem", "vorne", "links", "rechts", "oben"],
                _ => &[],
            },
            Language::Spanish => match p.as_str() {
                "de" => &["la", "el", "los", "las", "mi", "este", "que", "nuevo", "acuerdo"],
                "a" => &["la", "los", "las", "ver", "hacer", "todos", "mi", "casa", "nadie"],
                "en" => &["el", "la", "casa", "cuenta", "este", "mi", "un", "una", "realidad"],
                "por" => &["favor", "ejemplo", "qué", "eso", "ti", "la", "el", "supuesto", "fin"],
                "para" => &["ti", "mí", "que", "hacer", "el", "la", "siempre", "todos", "nada"],
                "con" => &["el", "la", "usted", "mucho", "migo", "tigo", "amigos", "cuidado"],
                "sin" => &["embargo", "duda", "probleма", "ti", "saber", "miedo", "parar"],
                _ => &[],
            },
            Language::French => match p.as_str() {
                "de" => &["la", "l'", "le", "plus", "retour", "rien", "faire", "voir", "tout"],
                "à" => &["la", "l'", "le", "bientôt", "demain", "faire", "tous", "cause", "point"],
                "dans" => &["le", "la", "les", "un", "une", "ce", "cette", "mon", "ma"],
                "pour" => &["le", "la", "vous", "moi", "faire", "que", "tous", "toujours"],
                "avec" => &["vous", "moi", "plaisir", "lui", "elle", "nous", "joie", "soin"],
                "sans" => &["doute", "problème", "faute", "tard", "cesse", "vous", "moi"],
                _ => &[],
            },
            _ => &[],
        }
    }

    /// Evaluates grammatical agreement between a preceding preposition and a candidate word
    pub fn matches_preposition_agreement(prep: &str, candidate: &str, lang: Language) -> bool {
        let p = prep.trim().to_lowercase();
        let c = candidate.trim().to_lowercase();
        if c.is_empty() {
            return false;
        }

        match lang {
            Language::Russian | Language::Belarusian | Language::Ukrainian => {
                // Prepositions with Prepositional Case (-е, -ом, -ем, -ах, -ях, -ии, -и)
                if matches!(p.as_str(), "в" | "во" | "на" | "о" | "об" | "обо" | "при") {
                    if c.ends_with('е') || c.ends_with("ом") || c.ends_with("ем")
                        || c.ends_with("ах") || c.ends_with("ях") || c.ends_with("ии")
                        || matches!(c.as_str(), "этом" | "том" | "нем" | "ней" | "себе" | "мне" | "тебе" | "нас" | "вас" | "них") {
                        return true;
                    }
                }
                // Prepositions with Instrumental Case (-ом, -ем, -ой, -ей, -ами, -ями)
                if matches!(p.as_str(), "с" | "со" | "над" | "под" | "перед" | "за" | "между") {
                    if c.ends_with("ом") || c.ends_with("ем") || c.ends_with("ой") || c.ends_with("ей")
                        || c.ends_with("ами") || c.ends_with("ями") || c.ends_with("ью")
                        || matches!(c.as_str(), "мной" | "тобой" | "нами" | "вами" | "ним" | "ней" | "ними" | "этим" | "тем" | "собой") {
                        return true;
                    }
                }
                // Prepositions with Genitive Case (-а, -я, -ы, -и, -ов, -ев, -ей)
                if matches!(p.as_str(), "из" | "от" | "до" | "у" | "для" | "без" | "около" | "после" | "кроме") {
                    if c.ends_with('а') || c.ends_with('я') || c.ends_with('ы') || c.ends_with('и')
                        || c.ends_with("ов") || c.ends_with("ев") || c.ends_with("ей")
                        || matches!(c.as_str(), "меня" | "тебя" | "нас" | "вас" | "него" | "нее" | "них" | "этого" | "того" | "себя") {
                        return true;
                    }
                }
                // Prepositions with Dative Case (-у, -ю, -ам, -ям)
                if matches!(p.as_str(), "к" | "ко" | "по") {
                    if c.ends_with('у') || c.ends_with('ю') || c.ends_with("ам") || c.ends_with("ям")
                        || matches!(c.as_str(), "мне" | "тебе" | "нам" | "вам" | "нему" | "ней" | "ним" | "этому" | "тому") {
                        return true;
                    }
                }
            }
            Language::English => {
                if p == "to" {
                    // Verbs (infinitive) or pronouns
                    if matches!(c.as_str(), "be" | "do" | "have" | "go" | "see" | "get" | "make" | "know" | "take" | "say" | "you" | "me" | "him" | "her" | "us" | "them" | "the") {
                        return true;
                    }
                } else {
                    // Pronouns or gerunds (-ing) or determiners
                    if c.ends_with("ing") || matches!(c.as_str(), "the" | "a" | "an" | "this" | "that" | "my" | "your" | "his" | "our" | "their" | "me" | "you" | "him" | "her" | "us" | "them" | "it") {
                        return true;
                    }
                }
            }
            _ => {}
        }
        false
    }

    // =========================================================================
    // 2. PREFIXES (ПРИСТАВКИ)
    // =========================================================================

    /// Known prefix alternations and phonological pairs
    const PREFIX_PAIRS: &'static [(&'static str, &'static str)] = &[
        // Russian prefixes
        ("пре", "при"),
        ("при", "пре"),
        ("раз", "рас"),
        ("рас", "раз"),
        ("разо", "рас"),
        ("без", "бес"),
        ("бес", "без"),
        ("из", "ис"),
        ("ис", "из"),
        ("воз", "вос"),
        ("вос", "воз"),
        ("вз", "вс"),
        ("вс", "вз"),
        ("низ", "нис"),
        ("нис", "низ"),
        ("з", "с"),
        ("па", "по"),
        ("пра", "про"),
        ("ат", "от"),
        ("аб", "об"),
        ("пад", "под"),
        ("да", "до"),
        // English / Latin prefixes
        ("un", "in"),
        ("in", "un"),
        ("un", "im"),
        ("im", "un"),
        ("in", "im"),
        ("im", "in"),
        ("dis", "mis"),
        ("mis", "dis"),
        ("diss", "dis"),
        ("diss", "des"),
    ];

    /// Common prefix prefixes list for stem extraction
    #[allow(dead_code)]
    const COMMON_PREFIXES: &'static [&'static str] = &[
        "пере", "пре", "при", "про", "по", "под", "подо", "от", "ото", "об", "обо", "до",
        "за", "на", "над", "надо", "в", "во", "вы", "с", "со", "из", "ис", "изо", "раз",
        "рас", "разо", "без", "бес", "безо", "воз", "вос", "возо", "не", "ни", "недо",
        "un", "in", "im", "il", "ir", "dis", "mis", "re", "pre", "post", "non", "anti",
        "sub", "super", "inter", "trans", "over", "under", "out",
    ];

    /// Analyzes whether two words share the same stem through recognized prefix variation.
    /// Returns Some(cost) if a prefix typo/alternation was detected (cost between 0.1 and 0.35).
    pub fn analyze_prefix_match(input: &str, candidate: &str) -> Option<f32> {
        let in_low = input.trim().to_lowercase();
        let cand_low = candidate.trim().to_lowercase();

        if in_low == cand_low {
            return Some(0.0);
        }

        // Test known prefix pairs
        for &(p1, p2) in Self::PREFIX_PAIRS {
            if in_low.starts_with(p1) && cand_low.starts_with(p2) {
                let stem1 = &in_low[p1.len()..];
                let stem2 = &cand_low[p2.len()..];
                if stem1 == stem2 && stem1.chars().count() >= 2 {
                    return Some(0.15);
                }
                // Double letter prefix typo (e.g. расказать -> рассказать, разсказ -> рассказ)
                if (p1 == "ра" && p2 == "рас" && stem2.starts_with('с') && stem1 == &stem2[1..])
                    || (p1 == "бе" && p2 == "бес" && stem2.starts_with('с') && stem1 == &stem2[1..])
                {
                    return Some(0.15);
                }
            }
        }

        // Test single 'з' vs 'с' prefix typo (зделать -> сделать, згореть -> сгореть, збросить -> сбросить)
        if in_low.starts_with('з') && cand_low.starts_with('с') {
            let s1 = &in_low[1..];
            let s2 = &cand_low[1..];
            if s1 == s2 && s1.chars().count() >= 3 {
                return Some(0.12);
            }
        }

        // Double 's' prefix typo in English (dissapoint -> disappoint, dissappear -> disappear)
        if in_low.starts_with("diss") && cand_low.starts_with("dis") {
            let s1 = &in_low[4..];
            let s2 = &cand_low[3..];
            let s1_collapsed: String = s1.chars().fold(String::new(), |mut acc, c| {
                if acc.chars().last() != Some(c) {
                    acc.push(c);
                }
                acc
            });
            let s2_collapsed: String = s2.chars().fold(String::new(), |mut acc, c| {
                if acc.chars().last() != Some(c) {
                    acc.push(c);
                }
                acc
            });
            if (s1 == s2 || s1_collapsed == s2_collapsed) && s1.chars().count() >= 3 {
                return Some(0.15);
            }
        }

        None
    }

    // =========================================================================
    // 3. SUFFIXES & ENDINGS (СУФФИКСЫ И ОКОНЧАНИЯ)
    // =========================================================================

    const ADJECTIVE_ENDINGS_RU: &'static [&'static str] = &[
        "ый", "ий", "ой", "ая", "яя", "ое", "ее", "ые", "ие", "ого", "его",
        "ому", "ему", "ым", "им", "ом", "ем", "ую", "юю", "ых", "их", "ыми", "ими",
    ];

    const VERB_ENDINGS_RU: &'static [&'static str] = &[
        "ть", "ти", "чь", "у", "ю", "ешь", "ёшь", "ет", "ёт", "ем", "ём", "ете", "ёте",
        "ут", "ют", "ат", "ят", "ил", "ила", "ило", "или", "ал", "ала", "ало", "али",
        "ел", "ела", "ело", "ели", "ся", "сь", "тся", "ться",
    ];

    const NOUN_ENDINGS_RU: &'static [&'static str] = &[
        "а", "я", "о", "е", "ы", "и", "у", "ю", "ом", "ем", "ой", "ей", "ам", "ям",
        "ами", "ями", "ах", "ях", "ов", "ев",
    ];

    const ENGLISH_AFFIX_TYPOS: &'static [(&'static str, &'static str)] = &[
        ("ance", "ence"),
        ("ence", "ance"),
        ("able", "ible"),
        ("ible", "able"),
        ("ly", "ley"),
        ("ly", "ely"),
        ("ely", "ly"),
        ("tion", "sion"),
        ("sion", "tion"),
        ("ing", "in"),
        ("in", "ing"),
        ("ately", "itely"),
        ("itely", "ately"),
    ];

    /// Analyzes whether two words share the same root/stem and differ only in inflectional ending
    /// or suffix spelling. Returns Some(cost) if an affix/ending correspondence matches.
    pub fn analyze_suffix_and_ending(input: &str, candidate: &str) -> Option<f32> {
        let in_low = input.trim().to_lowercase();
        let cand_low = candidate.trim().to_lowercase();

        if in_low == cand_low {
            return Some(0.0);
        }

        // 1. Specific high-frequency suffix orthographic rules:
        // -тся vs -ться (нравится / нравиться, хочется / хотеться, удастся / удасться)
        if (in_low.ends_with("тса") || in_low.ends_with("ца") || in_low.ends_with("ться"))
            && cand_low.ends_with("тся")
        {
            let stem_cand = cand_low.trim_end_matches("тся");
            if in_low.starts_with(stem_cand) {
                return Some(0.1);
            }
        }
        if (in_low.ends_with("тса") || in_low.ends_with("ца") || in_low.ends_with("тся"))
            && cand_low.ends_with("ться")
        {
            let stem_cand = cand_low.trim_end_matches("ться");
            if in_low.starts_with(stem_cand) {
                return Some(0.1);
            }
        }

        // 2. Russian 2nd person verb ending soft-sign drop:
        // делаеш -> делаешь, знаеш -> знаешь, хочеш -> хочешь, можеш -> можешь
        if in_low.ends_with("еш") && cand_low.ends_with("ешь") {
            if in_low.trim_end_matches("еш") == cand_low.trim_end_matches("ешь") {
                return Some(0.1);
            }
        }
        if in_low.ends_with("иш") && cand_low.ends_with("ишь") {
            if in_low.trim_end_matches("иш") == cand_low.trim_end_matches("ишь") {
                return Some(0.1);
            }
        }

        // 3. -н- vs -нн- alternation:
        // жареный / жаренный, ветреный / ветренный, стекляный -> стеклянный
        if in_low.ends_with("ный") && cand_low.ends_with("нный") {
            let s1 = in_low.trim_end_matches("ный");
            let s2 = cand_low.trim_end_matches("нный");
            if s1 == s2 {
                return Some(0.1);
            }
        }
        if in_low.ends_with("нный") && cand_low.ends_with("ный") {
            let s1 = in_low.trim_end_matches("нный");
            let s2 = cand_low.trim_end_matches("ный");
            if s1 == s2 {
                return Some(0.1);
            }
        }

        // 4. English suffix typos (ance/ence, able/ible, ly/ely):
        for &(s1, s2) in Self::ENGLISH_AFFIX_TYPOS {
            if in_low.ends_with(s1) && cand_low.ends_with(s2) {
                let stem1 = in_low.trim_end_matches(s1);
                let stem2 = cand_low.trim_end_matches(s2);
                let s1_c: String = stem1.chars().fold(String::new(), |mut acc, c| {
                    if acc.chars().last() != Some(c) { acc.push(c); }
                    acc
                });
                let s2_c: String = stem2.chars().fold(String::new(), |mut acc, c| {
                    if acc.chars().last() != Some(c) { acc.push(c); }
                    acc
                });
                if (stem1 == stem2 || s1_c == s2_c) && stem1.chars().count() >= 3 {
                    return Some(0.15);
                }
            }
        }

        // 5. English consonant doubling before -ing / -ed:
        // runing -> running, swimmin -> swimming, stoped -> stopped, happend -> happened
        if in_low.ends_with("ing") && cand_low.ends_with("ing") {
            let stem1 = in_low.trim_end_matches("ing");
            let stem2 = cand_low.trim_end_matches("ing");
            if stem2.len() == stem1.len() + 1 && stem2.starts_with(stem1) {
                return Some(0.15);
            }
        }
        if in_low.ends_with("ed") && cand_low.ends_with("ed") {
            let stem1 = in_low.trim_end_matches("ed");
            let stem2 = cand_low.trim_end_matches("ed");
            if (stem2.len() == stem1.len() + 1 && stem2.starts_with(stem1))
                || (stem1.len() == stem2.len() + 1 && stem1.starts_with(stem2))
            {
                return Some(0.15);
            }
        }

        // 6. General grammatical inflectional ending alternation across Russian/Slavic:
        // Find longest common stem
        let in_chars: Vec<char> = in_low.chars().collect();
        let cand_chars: Vec<char> = cand_low.chars().collect();
        let common_stem_len = in_chars
            .iter()
            .zip(cand_chars.iter())
            .take_while(|(a, b)| a == b)
            .count();

        if common_stem_len >= 3 {
            let end_in: String = in_chars[common_stem_len..].iter().collect();
            let end_cand: String = cand_chars[common_stem_len..].iter().collect();

            let is_adj_in = Self::ADJECTIVE_ENDINGS_RU.contains(&end_in.as_str());
            let is_adj_cand = Self::ADJECTIVE_ENDINGS_RU.contains(&end_cand.as_str());
            if is_adj_in && is_adj_cand {
                return Some(0.25);
            }

            let is_verb_in = Self::VERB_ENDINGS_RU.contains(&end_in.as_str());
            let is_verb_cand = Self::VERB_ENDINGS_RU.contains(&end_cand.as_str());
            if is_verb_in && is_verb_cand {
                return Some(0.25);
            }

            let is_noun_in = Self::NOUN_ENDINGS_RU.contains(&end_in.as_str());
            let is_noun_cand = Self::NOUN_ENDINGS_RU.contains(&end_cand.as_str());
            if is_noun_in && is_noun_cand {
                return Some(0.25);
            }
        }

        None
    }
}
