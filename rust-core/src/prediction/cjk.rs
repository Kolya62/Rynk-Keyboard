//! CJK Candidate Engine
//!
//! Provides Pinyin syllable-to-Hanzi candidate lookup for Chinese (Simplified, Traditional, Cantonese)
//! and Romaji-to-Kana/Kanji candidate lookup for Japanese.

use crate::keyboard::state::Language;

pub fn get_cjk_candidates(input: &str, lang: Language) -> Vec<String> {
    let clean = input.trim().to_lowercase();
    if clean.is_empty() {
        return Vec::new();
    }

    match lang {
        Language::ChineseSimplified => get_chinese_candidates(&clean, false),
        Language::ChineseTraditional | Language::Cantonese => get_chinese_candidates(&clean, true),
        Language::Japanese => get_japanese_candidates(&clean),
        _ => Vec::new(),
    }
}

fn get_chinese_candidates(input: &str, traditional: bool) -> Vec<String> {
    let table: &[(&str, &[&str], &[&str])] = &[
        ("ni", &["你", "泥", "拟", "妮", "呢"], &["你", "泥", "擬", "妮", "呢"]),
        ("hao", &["好", "号", "毫", "浩", "耗"], &["好", "號", "毫", "浩", "耗"]),
        ("nihao", &["你好"], &["你好"]),
        ("wo", &["我", "握", "窝", "卧"], &["我", "握", "窩", "臥"]),
        ("men", &["们", "门", "闷"], &["們", "門", "悶"]),
        ("women", &["我们"], &["我們"]),
        ("ta", &["他", "她", "它"], &["他", "她", "它"]),
        ("tamen", &["他们", "她们"], &["他們", "她們"]),
        ("shi", &["是", "时", "事", "十", "使"], &["是", "時", "事", "十", "使"]),
        ("de", &["的", "得", "地"], &["的", "得", "地"]),
        ("le", &["了", "乐"], &["了", "樂"]),
        ("zai", &["在", "再"], &["在", "再"]),
        ("you", &["有", "又", "右", "友"], &["有", "又", "右", "友"]),
        ("zhe", &["这", "着"], &["這", "著"]),
        ("ge", &["个", "各", "歌"], &["個", "各", "歌"]),
        ("zhong", &["中", "重", "种"], &["中", "重", "種"]),
        ("guo", &["国", "过", "果"], &["國", "過", "果"]),
        ("zhongguo", &["中国"], &["中國"]),
        ("ren", &["人", "任", "认"], &["人", "任", "認"]),
        ("da", &["大", "打", "达"], &["大", "打", "達"]),
        ("xiao", &["小", "笑", "消"], &["小", "笑", "消"]),
        ("bu", &["不", "部", "步"], &["不", "部", "步"]),
        ("shang", &["上", "商", "伤"], &["上", "商", "傷"]),
        ("xia", &["下", "夏", "吓"], &["下", "夏", "嚇"]),
        ("xie", &["谢", "写", "些"], &["謝", "寫", "些"]),
        ("xiexie", &["谢谢"], &["謝謝"]),
        ("jian", &["见", "间", "件"], &["見", "間", "件"]),
        ("zaijian", &["再见"], &["再見"]),
        ("ke", &["可", "克", "刻"], &["可", "克", "刻"]),
        ("qi", &["气", "起", "其"], &["氣", "起", "其"]),
        ("bukeqi", &["不客气"], &["不客氣"]),
        ("ma", &["吗", "妈", "马"], &["嗎", "媽", "馬"]),
        ("shen", &["什", "身", "深"], &["什", "身", "深"]),
        ("me", &["么", "没"], &["麼", "沒"]),
        ("shenme", &["什么"], &["什麼"]),
        ("ai", &["爱", "哀", "矮"], &["愛", "哀", "矮"]),
        ("tian", &["天", "田", "填"], &["天", "田", "填"]),
        ("di", &["地", "第", "低"], &["地", "第", "低"]),
        ("dian", &["点", "电", "店"], &["點", "電", "店"]),
        ("hua", &["话", "花", "画"], &["話", "花", "畫"]),
        ("dianhua", &["电话"], &["電話"]),
        ("shui", &["水", "谁", "睡"], &["水", "誰", "睡"]),
        ("chi", &["吃", "持", "池"], &["吃", "持", "池"]),
        ("fan", &["饭", "反", "范"], &["飯", "反", "範"]),
        ("chifan", &["吃饭"], &["吃飯"]),
        ("kan", &["看", "刊", "砍"], &["看", "刊", "砍"]),
        ("shu", &["书", "树", "数"], &["書", "樹", "數"]),
        ("kanshu", &["看书"], &["看書"]),
        ("shuo", &["说"], &["說"]),
        ("ting", &["听", "停", "厅"], &["聽", "停", "廳"]),
        ("xue", &["学", "雪", "血"], &["學", "雪", "血"]),
        ("xiao", &["校", "笑", "小"], &["校", "笑", "小"]),
        ("xuexiao", &["学校"], &["學校"]),
        ("sui", &["岁", "随", "碎"], &["歲", "隨", "碎"]),
        ("nian", &["年", "念"], &["年", "念"]),
        ("yue", &["月", "越", "约"], &["月", "越", "約"]),
        ("ri", &["日", "入"], &["日", "入"]),
        ("ming", &["明", "名", "命"], &["明", "名", "命"]),
        ("mingtian", &["明天"], &["明天"]),
        ("zuotian", &["昨天"], &["昨天"]),
        ("jintian", &["今天"], &["今天"]),
    ];

    for &(pinyin, simp_cands, trad_cands) in table {
        if pinyin == input {
            let cands = if traditional { trad_cands } else { simp_cands };
            return cands.iter().map(|&s| s.to_string()).collect();
        }
    }

    // Prefix search if not exact match
    for &(pinyin, simp_cands, trad_cands) in table {
        if pinyin.starts_with(input) {
            let cands = if traditional { trad_cands } else { simp_cands };
            return cands.iter().map(|&s| s.to_string()).collect();
        }
    }

    Vec::new()
}

