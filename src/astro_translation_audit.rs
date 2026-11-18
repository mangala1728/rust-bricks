// ## Map
// Role: core/astro_translation_audit — 吠陀占星經典翻譯完整度與雙模態排版品質審查純函式積木。
// Inputs: Markdown/Docx 純文字與 XML 片段。
// Outputs: 結構完整度報告、禁字違規、英文殘留與空白譯文檢測結果。
// Invariants: 零外部依賴、零全域可變狀態、禁止 unsafe、無遞迴、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AstroBookType {
    Vedanga,
    Garga,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstroSectionStatus {
    pub name: String,
    pub found: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstroCompletenessReport {
    pub file_path: String,
    pub book_type: AstroBookType,
    pub book_title: String,
    pub char_count: usize,
    pub line_count: usize,
    pub table_line_count: usize,
    pub has_tables: bool,
    pub verses_found: usize,
    pub min_verses: usize,
    pub sections: Vec<AstroSectionStatus>,
    pub missing_sections: Vec<String>,
    pub local_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AstroBannedError {
    pub line: usize,
    pub matched: String,
    pub snippet: String,
    pub reason: String,
}

const VEDANGA_SECTIONS: &[(&str, &[&str])] = &[
    ("導言與曆制背景", &["導言", "序言", "引言", "Introduction", "印度曆法"]),
    ("曆表 A：三十年會曆表", &["曆表 A", "Table A", "三十年", "30 Years", "Garga", "Dikshit", "冬至", "夏至"]),
    ("曆表 B：四百八十年大周期", &["曆表 B", "Table B", "480", "大周期", "Cycle", "Exeligmos", "閏月"]),
    ("交會朔望表", &["交會", "Parva", "朔望表", "Table of Parvas", "124"]),
    ("數理推導與天文計算", &["數理推導", "算式", "分數", "證明", "Calculation", "度數", "陽曆", "太陰"]),
    ("全書經文與注釋", &["頌", "第 1 頌", "第 46 頌", "經文", "Somakara", "釋義"]),
];

const GARGA_SECTIONS: &[(&str, &[&str])] = &[
    ("序言與導論考證", &["序言", "Foreword", "Daya Shanker", "導言", "Preface", "Pathak", "Sarasvatī"]),
    ("第一章：夭壽與化解格局", &["第一章", "夭壽", "死難", "Arishta", "Balarishta", "化解", "傘蓋", "Chhatra"]),
    ("第二章：十二宮主星飛宮果報", &["第二章", "十二宮主星", "命主星", "財帛主星", "Lagneśa", "Dhaneśa", "矩陣", "12×12"]),
    ("第三章：九曜入十二宮位果報", &["第三章", "九曜", "太陽", "月亮", "火星", "木星", "土星", "羅睺", "計都"]),
    ("九分群星宿法 (Navatāra)", &["九分群星宿", "Navatāra", "誕生宿", "財富宿", "災厄宿", "安適宿", "殺滅宿", "Janma", "Sampat"]),
    ("全套結構矩陣表格", &["矩陣表", "速查總表", "二十七宿", "12×12", "Navatāra", "主星類別"]),
];

/// 判斷文件屬於吠陀支占星 (Vedanga) 或迦爾嘎占星經 (Garga)
/// @outputs: 2
pub fn detect_astro_book_type(content: &str, file_path: &str) -> AstroBookType {
    let lower_path = file_path.to_lowercase();
    let head: String = content.chars().take(2000).collect();
    let lower_head = head.to_lowercase();
    if lower_path.contains("garga")
        || file_path.contains("迦爾嘎")
        || lower_head.contains("garga")
        || head.contains("迦爾嘎")
    {
        AstroBookType::Garga
    } else {
        AstroBookType::Vedanga
    }
}

/// 從文本統計去重後的 (?:VJ|GH)\s+(\d+(?:\.\d+)*) 經文頌號數
/// @outputs: 1
pub fn count_distinct_astro_verses(content: &str) -> usize {
    let mut found = BTreeSet::new();
    let bytes = content.as_bytes();
    let mut i = 0usize;

    while i + 3 < bytes.len() {
        let is_prefix = (bytes[i] == b'V' && bytes[i + 1] == b'J')
            || (bytes[i] == b'G' && bytes[i + 1] == b'H');
        if !is_prefix {
            i += 1;
            continue;
        }
        let prev_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
        if !prev_ok || !bytes[i + 2].is_ascii_whitespace() {
            i += 2;
            continue;
        }
        let mut j = i + 2;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        let num_start = j;
        while j < bytes.len() && (bytes[j].is_ascii_digit() || bytes[j] == b'.') {
            j += 1;
        }
        let token = content[num_start..j].trim_end_matches('.');
        if token.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            found.insert(token.to_string());
        }
        i = j.max(i + 2);
    }

    found.len()
}

/// 分析吠陀占星譯本 Markdown 之完整度特徵與離線判定
/// @outputs: 1
pub fn analyze_astro_markdown(content: &str, file_path: &str) -> AstroCompletenessReport {
    let book_type = detect_astro_book_type(content, file_path);
    let (book_title, specs, min_verses) = match book_type {
        AstroBookType::Vedanga => ("Vedanga Jyotisha (吠陀支占星)", VEDANGA_SECTIONS, 43usize),
        AstroBookType::Garga => ("Garga Hora Shastra (迦爾嘎占星經)", GARGA_SECTIONS, 40usize),
    };

    let char_count = content.chars().count();
    let line_count = content.lines().count();
    let table_line_count = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            t.starts_with('|') && t.ends_with('|')
        })
        .count();
    let has_tables = table_line_count >= 10;
    let verses_found = count_distinct_astro_verses(content);

    let mut sections = Vec::new();
    let mut missing_sections = Vec::new();
    for &(sec_name, keywords) in specs {
        let found = keywords.iter().any(|kw| content.contains(kw));
        sections.push(AstroSectionStatus {
            name: sec_name.to_string(),
            found,
        });
        if !found {
            missing_sections.push(sec_name.to_string());
        }
    }

    let local_passed = missing_sections.is_empty() && verses_found >= min_verses && has_tables;

    AstroCompletenessReport {
        file_path: file_path.to_string(),
        book_type,
        book_title: book_title.to_string(),
        char_count,
        line_count,
        table_line_count,
        has_tables,
        verses_found,
        min_verses,
        sections,
        missing_sections,
        local_passed,
    }
}

