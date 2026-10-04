use super::key::KeyboardMode;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShiftState {
    Off,
    Shifted,
    CapsLock,
}

impl ShiftState {
    pub fn is_uppercase(&self) -> bool {
        matches!(self, ShiftState::Shifted | ShiftState::CapsLock)
    }

    pub fn cycle_on_tap(&self) -> Self {
        match self {
            ShiftState::Off => ShiftState::Shifted,
            ShiftState::Shifted => ShiftState::Off,
            ShiftState::CapsLock => ShiftState::Off,
        }
    }

    pub fn on_char_typed(&self) -> Self {
        match self {
            ShiftState::Shifted => ShiftState::Off,
            other => *other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Russian,
    English,
    German,
    French,
    Spanish,
    Portuguese,
    Italian,
    Turkish,
    Ukrainian,
    Belarusian,
    Kazakh,
    Arabic,
    Polish,
    Czech,
    Romanian,
    Dutch,
    Swedish,
    Norwegian,
    Danish,
    Finnish,
    Greek,
    Hebrew,
    Persian,
    Urdu,
    Hindi,
    Bengali,
    Indonesian,
    Malay,
    Vietnamese,
    Thai,
    Hungarian,
    Bulgarian,
    Serbian,
    Croatian,
    Slovak,
    Slovenian,
    Lithuanian,
    Latvian,
    Estonian,
    Georgian,
    Armenian,
    Azerbaijani,
    Uzbek,
    Tajik,
    Kyrgyz,
    Turkmen,
    Mongolian,
    Tagalog,
    Albanian,
    Bosnian,
    Macedonian,
    Icelandic,
    Irish,
    Welsh,
    Basque,
    Catalan,
    Galician,
    Afrikaans,
    Swahili,
    Hausa,
    Yoruba,
    Igbo,
    Zulu,
    Esperanto,
    Latin,
    Tamil,
    Telugu,
    Marathi,
    Gujarati,
    Kannada,
    Malayalam,
    Punjabi,
    Nepali,
    Sinhala,
    Burmese,
    Khmer,
    Amharic,
    Somali,
    Kurdish,
    Maltese,
    Korean,
    Japanese,
    ChineseSimplified,
    ChineseTraditional,
    Cantonese,
}

impl Language {
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Russian => "Русский",
            Language::English => "English",
            Language::German => "Deutsch",
            Language::French => "Français",
            Language::Spanish => "Español",
            Language::Portuguese => "Português",
            Language::Italian => "Italiano",
            Language::Turkish => "Türkçe",
            Language::Ukrainian => "Українська",
            Language::Belarusian => "Беларуская",
            Language::Kazakh => "Қазақша",
            Language::Arabic => "العربية",
            Language::Polish => "Polski",
            Language::Czech => "Čeština",
            Language::Romanian => "Română",
            Language::Dutch => "Nederlands",
            Language::Swedish => "Svenska",
            Language::Norwegian => "Norsk",
            Language::Danish => "Dansk",
            Language::Finnish => "Suomi",
            Language::Greek => "Ελληνικά",
            Language::Hebrew => "עברית",
            Language::Persian => "فارسی",
            Language::Urdu => "اردو",
            Language::Hindi => "हिन्दी",
            Language::Bengali => "বাংলা",
            Language::Indonesian => "Bahasa Indonesia",
            Language::Malay => "Bahasa Melayu",
            Language::Vietnamese => "Tiếng Việt",
            Language::Thai => "ไทย",
            Language::Hungarian => "Magyar",
            Language::Bulgarian => "Български",
            Language::Serbian => "Српски",
            Language::Croatian => "Hrvatski",
            Language::Slovak => "Slovenčina",
            Language::Slovenian => "Slovenščina",
            Language::Lithuanian => "Lietuvių",
            Language::Latvian => "Latviešu",
            Language::Estonian => "Eesti",
            Language::Georgian => "ქართული",
            Language::Armenian => "Հայերեն",
            Language::Azerbaijani => "Azərbaycan",
            Language::Uzbek => "Oʻzbekcha",
            Language::Tajik => "Тоҷикӣ",
            Language::Kyrgyz => "Кыргызча",
            Language::Turkmen => "Türkmençe",
            Language::Mongolian => "Монгол",
            Language::Tagalog => "Tagalog",
            Language::Albanian => "Shqip",
            Language::Bosnian => "Bosanski",
            Language::Macedonian => "Македонски",
            Language::Icelandic => "Íslenska",
            Language::Irish => "Gaeilge",
            Language::Welsh => "Cymraeg",
            Language::Basque => "Euskara",
            Language::Catalan => "Català",
            Language::Galician => "Galego",
            Language::Afrikaans => "Afrikaans",
            Language::Swahili => "Kiswahili",
            Language::Hausa => "Hausa",
            Language::Yoruba => "Yorùbá",
            Language::Igbo => "Igbo",
            Language::Zulu => "isiZulu",
            Language::Esperanto => "Esperanto",
            Language::Latin => "Latina",
            Language::Tamil => "தமிழ்",
            Language::Telugu => "తెలుగు",
            Language::Marathi => "मराठी",
            Language::Gujarati => "ગુજરાતી",
            Language::Kannada => "ಕನ್ನಡ",
            Language::Malayalam => "മലയാളം",
            Language::Punjabi => "ਪੰਜਾਬੀ",
            Language::Nepali => "नेपाली",
            Language::Sinhala => "සිංහල",
            Language::Burmese => "မြန်မာ",
            Language::Khmer => "ភាសាខ្មែរ",
            Language::Amharic => "አማርኛ",
            Language::Somali => "Soomaali",
            Language::Kurdish => "Kurdî",
            Language::Maltese => "Malti",
            Language::Korean => "한국어",
            Language::Japanese => "日本語",
            Language::ChineseSimplified => "中文 (简体)",
            Language::ChineseTraditional => "中文 (繁體)",
            Language::Cantonese => "粵語 (香港)",
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Language::Russian => "RU",
            Language::English => "EN",
            Language::German => "DE",
            Language::French => "FR",
            Language::Spanish => "ES",
            Language::Portuguese => "PT",
            Language::Italian => "IT",
            Language::Turkish => "TR",
            Language::Ukrainian => "UK",
            Language::Belarusian => "BE",
            Language::Kazakh => "KK",
            Language::Arabic => "AR",
            Language::Polish => "PL",
            Language::Czech => "CS",
            Language::Romanian => "RO",
            Language::Dutch => "NL",
            Language::Swedish => "SV",
            Language::Norwegian => "NO",
            Language::Danish => "DA",
            Language::Finnish => "FI",
            Language::Greek => "EL",
            Language::Hebrew => "HE",
            Language::Persian => "FA",
            Language::Urdu => "UR",
            Language::Hindi => "HI",
            Language::Bengali => "BN",
            Language::Indonesian => "ID",
            Language::Malay => "MS",
            Language::Vietnamese => "VI",
            Language::Thai => "TH",
            Language::Hungarian => "HU",
            Language::Bulgarian => "BG",
            Language::Serbian => "SR",
            Language::Croatian => "HR",
            Language::Slovak => "SK",
            Language::Slovenian => "SL",
            Language::Lithuanian => "LT",
            Language::Latvian => "LV",
            Language::Estonian => "ET",
            Language::Georgian => "KA",
            Language::Armenian => "HY",
            Language::Azerbaijani => "AZ",
            Language::Uzbek => "UZ",
            Language::Tajik => "TG",
            Language::Kyrgyz => "KY",
            Language::Turkmen => "TK",
            Language::Mongolian => "MN",
            Language::Tagalog => "TL",
            Language::Albanian => "SQ",
            Language::Bosnian => "BS",
            Language::Macedonian => "MK",
            Language::Icelandic => "IS",
            Language::Irish => "GA",
            Language::Welsh => "CY",
            Language::Basque => "EU",
            Language::Catalan => "CA",
            Language::Galician => "GL",
            Language::Afrikaans => "AF",
            Language::Swahili => "SW",
            Language::Hausa => "HA",
            Language::Yoruba => "YO",
            Language::Igbo => "IG",
            Language::Zulu => "ZU",
            Language::Esperanto => "EO",
            Language::Latin => "LA",
            Language::Tamil => "TA",
            Language::Telugu => "TE",
            Language::Marathi => "MR",
            Language::Gujarati => "GU",
            Language::Kannada => "KN",
            Language::Malayalam => "ML",
            Language::Punjabi => "PA",
            Language::Nepali => "NE",
            Language::Sinhala => "SI",
            Language::Burmese => "MY",
            Language::Khmer => "KM",
            Language::Amharic => "AM",
            Language::Somali => "SO",
            Language::Kurdish => "KU",
            Language::Maltese => "MT",
            Language::Korean => "KO",
            Language::Japanese => "JA",
            Language::ChineseSimplified => "ZH-CN",
            Language::ChineseTraditional => "ZH-TW",
            Language::Cantonese => "ZH-HK",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::Russian => "ru",
            Language::English => "en",
            Language::German => "de",
            Language::French => "fr",
            Language::Spanish => "es",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Turkish => "tr",
            Language::Ukrainian => "uk",
            Language::Belarusian => "be",
            Language::Kazakh => "kk",
            Language::Arabic => "ar",
            Language::Polish => "pl",
            Language::Czech => "cs",
            Language::Romanian => "ro",
            Language::Dutch => "nl",
            Language::Swedish => "sv",
            Language::Norwegian => "no",
            Language::Danish => "da",
            Language::Finnish => "fi",
            Language::Greek => "el",
            Language::Hebrew => "he",
            Language::Persian => "fa",
            Language::Urdu => "ur",
            Language::Hindi => "hi",
            Language::Bengali => "bn",
            Language::Indonesian => "id",
            Language::Malay => "ms",
            Language::Vietnamese => "vi",
            Language::Thai => "th",
            Language::Hungarian => "hu",
            Language::Bulgarian => "bg",
            Language::Serbian => "sr",
            Language::Croatian => "hr",
            Language::Slovak => "sk",
            Language::Slovenian => "sl",
            Language::Lithuanian => "lt",
            Language::Latvian => "lv",
            Language::Estonian => "et",
            Language::Georgian => "ka",
            Language::Armenian => "hy",
            Language::Azerbaijani => "az",
            Language::Uzbek => "uz",
            Language::Tajik => "tg",
            Language::Kyrgyz => "ky",
            Language::Turkmen => "tk",
            Language::Mongolian => "mn",
            Language::Tagalog => "tl",
            Language::Albanian => "sq",
            Language::Bosnian => "bs",
            Language::Macedonian => "mk",
            Language::Icelandic => "is",
            Language::Irish => "ga",
            Language::Welsh => "cy",
            Language::Basque => "eu",
            Language::Catalan => "ca",
            Language::Galician => "gl",
            Language::Afrikaans => "af",
            Language::Swahili => "sw",
            Language::Hausa => "ha",
            Language::Yoruba => "yo",
            Language::Igbo => "ig",
            Language::Zulu => "zu",
            Language::Esperanto => "eo",
            Language::Latin => "la",
            Language::Tamil => "ta",
            Language::Telugu => "te",
            Language::Marathi => "mr",
            Language::Gujarati => "gu",
            Language::Kannada => "kn",
            Language::Malayalam => "ml",
            Language::Punjabi => "pa",
            Language::Nepali => "ne",
            Language::Sinhala => "si",
            Language::Burmese => "my",
            Language::Khmer => "km",
            Language::Amharic => "am",
            Language::Somali => "so",
            Language::Kurdish => "ku",
            Language::Maltese => "mt",
            Language::Korean => "ko",
            Language::Japanese => "ja",
            Language::ChineseSimplified => "zh_cn",
            Language::ChineseTraditional => "zh_tw",
            Language::Cantonese => "zh_hk",
        }
    }

