// ## Map
// Role: core (聖典經文連續性、篇章全體性、標點閉合與剛性禁字純函式積木)
// Inputs: 經文 Markdown / 文本切片、篇章與頌數標準約束、檔案名稱。
// Outputs: 詩節標記清單、單調連續性錯誤、標點閉合錯誤、禁字違規清單、篇章全體性錯誤、品質門禁綜合裁決。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、無遞迴 (R09-7)、巢狀深度 <= 2 (R09-3)、單函式 <= 100 行。

#![forbid(unsafe_code)]

/// 來源實體檔案序號空洞與重複報告
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageGapReport {
    pub total_files: usize,
    pub min_index: u32,
    pub max_index: u32,
    pub missing_indices: Vec<u32>,
    pub duplicate_indices: Vec<u32>,
}

/// 結尾題記 (Colophon / Puṣpikā / Iti Samāptam) 錨點比對結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColophonMatch {
    pub is_matched: bool,
    pub matched_signature: String,
    pub snippet: String,
}

/// 剛性禁字違規條目
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BannedViolation {
    pub line: usize,
    pub matched: String,
    pub reason: String,
    pub snippet: String,
}

/// 品質門禁綜合審查報告
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QualityReport {
    pub file_name: String,
    pub verdict: String, // "PASS" 或 "FAIL"
    pub verses_found_count: usize,
    pub verse_sample: Vec<i32>,
    pub filename_errors: Vec<String>,
    pub monotonicity_errors: Vec<String>,
    pub closure_errors: Vec<String>,
    pub banned_violations: Vec<BannedViolation>,
    pub totality_errors: Vec<String>,
}

/// 判斷字元是否為天城文或孟加拉文數字並轉為 ASCII '0'..'9'
/// @outputs: 2
fn convert_indic_digit(c: char) -> char {
    match c {
        '०' | '০' => '0',
        '१' | '১' => '1',
        '२' | '২' => '2',
        '३' | '৩' => '3',
        '४' | '৪' => '4',
        '५' | '৫' => '5',
        '६' | '৬' => '6',
        '७' | '৭' => '7',
        '८' | '৮' => '8',
        '९' | '৯' => '9',
        other => other,
    }
}

/// 將天城文與孟加拉文數字轉換為標準阿拉伯數字
/// @outputs: 1
pub fn normalize_indic_digits(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        out.push(convert_indic_digit(c));
    }
    out
}

/// 判斷字元是否為 CJK 統一表意文字
/// @outputs: 2
pub fn is_cjk_char(c: char) -> bool {
    ('\u{4E00}'..='\u{9FFF}').contains(&c)
}

/// 判斷字元是否為拉丁字母或 IAST 變音符號字元
/// @outputs: 2
pub fn is_latin_iast_char(c: char) -> bool {
    c.is_ascii_alphabetic()
        || ('\u{0100}'..='\u{024F}').contains(&c)
        || ('\u{1E00}'..='\u{1EFF}').contains(&c)
}

/// 判斷字元是否為天城文或孟加拉文字元
/// @outputs: 2
pub fn is_indic_script_char(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c) || ('\u{0980}'..='\u{09FF}').contains(&c)
}

/// 檢查交付檔案名稱是否符合三位一體雙語規範
/// 必須包含拉丁字母/IAST（原著名稱）與中文字符（中文譯名），且以 _繁體中文全譯本 結尾。
/// @outputs: 2
pub fn check_filename_bilingual(file_path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let base = match file_path.rsplit_once(['/', '\\']) {
        Some((_, name)) => name,
        None => file_path,
    };

    let stem = match base.rsplit_once('.') {
        Some((prefix, _)) => prefix,
        None => base,
    };

    // 僅針對全本交付檔案 (含全譯本或定本) 進行剛性檢查
    if !stem.contains("全譯本") && !stem.contains("定本") {
        return errors;
    }

    let has_latin = stem.chars().any(is_latin_iast_char);
    let has_cjk = stem.chars().any(is_cjk_char);

    if !has_latin {
        errors.push(format!(
            "❌ 檔名違規: '{base}' 僅有純中文，缺乏原典羅馬拼音/外文題名（例如 'Camatkāra-candrikā_奇妙月光_繁體中文全譯本'），違反雙語命名規範！"
        ));
    }
    if !has_cjk {
        errors.push(format!("❌ 檔名違規: '{base}' 缺乏中文譯名，違反雙語命名規範！"));
    }

    errors
}

