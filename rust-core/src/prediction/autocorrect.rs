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

    pub fn get_adjacent_chars(c: char) -> &'static str {
        let n = Self::strip_diacritics(c);
        match n {
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
            // Cyrillic ЙЦУКЕН + phonetics
            'й' => "цфи",
            'ц' => "йуыфч",
            'у' => "цквы",
            'к' => "уеавг",
            'е' => "кнпаиё",
            'ё' => "кнпаие",
            'н' => "егрп",
            'г' => "ншорк",
            'ш' => "гщлож",
            'щ' => "шздл",
            'з' => "щхждс",
            'х' => "зэж",
            'ф' => "йцыяв",
            'ы' => "цуфвячи",
            'в' => "укыачсф",
            'а' => "кевпсмо",
            'п' => "енаримб",
            'р' => "нгпоит",
            'о' => "гшрлта",
            'л' => "шщодьб",
            'д' => "щзлжбют",
            'ж' => "зхдэюш",
            'э' => "хж",
            'я' => "фыч",
            'ч' => "яывсц",
            'с' => "чвамз",
            'м' => "сапи",
            'и' => "мпротеы",
            'т' => "ироьд",
            'ь' => "толбъ",
            'ъ' => "толбь",
            'б' => "ьлдюп",
            'ю' => "бдж",
            _ => "",
        }
    }

    /// Checks if two characters are adjacent neighbors on QWERTY or ЙЦУКЕН layouts
    pub fn is_layout_neighbor(c1: char, c2: char) -> bool {
        let n1 = Self::strip_diacritics(c1);
        let n2 = Self::strip_diacritics(c2);
        if n1 == n2 {
            return true;
        }
        Self::get_adjacent_chars(n1).contains(n2)
    }

    #[inline]
    pub fn is_phonetic_confusion(c1: char, c2: char) -> bool {
        let (a, b) = if c1 < c2 { (c1, c2) } else { (c2, c1) };
        matches!(
            (a, b),
            ('а', 'о')
                | ('е', 'и')
                | ('з', 'с')
                | ('д', 'т')
                | ('б', 'п')
                | ('в', 'ф')
                | ('г', 'к')
                | ('ж', 'ш')
                | ('ё', 'е')
                | ('и', 'й')
                | ('ц', 'ч')
                | ('ь', 'ъ')
        )
    }
}