/// 檢查單字是否在所有出現位置都違反後接/前導條件（如「達斯(?!亞)」與「(?<!以)以太(?!陽)」）
/// @outputs: 1
fn has_unprotected_das_or_ether(line: &str, keyword: &str, forbidden_prefix: Option<&str>, allowed_suffix: &str) -> bool {
    let mut start = 0usize;
    while let Some(rel) = line[start..].find(keyword) {
        let pos = start.saturating_add(rel);
        let after = &line[pos + keyword.len()..];
        let before = &line[..pos];
        let suffix_ok = !allowed_suffix.is_empty() && after.starts_with(allowed_suffix);
        let prefix_ok = match forbidden_prefix {
            Some(pre) => before.ends_with(pre),
            None => false,
        };
        if !suffix_ok && !prefix_ok {
            return true;
        }
        start = pos.saturating_add(keyword.len());
    }
    false
}

/// 依吠陀占星翻譯紅線逐行掃描禁字與半形連接號
/// @outputs: 1
pub fn check_astro_banned_words(text: &str) -> Vec<AstroBannedError> {
    let mut errors = Vec::new();

    for (idx_0, line) in text.lines().enumerate() {
        let line_no = idx_0.saturating_add(1);
        let snippet: String = line.trim().chars().take(60).collect();

        if line.contains("皈依") {
            errors.push(AstroBannedError {
                line: line_no,
                matched: "皈依".to_string(),
                snippet: snippet.clone(),
                reason: "嚴禁使用「皈依」，一律改用通俗字「歸依」".to_string(),
            });
        }
        if line.contains("平台") {
            errors.push(AstroBannedError {
                line: line_no,
                matched: "平台".to_string(),
                snippet: snippet.clone(),
                reason: "嚴禁使用「平台」，靈性境界與階梯一律改用「層級」".to_string(),
            });
        }
        if line.contains("證悟") {
            errors.push(AstroBannedError {
                line: line_no,
                matched: "證悟".to_string(),
                snippet: snippet.clone(),
                reason: "作形容詞或動詞時避用「證悟」，改為「覺悟自我的／感悟／親證」".to_string(),
            });
        }
        if has_unprotected_das_or_ether(line, "達斯", None, "亞") {
            errors.push(AstroBannedError {
                line: line_no,
                matched: "達斯".to_string(),
                snippet: snippet.clone(),
                reason: "dāsa 結尾 a 音不可省略，一律作「達薩」，嚴禁「達斯」".to_string(),
            });
        }
        if has_unprotected_das_or_ether(line, "以太", Some("以"), "陽") {
            errors.push(AstroBannedError {
                line: line_no,
                matched: "以太".to_string(),
                snippet: snippet.clone(),
                reason: "五大元素之 ether 一律作「乙太」，嚴禁「以太」".to_string(),
            });
        }
        if let Some(dot) = line.chars().find(|&c| c == '\u{02D9}' || c == '\u{00B7}' || c == '\u{2027}') {
            errors.push(AstroBannedError {
                line: line_no,
                matched: dot.to_string(),
                snippet,
                reason: "專名連接號嚴禁半形或間隔號，一律統一為全形句點「．」(U+FF0E)".to_string(),
            });
        }
    }

    errors
}