/// 檢查是否為章節標題行（排除詩節標題如 Text N, Verse N）
/// @outputs: 2
fn is_chapter_header(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return false;
    }
    let norm = normalize_indic_digits(trimmed);
    if extract_title_verse_num(&norm).is_some() {
        return false;
    }
    let clean = trimmed.trim_start_matches('#').trim();
    let lower = clean.to_lowercase();
    lower.starts_with("chapter")
        || lower.starts_with("candrikā")
        || lower.starts_with("candrika")
        || lower.starts_with("adhyāya")
        || lower.starts_with("adhyaya")
        || lower.starts_with("kautūhala")
        || lower.contains("laharī")
        || lower.contains("lahari")
        || lower.contains("vibhāga")
        || lower.contains("vibhaga")
        || clean.contains("章")
        || clean.contains("道月光")
        || clean.contains("篇")
        || clean.contains("卷")
        || clean.contains("品")
        || clean.contains("浪")
        || clean.contains("編")
        || clean.contains("分")
        || clean.contains("কুতূহল")
        || clean.contains("অধ্যায়")
        || clean.contains("তরঙ্গ")
}

/// 檢查是否為結尾題記或跋語標題（Puṣpikā / Colophon）
/// @outputs: 2
fn is_colophon_header(line: &str) -> bool {
    let trimmed = line.trim();
    if !trimmed.starts_with('#') {
        return false;
    }
    let clean = trimmed.trim_start_matches('#').trim();
    let lower = clean.to_lowercase();
    clean.contains("題記")
        || clean.contains("跋語")
        || lower.contains("puṣpikā")
        || lower.contains("puspika")
        || lower.contains("colophon")
}

/// 檢查是否為結尾題記梵文行（例如 इति श्री... 或 iti śrī...）
/// @outputs: 2
fn is_colophon_line(line: &str) -> bool {
    let cleaned = line.trim_start_matches(&['>', '#', '*', ' ', '\t'][..]).trim();
    let lower = cleaned.to_lowercase();
    cleaned.starts_with("इति")
        || cleaned.starts_with("ইতি")
        || lower.starts_with("iti ")
        || lower.starts_with("iti-")
}

/// 從標題行抽取頌號（例如 Verse 27, Text 27, 第 27 頌）
/// @outputs: 3
fn extract_title_verse_num(norm_line: &str) -> Option<i32> {
    let trimmed = norm_line.trim_start_matches('#').trim();
    let lower = trimmed.to_lowercase();

    // 格式 1: Verse N, Text N, Sutra N
    for prefix in &["text", "verse", "sutra", "sūtra"] {
        if lower.starts_with(prefix) {
            let rest = trimmed[prefix.len()..].trim_start();
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(num) = digits.parse::<i32>() {
                return Some(num);
            }
        }
    }

    // 格式 2: 第 N 頌 / 第 N 节 / 第 N 節
    if trimmed.starts_with('第') {
        let rest = &trimmed['第'.len_utf8()..].trim_start();
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        let after_digits = rest[digits.len()..].trim_start();
        let is_verse_unit = after_digits.starts_with('頌')
            || after_digits.starts_with('颂')
            || after_digits.starts_with('節')
            || after_digits.starts_with('节');
        if !digits.is_empty() && is_verse_unit {
            if let Ok(num) = digits.parse::<i32>() {
                return Some(num);
            }
        }
    }

    None
}

/// 從原文閉合處抽取頌號（例如 ॥ 27 ॥ 或 || 27 ||）
/// @outputs: 2
fn extract_danda_verse_num(norm_line: &str) -> Option<i32> {
    for marker in &["॥", "||"] {
        if let Some(pos) = norm_line.find(marker) {
            let rest = &norm_line[pos + marker.len()..];
            let trimmed = rest.trim_start();
            let digits: String = trimmed.chars().take_while(|c| c.is_ascii_digit()).collect();
            let after = trimmed[digits.len()..].trim_start();
            let closed = after.starts_with("॥") || after.starts_with("||");
            if !digits.is_empty() && closed {
                if let Ok(num) = digits.parse::<i32>() {
                    return Some(num);
                }
            }
        }
    }
    None
}