fn get_japanese_candidates(input: &str) -> Vec<String> {
    let table: &[(&str, &[&str])] = &[
        ("a", &["あ", "ア"]),
        ("i", &["い", "イ"]),
        ("u", &["う", "ウ"]),
        ("e", &["え", "エ"]),
        ("o", &["お", "オ"]),
        ("ka", &["か", "カ", "日", "科", "化"]),
        ("ki", &["き", "キ", "気", "木", "機"]),
        ("ku", &["く", "ク", "苦", "区"]),
        ("ke", &["け", "ケ"]),
        ("ko", &["こ", "コ", "子", "小"]),
        ("sa", &["さ", "サ"]),
        ("shi", &["し", "シ", "市", "四", "死"]),
        ("su", &["す", "ス", "巣"]),
        ("se", &["せ", "セ", "世"]),
        ("so", &["そ", "ソ"]),
        ("ta", &["た", "タ", "田", "多"]),
        ("chi", &["ち", "チ", "地", "知", "血"]),
        ("tsu", &["つ", "ツ"]),
        ("te", &["て", "テ", "手"]),
        ("to", &["と", "ト", "戸"]),
        ("na", &["な", "ナ", "名", "菜"]),
        ("ni", &["に", "ニ", "二"]),
        ("nu", &["ぬ", "ヌ"]),
        ("ne", &["ね", "ネ", "根"]),
        ("no", &["の", "ノ", "野"]),
        ("ha", &["は", "ハ", "歯", "葉"]),
        ("hi", &["ひ", "ヒ", "日", "火"]),
        ("fu", &["ふ", "フ", "不"]),
        ("he", &["へ", "ヘ"]),
        ("ho", &["ほ", "ホ", "保"]),
        ("ma", &["ま", "マ", "真", "間"]),
        ("mi", &["み", "ミ", "見", "身"]),
        ("mu", &["む", "ム", "無"]),
        ("me", &["め", "メ", "目"]),
        ("mo", &["も", "モ", "毛"]),
        ("ya", &["や", "ヤ", "夜", "野"]),
        ("yu", &["ゆ", "ユ", "湯"]),
        ("yo", &["よ", "ヨ", "代"]),
        ("ra", &["ら", "ラ"]),
        ("ri", &["り", "リ", "理", "里"]),
        ("ru", &["る", "ル"]),
        ("re", &["れ", "レ", "礼"]),
        ("ro", &["ろ", "ロ", "路"]),
        ("wa", &["わ", "ワ", "輪", "和"]),
        ("wo", &["を", "ヲ"]),
        ("nn", &["ん", "ン"]),
        ("ga", &["が", "ガ"]),
        ("gi", &["ぎ", "ギ"]),
        ("gu", &["ぐ", "グ"]),
        ("ge", &["げ", "ゲ"]),
        ("go", &["ご", "ゴ", "五", "語"]),
        ("za", &["ざ", "ザ"]),
        ("ji", &["じ", "ジ", "字", "事", "時"]),
        ("zu", &["ず", "ズ", "図"]),
        ("ze", &["ぜ", "ゼ"]),
        ("zo", &["ぞ", "ゾ"]),
        ("da", &["だ", "ダ"]),
        ("de", &["で", "デ"]),
        ("do", &["ど", "ド"]),
        ("ba", &["ば", "バ", "場"]),
        ("bi", &["び", "ビ", "美"]),
        ("bu", &["ぶ", "ブ", "部"]),
        ("be", &["べ", "ベ"]),
        ("bo", &["ぼ", "ボ"]),
        ("pa", &["ぱ", "パ"]),
        ("pi", &["ぴ", "ピ"]),
        ("pu", &["ぷ", "プ"]),
        ("pe", &["ぺ", "ペ"]),
        ("po", &["ぽ", "ポ"]),
        ("arigatou", &["ありがとう", "有難う"]),
        ("konnichiwa", &["こんにちは", "今日波"]),
        ("nihon", &["日本", "にほん", "ニホン"]),
        ("watashi", &["私", "わたし", "ワタシ"]),
        ("desu", &["です", "デス"]),
        ("hai", &["はい", "ハイ"]),
        ("iie", &["いいえ"]),
        ("ohayou", &["おはよう", "お早う"]),
        ("sayounara", &["さようなら", "左様なら"]),
        ("sumimasen", &["すみません"]),
        ("gomen", &["ごめん", "御免"]),
        ("daisuki", &["大好き", "だいすき"]),
        ("sushi", &["寿司", "すし"]),
        ("tokyo", &["東京", "とうきょう"]),
        ("anata", &["あなた", "貴方"]),
        ("kore", &["これ", "此れ"]),
        ("sore", &["それ", "其れ"]),
        ("are", &["あれ", "彼れ"]),
        ("doko", &["どこ", "何処"]),
        ("nani", &["何", "なに"]),
        ("dare", &["誰", "だれ"]),
        ("itsu", &["いつ", "何時"]),
        ("douzo", &["どうぞ"]),
        ("shiteru", &["してる"]),
        ("wakarimashita", &["分かりました", "わかりました"]),
    ];

    for &(romaji, cands) in table {
        if romaji == input {
            return cands.iter().map(|&s| s.to_string()).collect();
        }
    }

    for &(romaji, cands) in table {
        if romaji.starts_with(input) {
            return cands.iter().map(|&s| s.to_string()).collect();
        }
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chinese_pinyin_candidates() {
        let cands_ni = get_cjk_candidates("ni", Language::ChineseSimplified);
        assert!(!cands_ni.is_empty());
        assert!(cands_ni.contains(&"你".to_string()));

        let cands_hao = get_cjk_candidates("hao", Language::ChineseSimplified);
        assert!(!cands_hao.is_empty());
        assert!(cands_hao.contains(&"好".to_string()));

        let cands_guo_trad = get_cjk_candidates("guo", Language::ChineseTraditional);
        assert!(!cands_guo_trad.is_empty());
        assert!(cands_guo_trad.contains(&"國".to_string()));
    }

    #[test]
    fn test_japanese_romaji_candidates() {
        let cands_ka = get_cjk_candidates("ka", Language::Japanese);
        assert!(!cands_ka.is_empty());
        assert!(cands_ka.contains(&"か".to_string()));

        let cands_arigatou = get_cjk_candidates("arigatou", Language::Japanese);
        assert!(!cands_arigatou.is_empty());
        assert!(cands_arigatou.contains(&"ありがとう".to_string()));
    }
}