    pub fn space_label(&self) -> &'static str {
        match self {
            Language::Russian => "пробел",
            Language::English => "space",
            Language::German => "Leertaste",
            Language::French => "espace",
            Language::Spanish => "espacio",
            Language::Portuguese => "espaço",
            Language::Italian => "spazio",
            Language::Turkish => "boşluk",
            Language::Ukrainian => "пробіл",
            Language::Belarusian => "прабел",
            Language::Kazakh => "бос орын",
            Language::Arabic => "مسافة",
            Language::Polish => "spacja",
            Language::Czech => "mezerník",
            Language::Romanian => "spațiu",
            Language::Dutch => "spatie",
            Language::Swedish => "mellanslag",
            Language::Norwegian => "mellomrom",
            Language::Danish => "mellemrum",
            Language::Finnish => "välilyönti",
            Language::Greek => "διάστημα",
            Language::Hebrew => "רווח",
            Language::Persian => "فاصله",
            Language::Urdu => "فاصلہ",
            Language::Hindi => "स्पेस",
            Language::Bengali => "স্পेस",
            Language::Indonesian => "spasi",
            Language::Malay => "jarak",
            Language::Vietnamese => "dấu cách",
            Language::Thai => "วรรค",
            Language::Hungarian => "szóköz",
            Language::Bulgarian => "интервал",
            Language::Serbian => "размак",
            Language::Croatian => "razmak",
            Language::Slovak => "medzerník",
            Language::Slovenian => "preslednica",
            Language::Lithuanian => "tarpas",
            Language::Latvian => "atstarpe",
            Language::Estonian => "tühik",
            Language::Georgian => "ჰარი",
            Language::Armenian => "բացատ",
            Language::Azerbaijani => "boşluq",
            Language::Uzbek => "boʻsh joy",
            Language::Tajik => "фосила",
            Language::Kyrgyz => "боштук",
            Language::Turkmen => "boşluk",
            Language::Mongolian => "зай",
            Language::Tagalog => "puwang",
            Language::Albanian => "hapësirë",
            Language::Bosnian => "razmak",
            Language::Macedonian => "проред",
            Language::Icelandic => "bil",
            Language::Irish => "spás",
            Language::Welsh => "bwlch",
            Language::Basque => "zuriunea",
            Language::Catalan => "espai",
            Language::Galician => "espazo",
            Language::Afrikaans => "spasie",
            Language::Swahili => "nafasi",
            Language::Hausa => "fili",
            Language::Yoruba => "ààyè",
            Language::Igbo => "oghere",
            Language::Zulu => "isikhala",
            Language::Esperanto => "spaceto",
            Language::Latin => "spatium",
            Language::Tamil => "இடைவெளி",
            Language::Telugu => "స్పేస్",
            Language::Marathi => "स्पेस",
            Language::Gujarati => "જગ્યા",
            Language::Kannada => "ಸ್ಪೇಸ್",
            Language::Malayalam => "സ്പേസ്",
            Language::Punjabi => "ਸਪੇਸ",
            Language::Nepali => "खाली ठाउँ",
            Language::Sinhala => "හිස්තැන",
            Language::Burmese => "နေရာလွတ်",
            Language::Khmer => "ដកឃ្លា",
            Language::Amharic => "ክፍተት",
            Language::Somali => "meel",
            Language::Kurdish => "valahî",
            Language::Maltese => "spazju",
            Language::Korean => "간격",
            Language::Japanese => "空白",
            Language::ChineseSimplified => "空格",
            Language::ChineseTraditional => "空格",
            Language::Cantonese => "空格",
        }
    }

    pub fn to_id(&self) -> i32 {
        match self {
            Language::Russian => 0,
            Language::English => 1,
            Language::German => 2,
            Language::French => 3,
            Language::Spanish => 4,
            Language::Portuguese => 5,
            Language::Italian => 6,
            Language::Turkish => 7,
            Language::Ukrainian => 8,
            Language::Belarusian => 9,
            Language::Kazakh => 10,
            Language::Arabic => 11,
            Language::Polish => 12,
            Language::Czech => 13,
            Language::Romanian => 14,
            Language::Dutch => 15,
            Language::Swedish => 16,
            Language::Norwegian => 17,
            Language::Danish => 18,
            Language::Finnish => 19,
            Language::Greek => 20,
            Language::Hebrew => 21,
            Language::Persian => 22,
            Language::Urdu => 23,
            Language::Hindi => 24,
            Language::Bengali => 25,
            Language::Indonesian => 26,
            Language::Malay => 27,
            Language::Vietnamese => 28,
            Language::Thai => 29,
            Language::Hungarian => 30,
            Language::Bulgarian => 31,
            Language::Serbian => 32,
            Language::Croatian => 33,
            Language::Slovak => 34,
            Language::Slovenian => 35,
            Language::Lithuanian => 36,
            Language::Latvian => 37,
            Language::Estonian => 38,
            Language::Georgian => 39,
            Language::Armenian => 40,
            Language::Azerbaijani => 41,
            Language::Uzbek => 42,
            Language::Tajik => 43,
            Language::Kyrgyz => 44,
            Language::Turkmen => 45,
            Language::Mongolian => 46,
            Language::Tagalog => 47,
            Language::Albanian => 48,
            Language::Bosnian => 49,
            Language::Macedonian => 50,
            Language::Icelandic => 51,
            Language::Irish => 52,
            Language::Welsh => 53,
            Language::Basque => 54,
            Language::Catalan => 55,
            Language::Galician => 56,
            Language::Afrikaans => 57,
            Language::Swahili => 58,
            Language::Hausa => 59,
            Language::Yoruba => 60,
            Language::Igbo => 61,
            Language::Zulu => 62,
            Language::Esperanto => 63,
            Language::Latin => 64,
            Language::Tamil => 65,
            Language::Telugu => 66,
            Language::Marathi => 67,
            Language::Gujarati => 68,
            Language::Kannada => 69,
            Language::Malayalam => 70,
            Language::Punjabi => 71,
            Language::Nepali => 72,
            Language::Sinhala => 73,
            Language::Burmese => 74,
            Language::Khmer => 75,
            Language::Amharic => 76,
            Language::Somali => 77,
            Language::Kurdish => 78,
            Language::Maltese => 79,
            Language::Korean => 80,
            Language::Japanese => 81,
            Language::ChineseSimplified => 82,
            Language::ChineseTraditional => 83,
            Language::Cantonese => 84,
        }
    }

    pub fn from_id(id: i32) -> Self {
        match id {
            0 => Language::Russian,
            1 => Language::English,
            2 => Language::German,
            3 => Language::French,
            4 => Language::Spanish,
            5 => Language::Portuguese,
            6 => Language::Italian,
            7 => Language::Turkish,
            8 => Language::Ukrainian,
            9 => Language::Belarusian,
            10 => Language::Kazakh,
            11 => Language::Arabic,
            12 => Language::Polish,
            13 => Language::Czech,
            14 => Language::Romanian,
            15 => Language::Dutch,
            16 => Language::Swedish,
            17 => Language::Norwegian,
            18 => Language::Danish,
            19 => Language::Finnish,
            20 => Language::Greek,
            21 => Language::Hebrew,
            22 => Language::Persian,
            23 => Language::Urdu,
            24 => Language::Hindi,
            25 => Language::Bengali,
            26 => Language::Indonesian,
            27 => Language::Malay,
            28 => Language::Vietnamese,
            29 => Language::Thai,
            30 => Language::Hungarian,
            31 => Language::Bulgarian,
            32 => Language::Serbian,
            33 => Language::Croatian,
            34 => Language::Slovak,
            35 => Language::Slovenian,
            36 => Language::Lithuanian,
            37 => Language::Latvian,
            38 => Language::Estonian,
            39 => Language::Georgian,
            40 => Language::Armenian,
            41 => Language::Azerbaijani,
            42 => Language::Uzbek,
            43 => Language::Tajik,
            44 => Language::Kyrgyz,
            45 => Language::Turkmen,
            46 => Language::Mongolian,
            47 => Language::Tagalog,
            48 => Language::Albanian,
            49 => Language::Bosnian,
            50 => Language::Macedonian,
            51 => Language::Icelandic,
            52 => Language::Irish,
            53 => Language::Welsh,
            54 => Language::Basque,
            55 => Language::Catalan,
            56 => Language::Galician,
            57 => Language::Afrikaans,
            58 => Language::Swahili,
            59 => Language::Hausa,
            60 => Language::Yoruba,
            61 => Language::Igbo,
            62 => Language::Zulu,
            63 => Language::Esperanto,
            64 => Language::Latin,
            65 => Language::Tamil,
            66 => Language::Telugu,
            67 => Language::Marathi,
            68 => Language::Gujarati,
            69 => Language::Kannada,
            70 => Language::Malayalam,
            71 => Language::Punjabi,
            72 => Language::Nepali,
            73 => Language::Sinhala,
            74 => Language::Burmese,
            75 => Language::Khmer,
            76 => Language::Amharic,
            77 => Language::Somali,
            78 => Language::Kurdish,
            79 => Language::Maltese,
            80 => Language::Korean,
            81 => Language::Japanese,
            82 => Language::ChineseSimplified,
            83 => Language::ChineseTraditional,
            84 => Language::Cantonese,
            _ => Language::Russian,
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_lowercase().as_str() {
            "ru" => Some(Language::Russian),
            "en" => Some(Language::English),
            "de" => Some(Language::German),
            "fr" => Some(Language::French),
            "es" => Some(Language::Spanish),
            "pt" => Some(Language::Portuguese),
            "it" => Some(Language::Italian),
            "tr" => Some(Language::Turkish),
            "uk" => Some(Language::Ukrainian),
            "be" => Some(Language::Belarusian),
            "kk" => Some(Language::Kazakh),
            "ar" => Some(Language::Arabic),
            "pl" => Some(Language::Polish),
            "cs" => Some(Language::Czech),
            "ro" => Some(Language::Romanian),
            "nl" => Some(Language::Dutch),
            "sv" => Some(Language::Swedish),
            "no" => Some(Language::Norwegian),
            "da" => Some(Language::Danish),
            "fi" => Some(Language::Finnish),
            "el" => Some(Language::Greek),
            "he" => Some(Language::Hebrew),
            "fa" => Some(Language::Persian),
            "ur" => Some(Language::Urdu),
            "hi" => Some(Language::Hindi),
            "bn" => Some(Language::Bengali),
            "id" => Some(Language::Indonesian),
            "ms" => Some(Language::Malay),
            "vi" => Some(Language::Vietnamese),
            "th" => Some(Language::Thai),
            "hu" => Some(Language::Hungarian),
            "bg" => Some(Language::Bulgarian),
            "sr" => Some(Language::Serbian),
            "hr" => Some(Language::Croatian),
            "sk" => Some(Language::Slovak),
            "sl" => Some(Language::Slovenian),
            "lt" => Some(Language::Lithuanian),
            "lv" => Some(Language::Latvian),
            "et" => Some(Language::Estonian),
            "ka" => Some(Language::Georgian),
            "hy" => Some(Language::Armenian),
            "az" => Some(Language::Azerbaijani),
            "uz" => Some(Language::Uzbek),
            "tg" => Some(Language::Tajik),
            "ky" => Some(Language::Kyrgyz),
            "tk" => Some(Language::Turkmen),
            "mn" => Some(Language::Mongolian),
            "tl" => Some(Language::Tagalog),
            "sq" => Some(Language::Albanian),
            "bs" => Some(Language::Bosnian),
            "mk" => Some(Language::Macedonian),
            "is" => Some(Language::Icelandic),
            "ga" => Some(Language::Irish),
            "cy" => Some(Language::Welsh),
            "eu" => Some(Language::Basque),
            "ca" => Some(Language::Catalan),
            "gl" => Some(Language::Galician),
            "af" => Some(Language::Afrikaans),
            "sw" => Some(Language::Swahili),
            "ha" => Some(Language::Hausa),
            "yo" => Some(Language::Yoruba),
            "ig" => Some(Language::Igbo),
            "zu" => Some(Language::Zulu),
            "eo" => Some(Language::Esperanto),
            "la" => Some(Language::Latin),
            "ta" => Some(Language::Tamil),
            "te" => Some(Language::Telugu),
            "mr" => Some(Language::Marathi),
            "gu" => Some(Language::Gujarati),
            "kn" => Some(Language::Kannada),
            "ml" => Some(Language::Malayalam),
            "pa" => Some(Language::Punjabi),
            "ne" => Some(Language::Nepali),
            "si" => Some(Language::Sinhala),
            "my" => Some(Language::Burmese),
            "km" => Some(Language::Khmer),
            "am" => Some(Language::Amharic),
            "so" => Some(Language::Somali),
            "ku" => Some(Language::Kurdish),
            "mt" => Some(Language::Maltese),
            "ko" => Some(Language::Korean),
            "ja" => Some(Language::Japanese),
            "zh" | "zh_cn" | "zh-cn" | "zh-hans" => Some(Language::ChineseSimplified),
            "zh_tw" | "zh-tw" | "zh-hant" => Some(Language::ChineseTraditional),
            "zh_hk" | "zh-hk" | "yue" => Some(Language::Cantonese),
            _ => None,
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Language::Russian => Language::English,
            Language::English => Language::Russian,
            other => *other,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum InputFieldMode {
    #[default]
    Normal = 0,
    Password = 1,
    VisiblePassword = 2,
    Email = 3,
    Uri = 4,
    Number = 5,
    Phone = 6,
    Date = 7,
    Time = 8,
    Multiline = 9,
}

impl InputFieldMode {
    pub fn from_id(id: i32) -> Self {
        match id {
            1 => InputFieldMode::Password,
            2 => InputFieldMode::VisiblePassword,
            3 => InputFieldMode::Email,
            4 => InputFieldMode::Uri,
            5 => InputFieldMode::Number,
            6 => InputFieldMode::Phone,
            7 => InputFieldMode::Date,
            8 => InputFieldMode::Time,
            9 => InputFieldMode::Multiline,
            _ => InputFieldMode::Normal,
        }
    }

    #[inline]
    pub fn is_password(&self) -> bool {
        matches!(
            self,
            InputFieldMode::Password | InputFieldMode::VisiblePassword
        )
    }

    #[inline]
    pub fn allows_suggestions(&self) -> bool {
        !self.is_password() && *self != InputFieldMode::Number && *self != InputFieldMode::Phone
    }

    #[inline]
    pub fn allows_autocorrect(&self) -> bool {
        !self.is_password()
            && *self != InputFieldMode::Email
            && *self != InputFieldMode::Uri
            && *self != InputFieldMode::Number
            && *self != InputFieldMode::Phone
    }

    #[inline]
    pub fn allows_learning(&self) -> bool {
        !self.is_password()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum KeyboardOutputEvent {
    CommitText(String),
    DeleteSurroundingText { before: u32, after: u32 },
    SendKeyEvent(i32),
    PerformHaptic(HapticFeedbackType),
    SwitchInputMethod,
    OpenSettings,
    MoveCursor(i32),
    DeleteWord,
    HideKeyboard,
    ClearClipboard,
    ClipboardPasted(String),
}

impl KeyboardOutputEvent {
    pub fn write_to_binary(&self, buf: &mut Vec<u8>) {
        match self {
            KeyboardOutputEvent::CommitText(text) => {
                buf.push(1); // Type 1: CommitText
                let bytes = text.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(bytes);
            }
            KeyboardOutputEvent::DeleteSurroundingText { before, after } => {
                buf.push(2); // Type 2: DeleteSurroundingText
                buf.extend_from_slice(&8u32.to_le_bytes());
                buf.extend_from_slice(&before.to_le_bytes());
                buf.extend_from_slice(&after.to_le_bytes());
            }
            KeyboardOutputEvent::SendKeyEvent(code) => {
                buf.push(3); // Type 3: SendKeyEvent
                buf.extend_from_slice(&4u32.to_le_bytes());
                buf.extend_from_slice(&code.to_le_bytes());
            }
            KeyboardOutputEvent::PerformHaptic(haptic) => {
                buf.push(4); // Type 4: PerformHaptic
                buf.extend_from_slice(&1u32.to_le_bytes());
                let h = match haptic {
                    HapticFeedbackType::KeyTick => 0u8,
                    HapticFeedbackType::KeyClick => 1u8,
                    HapticFeedbackType::KeyHeavyClick => 2u8,
                    HapticFeedbackType::LongPress => 3u8,
                };
                buf.push(h);
            }
            KeyboardOutputEvent::MoveCursor(delta) => {
                buf.push(5); // Type 5: MoveCursor
                buf.extend_from_slice(&4u32.to_le_bytes());
                buf.extend_from_slice(&delta.to_le_bytes());
            }
            KeyboardOutputEvent::DeleteWord => {
                buf.push(6); // Type 6: DeleteWord
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            KeyboardOutputEvent::OpenSettings => {
                buf.push(7); // Type 7: OpenSettings
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            KeyboardOutputEvent::SwitchInputMethod => {
                buf.push(8); // Type 8: SwitchInputMethod
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            KeyboardOutputEvent::HideKeyboard => {
                buf.push(9); // Type 9: HideKeyboard
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            KeyboardOutputEvent::ClearClipboard => {
                buf.push(10); // Type 10: ClearClipboard
                buf.extend_from_slice(&0u32.to_le_bytes());
            }
            KeyboardOutputEvent::ClipboardPasted(text) => {
                buf.push(11); // Type 11: ClipboardPasted
                let bytes = text.as_bytes();
                buf.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
                buf.extend_from_slice(bytes);
            }
        }
    }
}

pub fn serialize_events_binary(events: &[KeyboardOutputEvent]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + events.len() * 16);
    buf.extend_from_slice(&(events.len() as u32).to_le_bytes());
    for event in events {
        event.write_to_binary(&mut buf);
    }
    buf
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HapticFeedbackType {
    KeyTick,
    KeyClick,
    KeyHeavyClick,
    LongPress,
}

#[derive(Clone, Debug)]
pub struct KeyboardState {
    pub language: Language,
    pub mode: KeyboardMode,
    pub shift_state: ShiftState,
    pub field_mode: InputFieldMode,
    pub composing_text: String,
    pub last_committed_word: String,
    pub last_autocorrect_original: Option<String>,
    pub last_autocorrect_replacement: Option<String>,
    pub rejected_autocorrect_word: Option<String>,
    pub last_char_was_space: bool,
    pub clipboard_preview: Option<String>,
    pub output_events: Vec<KeyboardOutputEvent>,
    pub space_swipe_dx: f32,
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self {
            language: Language::Russian,
            mode: KeyboardMode::Alphabet,
            shift_state: ShiftState::Off,
            field_mode: InputFieldMode::Normal,
            composing_text: String::with_capacity(64),
            last_committed_word: String::with_capacity(32),
            last_autocorrect_original: None,
            last_autocorrect_replacement: None,
            rejected_autocorrect_word: None,
            last_char_was_space: false,
            clipboard_preview: None,
            output_events: Vec::with_capacity(16),
            space_swipe_dx: 0.0,
        }
    }
}

impl KeyboardState {
    pub fn push_event(&mut self, event: KeyboardOutputEvent) {
        self.output_events.push(event);
    }

    pub fn drain_events(&mut self) -> Vec<KeyboardOutputEvent> {
        std::mem::take(&mut self.output_events)
    }
}