/// 自經文文本抽取詩節號標記清單
/// 回傳清單元素: (verse_num, matched_raw, line_idx)
/// 章節標題記錄為 verse_num = -1
/// @outputs: 1
pub fn extract_verse_numbers_from_text(content: &str) -> Vec<(i32, String, usize)> {
    let mut raw_tokens = Vec::new();
    let mut in_colophon_block = false;

    for (idx_0, line) in content.lines().enumerate() {
        let line_idx = idx_0.saturating_add(1);
        let trimmed = line.trim();

        if trimmed.starts_with('|') && trimmed.ends_with('|') {
            continue;
        }

        if is_colophon_header(trimmed) {
            in_colophon_block = true;
            continue;
        }

        if is_chapter_header(trimmed) {
            in_colophon_block = false;
            raw_tokens.push((-1, trimmed.to_string(), line_idx));
            continue;
        }

        if in_colophon_block || is_colophon_line(trimmed) {
            continue;
        }

        let norm_line = normalize_indic_digits(trimmed);
        if let Some(num) = extract_title_verse_num(&norm_line) {
            raw_tokens.push((num, trimmed.to_string(), line_idx));
            continue;
        }
        if let Some(num) = extract_danda_verse_num(&norm_line) {
            raw_tokens.push((num, trimmed.to_string(), line_idx));
        }
    }

    consolidate_tokens(raw_tokens)
}

/// 整合標題行與閉合雙豎線：同一詩節單元內（15行以內）出現相同編號視為同一詩節
/// @outputs: 1
fn consolidate_tokens(tokens: Vec<(i32, String, usize)>) -> Vec<(i32, String, usize)> {
    let mut consolidated: Vec<(i32, String, usize)> = Vec::new();
    for (num, raw, line_idx) in tokens {
        if num == -1 {
            consolidated.push((num, raw, line_idx));
            continue;
        }
        if let Some(&(last_num, _, last_line)) = consolidated.last() {
            if last_num == num && line_idx.saturating_sub(last_line) <= 15 {
                continue;
            }
        }
        consolidated.push((num, raw, line_idx));
    }
    consolidated
}

/// 依章節邊界將經文標記切分為篇章
/// @outputs: 1
fn split_into_chapters(verses: &[(i32, String, usize)]) -> Vec<(String, Vec<(i32, String, usize)>)> {
    let mut chapters = Vec::new();
    let mut curr_name = "第一章".to_string();
    let mut curr_verses = Vec::new();

    for &(num, ref raw, line_idx) in verses {
        if num == -1 {
            if !curr_verses.is_empty() {
                chapters.push((curr_name.clone(), curr_verses));
                curr_verses = Vec::new();
            }
            curr_name = raw.trim_start_matches('#').trim().to_string();
        } else {
            curr_verses.push((num, raw.clone(), line_idx));
        }
    }
    if !curr_verses.is_empty() {
        chapters.push((curr_name, curr_verses));
    }

    chapters
}

/// 自篇章標題字串提取預期經文範圍 (起始頌, 終止頌)
/// 支援如 "1.2.31~1.2.71", "31~71", "31..71", "第 31~71 頌", "1.2.31-1.2.71"
/// 若未標注特定區間，預設回傳 (1, None)
/// @outputs: 1
pub fn extract_expected_range_from_chap_name(chap_name: &str) -> (i32, Option<i32>) {
    let normalized = normalize_indic_digits(chap_name);
    let sep_opt = if let Some(idx) = normalized.find('~') {
        Some((idx, 1))
    } else if let Some(idx) = normalized.find("..") {
        Some((idx, 2))
    } else {
        None
    };

    if let Some((sep_idx, sep_len)) = sep_opt {
        let left_part = &normalized[..sep_idx];
        let right_part = &normalized[sep_idx + sep_len..];

        let parse_tail_num = |s: &str| -> Option<i32> {
            let mut digits = String::new();
            for c in s.chars().rev() {
                if c.is_ascii_digit() {
                    digits.push(c);
                } else if !digits.is_empty() {
                    break;
                }
            }
            if digits.is_empty() {
                None
            } else {
                let s_rev: String = digits.chars().rev().collect();
                s_rev.parse::<i32>().ok()
            }
        };

        let parse_head_num = |s: &str| -> Option<i32> {
            let trimmed = s.trim_start_matches(|c: char| c.is_whitespace() || c == '(' || c == '（' || c == '[');
            let mut token = String::new();
            for c in trimmed.chars() {
                if c.is_ascii_digit() || c == '.' {
                    token.push(c);
                } else if !token.is_empty() {
                    break;
                }
            }
            if let Some(pos) = token.rfind('.') {
                token[pos + 1..].parse::<i32>().ok()
            } else {
                token.parse::<i32>().ok()
            }
        };

        if let (Some(start), Some(end)) = (parse_tail_num(left_part), parse_head_num(right_part)) {
            if start > 0 && end >= start {
                return (start, Some(end));
            }
        }
    }

    (1, None)
}