/// 判斷是否為殘留長英文段落 (^[A-Z][a-zA-Z\s,;:'"-]{30,})
/// @outputs: 1
fn is_long_english_leak_line(trimmed: &str) -> bool {
    let mut chars = trimmed.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_uppercase() {
        return false;
    }
    let mut count = 0usize;
    for c in chars {
        let valid = c.is_ascii_alphabetic()
            || c == ' '
            || c == '\t'
            || c == ','
            || c == ';'
            || c == ':'
            || c == '\''
            || c == '"'
            || c == '-';
        if !valid {
            break;
        }
        count += 1;
        if count >= 30 {
            return true;
        }
    }
    false
}

/// 檢測 Translation: 標籤或長句英文翻譯洩漏
/// @outputs: 1
pub fn check_astro_english_leakage(text: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for (idx_0, line) in text.lines().enumerate() {
        let line_no = idx_0.saturating_add(1);
        let trimmed = line.trim();
        if trimmed.starts_with("Translation:") {
            let clip: String = trimmed.chars().take(40).collect();
            errors.push(format!("Line {line_no}: 殘留英文翻譯標籤 '{clip}'"));
        } else if is_long_english_leak_line(trimmed) {
            let clip: String = trimmed.chars().take(50).collect();
            errors.push(format!("Line {line_no}: 疑似殘留長英文句子 '{clip}'"));
        }
    }
    errors
}

/// 檢查段落列表內是否存在空白未填或跳節之「譯文:」單元
/// @outputs: 1
pub fn check_astro_empty_translations(paragraphs: &[String]) -> Vec<String> {
    let mut errors = Vec::new();
    for (idx, p) in paragraphs.iter().enumerate() {
        let text = p.trim();
        if text != "譯文:" && text != "譯文：" {
            continue;
        }
        let next_opt = paragraphs.get(idx.saturating_add(1)).map(|s| s.trim());
        match next_opt {
            None | Some("") => {
                errors.push(format!("Paragraph {idx}: 存在空白未填之「譯文:」單元"));
            }
            Some(next) if next.starts_with("VJ ") || next.starts_with("GS ") => {
                errors.push(format!("Paragraph {idx}: 「譯文:」未填入中文內容即跳下一個節號"));
            }
            _ => {}
        }
    }
    errors
}