/// 檢驗單一章節內的經文單調連續性
/// @outputs: 1
fn check_chapter_verses_monotonicity(chap_name: &str, chap_verses: &[(i32, String, usize)]) -> Vec<String> {
    let mut errors = Vec::new();
    if chap_verses.is_empty() {
        return errors;
    }

    let (expected_start, expected_end_opt) = extract_expected_range_from_chap_name(chap_name);

    // 1. 起始完整度檢查
    let (first_v, _, first_line) = chap_verses[0];
    if first_v > expected_start {
        errors.push(format!(
            "❌ 完整度缺失 (Initial Verse Missing): 【{chap_name}】起始頌號為第 {first_v} 頌 (行 {first_line})，缺失前導經文 [第 {}..{} 頌]，非完整全本！",
            expected_start,
            first_v.saturating_sub(1)
        ));
    } else if first_v < expected_start {
        errors.push(format!(
            "❌ 起始頌號異常 (Unexpected Start Verse): 【{chap_name}】起始頌號為第 {first_v} 頌 (行 {first_line})，早於篇章預期之第 {expected_start} 頌！"
        ));
    }

    // 2. 檢驗章節內單調連續性
    for i in 0..chap_verses.len().saturating_sub(1) {
        let (curr_v, _, curr_line) = chap_verses[i];
        let (next_v, _, next_line) = chap_verses[i + 1];

        if next_v == curr_v {
            errors.push(format!(
                "重複頌號: 【{chap_name}】第 {curr_v} 頌在行 {curr_line} 與行 {next_line} 重複出現"
            ));
        } else if next_v > curr_v + 1 {
            let missing_count = next_v - curr_v - 1;
            let missing_range = if missing_count == 1 {
                format!("{}", curr_v + 1)
            } else {
                format!("{}..{}", curr_v + 1, next_v - 1)
            };
            errors.push(format!(
                "❌ 漏頌警告 (Missing Verses): 【{chap_name}】第 {curr_v} 頌 (行 {curr_line}) 與第 {next_v} 頌 (行 {next_line}) 之間缺失第 [{missing_range}] 頌！"
            ));
        } else if next_v < curr_v {
            errors.push(format!(
                "❌ 序號倒流 (Reverse Order): 【{chap_name}】第 {curr_v} 頌 (行 {curr_line}) 後面緊接著第 {next_v} 頌 (行 {next_line})"
            ));
        }
    }

    // 3. 終止完整度檢查（若章節名稱標記有結束界限）
    if let Some(expected_end) = expected_end_opt {
        let (last_v, _, last_line) = chap_verses[chap_verses.len() - 1];
        if last_v < expected_end {
            errors.push(format!(
                "❌ 篇章末頌缺失 (Final Verse Missing): 【{chap_name}】結束於第 {last_v} 頌 (行 {last_line})，未達篇章預期之第 {expected_end} 頌！"
            ));
        }
    }

    errors
}

/// 檢驗詩節序號單調連續性與起始完整度 (V_{i+1} = V_i + 1)
/// @outputs: 2
pub fn check_verse_monotonicity(verses: &[(i32, String, usize)]) -> Vec<String> {
    let mut errors = Vec::new();
    if verses.is_empty() || verses.iter().all(|v| v.0 == -1) {
        errors.push("⚠️ 未在文本中偵測到任何標準經文編號 (Verse N / Text N / ॥ N ॥)".to_string());
        return errors;
    }

    let chapters = split_into_chapters(verses);
    for (chap_name, chap_verses) in chapters {
        let chap_errs = check_chapter_verses_monotonicity(&chap_name, &chap_verses);
        errors.extend(chap_errs);
    }

    errors
}

/// 檢驗經文原文段落的標點閉合（雙豎線 ॥ / ||），防範未完結之斷句
/// @outputs: 1
pub fn check_danda_sentence_closure(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let mut in_sanskrit_block = false;
    let mut block_start_line = 0;
    let mut last_indic_line = String::new();

    for (idx_0, line) in content.lines().enumerate() {
        let line_idx = idx_0.saturating_add(1);
        let stripped = line.trim();

        let is_prose = stripped.chars().any(is_cjk_char)
            || stripped.starts_with('#')
            || stripped.starts_with('*')
            || stripped.starts_with('|');

        let has_indic = stripped.chars().any(is_indic_script_char) && !is_prose;

        if has_indic {
            if !in_sanskrit_block {
                in_sanskrit_block = true;
                block_start_line = line_idx;
            }
            last_indic_line = stripped.to_string();
        } else if in_sanskrit_block {
            let trimmed_last = last_indic_line.trim_end();
            let ends_with_double = trimmed_last.ends_with("॥")
                || trimmed_last.ends_with("||")
                || trimmed_last.ends_with("॥ ")
                || trimmed_last.ends_with("|| ");

            if !ends_with_double {
                let err_msg = if trimmed_last.ends_with('।') || trimmed_last.ends_with('|') {
                    format!("行 {block_start_line}-{}: 原文區塊僅以單豎線「।」結尾，疑似缺失下半頌！", line_idx.saturating_sub(1))
                } else {
                    let snippet: String = trimmed_last.chars().take(40).collect();
                    format!("行 {block_start_line}-{}: 原文區塊末尾缺乏雙豎線「॥」閉合標點，疑似遭跨頁截斷 (Truncated)！末句: '{snippet}'", line_idx.saturating_sub(1))
                };
                errors.push(err_msg);
            }
            in_sanskrit_block = false;
            last_indic_line.clear();
        }
    }

    errors
}

/// 掃描單行文字中的禁字與標點違規
/// @outputs: 1
fn scan_line_for_banned_words(line_idx: usize, line: &str) -> Vec<BannedViolation> {
    let mut violations = Vec::new();
    let trimmed = line.trim();

    if line.contains("皈依") {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "皈依".to_string(),
            reason: "嚴禁使用「皈依」，一律改用通俗字「歸依」".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }
    if line.contains("平台") {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "平台".to_string(),
            reason: "嚴禁使用「平台」，靈性境界與階梯一律改用「層級」".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }
    if line.contains("證悟") {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "證悟".to_string(),
            reason: "作形容詞或動詞時避用「證悟」，改為「覺悟自我的／感悟／親證」".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }
    if line.contains("達斯") && !line.contains("達斯亞") {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "達斯".to_string(),
            reason: "dāsa 結尾 a 音不可省略，一律作「達薩」，嚴禁「達斯」".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }
    if line.contains("主因鐸") || line.contains("主因陀羅") {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "主因鐸/主因陀羅".to_string(),
            reason: "半神人不可稱「主」，Lord Indra 必須翻譯為「天帝因鐸」".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }

    // 以太檢查 (排除 以太陽...)
    let mut search_from = 0;
    while let Some(pos) = line[search_from..].find("以太") {
        let abs_pos = search_from.saturating_add(pos);
        let after_idx = abs_pos.saturating_add("以太".len());
        let after_sun = line[after_idx..].starts_with('陽');
        if !after_sun {
            violations.push(BannedViolation {
                line: line_idx,
                matched: "以太".to_string(),
                reason: "五大元素之 ether 一律作「乙太」，嚴禁「以太」".to_string(),
                snippet: trimmed.chars().take(60).collect(),
            });
            break;
        }
        search_from = after_idx;
    }

    if line.contains('·') || line.contains('•') || line.contains('\u{02D9}') || line.contains('\u{2027}') {
        violations.push(BannedViolation {
            line: line_idx,
            matched: "·".to_string(),
            reason: "專名連接號嚴禁半形或間隔號，一律統一為全形句點「．」(U+FF0E)".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }

    if trimmed == "譯文:" || trimmed == "譯文：" {
        violations.push(BannedViolation {
            line: line_idx,
            matched: trimmed.to_string(),
            reason: "存在空白未填之「譯文:」單元，繁體中文譯文不可缺失！".to_string(),
            snippet: trimmed.chars().take(60).collect(),
        });
    }

    violations
}

/// 檢驗絕對禁字與英文殘留
/// @outputs: 1
pub fn check_banned_words_and_leakage(content: &str) -> Vec<BannedViolation> {
    let mut violations = Vec::new();
    for (idx_0, line) in content.lines().enumerate() {
        let line_idx = idx_0.saturating_add(1);
        let line_violations = scan_line_for_banned_words(line_idx, line);
        violations.extend(line_violations);
    }
    violations
}

/// 檢驗篇章全體性、末章末頌閉合與全書預期頌數（防篇章截斷與 50+ 頌缺失）
/// @outputs: 2
/// @domain: expected_chapters [0..100]
/// @domain: min_final_verse [0..10000]
/// @domain: min_total_verses [0..100000]
pub fn check_book_totality(
    verse_tokens: &[(i32, String, usize)],
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
) -> Vec<String> {
    // 執行期守衛 (R09-4 Anti-Fake Compliance)
    if let Some(ch) = expected_chapters {
        if ch > 100 {
            return vec!["預期篇章數超出定義域 [0..100]".to_string()];
        }
    }
    if let Some(fv) = min_final_verse {
        if fv < 0 || fv > 10000 {
            return vec!["末章頌數超出定義域 [0..10000]".to_string()];
        }
    }
    if let Some(tv) = min_total_verses {
        if tv > 100000 {
            return vec!["總頌數超出定義域 [0..100000]".to_string()];
        }
    }

    let mut errors = Vec::new();
    let chapters = split_into_chapters(verse_tokens);

    // 0. 若未指定明確篇章與頌數標準，執行文獻學自適應全體性檢驗 (Zero-Config Dynamic Totality)
    if expected_chapters.is_none() && min_final_verse.is_none() && min_total_verses.is_none() {
        if chapters.is_empty() {
            errors.push("❌ 篇章全體性錯誤: 經文中未偵測到任何有效章節或頌節！".to_string());
            return errors;
        }
        for (chap_name, verses) in &chapters {
            if verses.is_empty() {
                errors.push(format!("❌ 篇章經文缺失: 篇章【{chap_name}】未包含任何詩節！"));
                continue;
            }
            let (expected_start, expected_end_opt) = extract_expected_range_from_chap_name(chap_name);
            let first_v = verses[0].0;
            if first_v != expected_start {
                errors.push(format!(
                    "❌ 篇章首頌異常: 篇章【{chap_name}】起始頌為第 {first_v} 頌，非預期起始第 {expected_start} 頌（可能前面有缺頁/缺頌）！"
                ));
            }
            if let Some(expected_end) = expected_end_opt {
                let last_v = verses[verses.len() - 1].0;
                if last_v < expected_end {
                    errors.push(format!(
                        "❌ 篇章末頌異常: 篇章【{chap_name}】結束於第 {last_v} 頌，未達預期終結第 {expected_end} 頌！"
                    ));
                }
            }
        }
    }

    // 1. 檢驗篇章總數
    if let Some(exp_ch) = expected_chapters {
        if chapters.len() < exp_ch {
            let chap_names: Vec<&str> = chapters.iter().map(|c| c.0.as_str()).collect();
            errors.push(format!(
                "❌ 篇章全體性缺失 (Incomplete Chapters): 預期全書應收錄 {exp_ch} 個篇章，實際僅偵測到 {} 個篇章（已收錄：{chap_names:?}），嚴重缺失後續篇章！",
                chapters.len()
            ));
        }
    }

    // 2. 檢驗末章最高頌號
    if let Some(min_fv) = min_final_verse {
        if let Some((final_chap_name, final_verses)) = chapters.last() {
            let max_v = final_verses.iter().map(|v| v.0).max().unwrap_or(0);
            if max_v < min_fv {
                errors.push(format!(
                    "❌ 末章經文截斷 (Truncated Final Chapter): 【{final_chap_name}】最高僅收錄至第 {max_v} 頌，未達到原著全本預期之第 {min_fv} 頌（缺失 50+ 頌）！"
                ));
            }
        }
    }

    // 3. 檢驗全書總頌數
    let total_verses = verse_tokens.iter().filter(|v| v.0 != -1).count();
    if let Some(min_tv) = min_total_verses {
        if total_verses < min_tv {
            errors.push(format!(
                "❌ 全書經文量不足 (Insufficient Verses): 全書實際收錄 {total_verses} 頌，低於全本門檻下限 {min_tv} 頌！"
            ));
        }
    }

    errors
}

/// 從檔案名稱中提取自然頁碼/序號 (例如 "page_042.png" -> Some(42))
/// @outputs: 3
/// @domain: filename.len() [1..256]
pub fn extract_page_index_from_filename(filename: &str) -> Option<u32> {
    if filename.is_empty() || filename.len() > 256 {
        return None;
    }
    let stem = filename.split('/').last().unwrap_or(filename);
    let stem_no_ext = stem.split('.').next().unwrap_or(stem);

    let mut num_str = String::new();
    let mut found_digit = false;
    for c in stem_no_ext.chars().rev() {
        if c.is_ascii_digit() {
            num_str.push(c);
            found_digit = true;
        } else if found_digit {
            break;
        }
    }
    if num_str.is_empty() {
        return None;
    }
    let reversed: String = num_str.chars().rev().collect();
    reversed.parse::<u32>().ok()
}

/// 檢驗實體檔案序號清單是否有空洞 (Gap) 或重複 (Duplicate)
/// @outputs: 4
/// @domain: max_allowed_index [1..65535]
pub fn check_page_sequence_gaps(
    indices: &[u32],
    max_allowed_index: u32,
) -> Result<PageGapReport, &'static str> {
    if indices.is_empty() {
        return Err("序號清單不可為空");
    }
    if max_allowed_index == 0 || max_allowed_index > 65535 {
        return Err("最大容許索引超出定義域 [1..65535]");
    }
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();

    let min_index = sorted[0];
    let max_index = sorted[sorted.len() - 1];

    if max_index > max_allowed_index {
        return Err("偵測到超出上限之序號");
    }

    let mut missing = Vec::new();
    let mut duplicates = Vec::new();
    let scan_len = sorted.len().min(4096);

    for i in 0..scan_len.saturating_sub(1) {
        let curr = sorted[i];
        let next = sorted[i + 1];
        if next == curr {
            duplicates.push(curr);
        } else if next > curr.saturating_add(1) {
            let gap_start = curr.saturating_add(1);
            let gap_end = next.saturating_sub(1);
            for g in gap_start..=gap_end.min(gap_start.saturating_add(256)) {
                missing.push(g);
            }
        }
    }

    Ok(PageGapReport {
        total_files: sorted.len(),
        min_index,
        max_index,
        missing_indices: missing,
        duplicate_indices: duplicates,
    })
}

/// 結尾題記 (Colophon / Puṣpikā / Iti Samāptam) 容錯比對
/// @outputs: 2
pub fn match_terminal_colophon_anchor(text_tail: &str) -> ColophonMatch {
    if text_tail.is_empty() {
        return ColophonMatch {
            is_matched: false,
            matched_signature: String::new(),
            snippet: String::new(),
        };
    }

    let signatures = [
        "समाप्तम्",
        "समाप्तः",
        "समाप्ता",
        "समाप्त",
        "সমাপ্ত",
        "সমাপ্তম্",
        "iti samāptam",
        "iti samaptam",
        "iti samāptaḥ",
        "iti samaptah",
        "iti śrī",
        "iti sri",
        "śubham bhavatu",
        "subham bhavatu",
        "शुभं भवतु",
        "हरिः ॐ तत् सत्",
        "全書終",
        "第.*章終",
        "圓滿",
    ];

    let lower = text_tail.to_lowercase();
    for &sig in &signatures {
        if let Some(pos) = lower.find(sig) {
            let mut start = pos.saturating_sub(30);
            while start > 0 && !text_tail.is_char_boundary(start) {
                start = start.saturating_sub(1);
            }
            let mut end = (pos.saturating_add(sig.len()).saturating_add(30)).min(text_tail.len());
            while end < text_tail.len() && !text_tail.is_char_boundary(end) {
                end = end.saturating_add(1);
            }
            let snippet = text_tail[start..end].replace('\n', " ");
            return ColophonMatch {
                is_matched: true,
                matched_signature: sig.to_string(),
                snippet,
            };
        }
    }

    ColophonMatch {
        is_matched: false,
        matched_signature: String::new(),
        snippet: String::new(),
    }
}

/// 支援結尾題記驗證的進階全體性門禁組合子
/// @outputs: 1
pub fn check_book_totality_advanced(
    verse_tokens: &[(i32, String, usize)],
    content: &str,
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
    require_colophon: bool,
) -> Vec<String> {
    let mut errors = check_book_totality(verse_tokens, expected_chapters, min_final_verse, min_total_verses);
    if require_colophon {
        let tail_len = 2000.min(content.len());
        let tail_slice = &content[content.len().saturating_sub(tail_len)..];
        let colophon = match_terminal_colophon_anchor(tail_slice);
        if !colophon.is_matched {
            errors.push("❌ 結尾題記缺失 (Terminal Colophon Missing): 經文末尾未偵測到印度古典閉合標誌（如 इति समाप्तम् / সমাপ্ত / 全書終），書籍可能在末端被截斷！".to_string());
        }
    }
    errors
}

/// 執行完整五道剛性門禁審查（含篇章全體性與末章閉合）
/// @outputs: 2
pub fn inspect_markdown_content(
    content: &str,
    file_path: &str,
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
) -> QualityReport {
    let verse_tokens = extract_verse_numbers_from_text(content);
    let monotonicity_errors = check_verse_monotonicity(&verse_tokens);
    let closure_errors = check_danda_sentence_closure(content);
    let banned_violations = check_banned_words_and_leakage(content);
    let totality_errors = check_book_totality(
        &verse_tokens,
        expected_chapters,
        min_final_verse,
        min_total_verses,
    );
    let filename_errors = check_filename_bilingual(file_path);

    let all_passed = filename_errors.is_empty()
        && monotonicity_errors.is_empty()
        && closure_errors.is_empty()
        && banned_violations.is_empty()
        && totality_errors.is_empty();

    let sample: Vec<i32> = verse_tokens
        .iter()
        .filter(|v| v.0 != -1)
        .map(|v| v.0)
        .take(10)
        .collect();

    QualityReport {
        file_name: file_path.to_string(),
        verdict: if all_passed { "PASS" } else { "FAIL" }.to_string(),
        verses_found_count: verse_tokens.iter().filter(|v| v.0 != -1).count(),
        verse_sample: sample,
        filename_errors,
        monotonicity_errors,
        closure_errors,
        banned_violations,
        totality_errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_indic_digits_devanagari_and_bengali() {
        let input = "Dev: १२३४५६७८९० | Beng: ১২৩৪৫৬৭৮৯০";
        let norm = normalize_indic_digits(input);
        assert_eq!(norm, "Dev: 1234567890 | Beng: 1234567890");
    }

    #[test]
    fn test_check_filename_bilingual_valid_and_invalid() {
        let good = "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md";
        assert!(check_filename_bilingual(good).is_empty());

        let pure_chinese = "奇妙月光_繁體中文全譯本.md";
        let errs = check_filename_bilingual(pure_chinese);
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("缺乏原典羅馬拼音"));
    }

    #[test]
    fn test_banned_words_detector() {
        let text = "我們應當皈依至尊主，登上靈性平台，達到證悟境界，跟隨哈里達斯學習以太。";
        let violations = check_banned_words_and_leakage(text);
        assert_eq!(violations.len(), 5);
    }

    #[test]
    fn test_colophon_puspika_ignored_in_verse_numbers() {
        let md = "### Verse 45\n> verse 45 content || 45 ||\n### Verse 46\n> verse 46 content || 46 ||\n\n## 【東方第一浪圓滿題記（Puṣpikā）】\n> ### इति श्री-भक्ति-रसामृत-सिन्धौ पूर्व-विभागे सामान्य-भक्ति-निरूपणं नाम प्रथमा लहरी ॥ १ ॥\n*iti śrī-bhakti-rasāmṛta-sindhau pūrva-vibhāge sāmānya-bhakti-nirūpaṇaṁ nāma prathamā laharī || 1 ||*";
        let tokens = extract_verse_numbers_from_text(md);
        let verse_nums: Vec<i32> = tokens.into_iter().filter(|v| v.0 != -1).map(|v| v.0).collect();
        assert_eq!(verse_nums, vec![45, 46]);
    }

    #[test]
    fn test_extract_expected_range_from_chap_name() {
        let range1 = extract_expected_range_from_chap_name("東方第二浪中編：解脫超越辨析與奉愛無上獨立自足論（1.2.31~1.2.71）");
        assert_eq!(range1, (31, Some(71)));

        let range2 = extract_expected_range_from_chap_name("東方第二浪前編：修行奉愛界定與三品信德資格論（1.2.1~1.2.30）");
        assert_eq!(range2, (1, Some(30)));

        let range3 = extract_expected_range_from_chap_name("第 31~71 頌");
        assert_eq!(range3, (31, Some(71)));

        let range4 = extract_expected_range_from_chap_name("第一章：春季篇 [1..50]");
        assert_eq!(range4, (1, Some(50)));

        let range5 = extract_expected_range_from_chap_name("第二章：無範圍標注篇");
        assert_eq!(range5, (1, None));
    }

    #[test]
    fn test_subinstallment_continuity_validation() {
        let md_valid = "# 東方第二浪中編（1.2.31~1.2.33）\n### Verse 31\n> content || 31 ||\n### Verse 32\n> content || 32 ||\n### Verse 33\n> content || 33 ||\n";
        let report = inspect_markdown_content(md_valid, "Bhakti-rasāmṛta-sindhu_奉愛甘露之洋_繁體中文全譯本.md", None, None, None);
        assert_eq!(report.monotonicity_errors, Vec::<String>::new());
        assert_eq!(report.totality_errors, Vec::<String>::new());

        let md_missing_start = "# 東方第二浪中編（1.2.31~1.2.33）\n### Verse 32\n> content || 32 ||\n### Verse 33\n> content || 33 ||\n";
        let report_bad_start = inspect_markdown_content(md_missing_start, "Bhakti-rasāmṛta-sindhu_奉愛甘露之洋_繁體中文全譯本.md", None, None, None);
        assert!(!report_bad_start.monotonicity_errors.is_empty());

        let md_missing_end = "# 東方第二浪中編（1.2.31~1.2.33）\n### Verse 31\n> content || 31 ||\n### Verse 32\n> content || 32 ||\n";
        let report_bad_end = inspect_markdown_content(md_missing_end, "Bhakti-rasāmṛta-sindhu_奉愛甘露之洋_繁體中文全譯本.md", None, None, None);
        assert!(!report_bad_end.totality_errors.is_empty());
    }
}