/// 解碼常見 XML 跳脫字元（&amp; 置於最後防止雙重解碼）
/// @outputs: 1
fn unescape_xml_text(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// 從 Word document.xml 字串萃取所有 <w:p> 純文字段落
/// @outputs: 1
pub fn extract_docx_paragraphs_from_xml(xml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut search_from = 0usize;

    while let Some(start) = xml[search_from..].find("<w:p") {
        let abs_start = search_from.saturating_add(start);
        let Some(end_tag) = xml[abs_start..].find("</w:p>") else {
            break;
        };
        let abs_end = abs_start.saturating_add(end_tag).saturating_add("</w:p>".len());
        let p_chunk = &xml[abs_start..abs_end];
        paragraphs.push(extract_single_paragraph_text(p_chunk));
        search_from = abs_end;
    }

    paragraphs
}

/// 從單一 <w:p> XML 片段組合所有 <w:t> 文字並解碼實體
/// @outputs: 1
fn extract_single_paragraph_text(p_chunk: &str) -> String {
    let mut p_text = String::new();
    let mut t_search = 0usize;
    while let Some(t_start) = p_chunk[t_search..].find("<w:t") {
        let abs_t = t_search.saturating_add(t_start);
        let Some(tag_close) = p_chunk[abs_t..].find('>') else {
            break;
        };
        let text_begin = abs_t.saturating_add(tag_close).saturating_add(1);
        let Some(t_end) = p_chunk[text_begin..].find("</w:t>") else {
            break;
        };
        let text_end = text_begin.saturating_add(t_end);
        p_text.push_str(&p_chunk[text_begin..text_end]);
        t_search = text_end.saturating_add("</w:t>".len());
    }
    unescape_xml_text(&p_text)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_analyze_astro_markdown_vedanga_complete_and_defect() {
        let mut complete = String::from(
            "# Vedanga Jyotisha\n導言 Table A Table B Table of Parvas 數理推導 第 1 頌\n",
        );
        for _ in 0..12 {
            complete.push_str("| col1 | col2 |\n");
        }
        for v in 1..=45 {
            complete.push_str(&format!("VJ {v}\n"));
        }
        let rep = analyze_astro_markdown(&complete, "Vedanga_Jyotisha.md");
        assert_eq!(rep.book_type, AstroBookType::Vedanga);
        assert_eq!(rep.verses_found, 45);
        assert_eq!(rep.table_line_count, 12);
        assert!(rep.missing_sections.is_empty());
        assert!(rep.local_passed);

        let incomplete = "# Vedanga Jyotisha\n導言\nVJ 1\nVJ 2\n";
        let bad_rep = analyze_astro_markdown(incomplete, "Vedanga_Jyotisha.md");
        assert!(!bad_rep.local_passed);
        assert_eq!(bad_rep.verses_found, 2);
        assert_eq!(bad_rep.missing_sections.len(), 5);
    }

    /// @outputs: 1
    #[test]
    fn test_analyze_astro_markdown_garga_detection() {
        let mut garga = String::from(
            "# Garga Hora\n序言 第一章 第二章 第三章 Navatāra 矩陣表\n",
        );
        for _ in 0..10 {
            garga.push_str("| a | b |\n");
        }
        for v in 1..=40 {
            garga.push_str(&format!("GH 1.{v}\n"));
        }
        let rep = analyze_astro_markdown(&garga, "Garga_Hora.md");
        assert_eq!(rep.book_type, AstroBookType::Garga);
        assert_eq!(rep.min_verses, 40);
        assert!(rep.local_passed);
    }

    /// @outputs: 1
    #[test]
    fn test_check_astro_banned_words_and_middle_dot() {
        let bad = "皈依 平台 證悟 達斯 以太 拉嘎達·大仙";
        let errs = check_astro_banned_words(bad);
        assert_eq!(errs.len(), 6);

        let good = "歸依 層級 感悟 達斯亞 達薩 乙太 以太陽為基準 拉嘎達．大仙";
        assert!(check_astro_banned_words(good).is_empty());
    }

    /// @outputs: 1
    #[test]
    fn test_english_leakage_and_empty_translations_and_xml() {
        let leak = "Translation: Verily the sun rises.\nThis is a long untranslated English paragraph exceeding thirty characters.";
        let leak_errs = check_astro_english_leakage(leak);
        assert_eq!(leak_errs.len(), 2);

        let paras = vec![
            "譯文:".to_string(),
            "".to_string(),
            "譯文：".to_string(),
            "VJ 1-2".to_string(),
            "譯文:".to_string(),
            "正確譯文內容。".to_string(),
        ];
        let empty_errs = check_astro_empty_translations(&paras);
        assert_eq!(empty_errs.len(), 2);

        let xml = "<w:body><w:p><w:r><w:t>VJ &amp; GS</w:t></w:r></w:p><w:p><w:r><w:t>譯文:</w:t></w:r></w:p></w:body>";
        let extracted = extract_docx_paragraphs_from_xml(xml);
        assert_eq!(extracted, vec!["VJ & GS".to_string(), "譯文:".to_string()]);
    }
}
