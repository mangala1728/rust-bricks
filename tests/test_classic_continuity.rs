// ## Map
// Role: tests/test_classic_continuity.rs — 古典聖典經文連續性、標點閉合、禁字與全體性剛性門禁測試套件。
// Inputs: 標準件 (Golden)、挑戰件 (Defect / Challenge) 經文樣本。
// Outputs: 單元測試斷言結果 (全部 PASS)。
// Invariants: 零外部依賴、純 Rust 實現、禁止弱斷言、覆蓋邊界與突變樣本。

#![forbid(unsafe_code)]

use rust_bricks::classic_continuity::{
    check_filename_bilingual, inspect_markdown_content, normalize_indic_digits,
};

#[test]
fn test_indic_digit_normalization() {
    let input = "Dev: १२३४५६७८९० | Beng: ১২৩৪৫৬৭৮৯০";
    let norm = normalize_indic_digits(input);
    assert_eq!(norm, "Dev: 1234567890 | Beng: 1234567890");
}

#[test]
fn test_golden_continuous_verses() {
    let content = r#"# Sample-Classic_示範經文_繁體中文全譯本

## 第一章

Text 1 ．第一頌
यावत् किम् एतद् इति ता अहाहेति नोचुर्
तावत् सुदाम-तनया मुखम् उल्ललास ॥ १ ॥

譯文: 當她們尚未發出驚呼之時，那位牧牛少年已令眾人喜悅。

Text 2 ．第二頌
सर्वः कला-निधिর্ अहो युगपच् चुचुम्ब ॥ २ ॥

譯文: 當愛欲巨鱷尚未攫取她抵抗的肢體，那位所有藝術寶庫的至尊主，早已霍然起身，同時親吻了所有姑娘。

Text 3 ．第三頌
धन्यं भूषणम् अस्तु ते ॥ ३ ॥

譯文: 願這件飾物享有榮耀！願為妳帶來這件飾物的夫婿享有榮耀！
"#;

    let res = inspect_markdown_content(
        content,
        "Sample-Classic_示範經文_繁體中文全譯本.md",
        Some(1),
        Some(3),
        Some(3),
    );

    assert_eq!(res.verdict, "PASS");
    assert_eq!(res.verses_found_count, 3);
    assert!(res.monotonicity_errors.is_empty());
    assert!(res.closure_errors.is_empty());
    assert!(res.banned_violations.is_empty());
    assert!(res.totality_errors.is_empty());
    assert!(res.filename_errors.is_empty());
}

#[test]
fn test_incomplete_starting_verse_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一章

Text 27 ．第二十七頌
॥ २७ ॥
譯文: 從第二十七頌開始，缺失前二十六頌。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    assert_eq!(res.verdict, "FAIL");
    assert!(res.monotonicity_errors.iter().any(|e| e.contains("完整度缺失") && e.contains("起始頌號為第 27 頌")));
}

#[test]
fn test_missing_verse_gap_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一章

Text 1 ．第一頌
॥ १ ॥
譯文: 第一頌。

Text 5 ．第五頌
॥ ५ ॥
譯文: 第五頌（缺失第 2..4 頌）。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    assert_eq!(res.verdict, "FAIL");
    assert!(res.monotonicity_errors.iter().any(|e| e.contains("漏頌警告") && e.contains("2..4")));
}

#[test]
fn test_unclosed_danda_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一章

Text 1 ．第一頌
यावत् किम् एतद् इति ता अहाहेति नोचुर्

譯文: 原文末尾缺乏雙豎線閉合標點。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    assert_eq!(res.verdict, "FAIL");
    assert!(res.closure_errors.iter().any(|e| e.contains("缺乏雙豎線「॥」閉合標點")));
}

#[test]
fn test_banned_words_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一章

Text 1 ．第一頌
॥ १ ॥
譯文: 我們應當皈依至尊主，登上靈性平台，達到自我證悟境界，跟隨茹阿達達斯學習以太。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    assert_eq!(res.verdict, "FAIL");
    assert!(res.banned_violations.iter().any(|v| v.matched == "皈依"));
    assert!(res.banned_violations.iter().any(|v| v.matched == "平台"));
    assert!(res.banned_violations.iter().any(|v| v.matched == "證悟"));
    assert!(res.banned_violations.iter().any(|v| v.matched == "達斯"));
    assert!(res.banned_violations.iter().any(|v| v.matched == "以太"));
}

#[test]
fn test_filename_bilingual_gate() {
    let pure_chinese = "奇妙月光_繁體中文全譯本.md";
    let errs = check_filename_bilingual(pure_chinese);
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("缺乏原典羅馬拼音"));

    let good = "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md";
    let good_errs = check_filename_bilingual(good);
    assert!(good_errs.is_empty());
}

#[test]
fn test_blind_camatkara_incomplete_chapters_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一道月光：摩納希．甘嘎與珠寶盒篇
Text 1 ．第一頌
यावत् किम् एतद् इति ता अहाहेति नोचुर् ॥ १ ॥
譯文: 第一頌。

## 第二道月光：偷取長笛與阿彼曼紐偽裝開篇
Text 1 ．第一頌
प्रातः पत्तनतनया मनया पदव्या ॥ १ ॥
譯文: 第二章第一頌。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", Some(4), Some(55), Some(200));
    assert_eq!(res.verdict, "FAIL");
    assert!(res.totality_errors.iter().any(|e| e.contains("篇章全體性缺失") && e.contains("應收錄 4 個篇章")));
}

#[test]
fn test_blind_camatkara_chapter4_truncated_below_55_defect() {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一道月光
Text 1 ．第一頌
॥ १ ॥
譯文: 第一章。

## 第二道月光
Text 1 ．第一頌
॥ १ ॥
譯文: 第二章。

## 第三道月光
Text 1 ．第一頌
॥ १ ॥
譯文: 第三章。

## 第四道月光
Text 1 ．第一頌
॥ १ ॥
譯文: 第四章第一頌。

Text 30 ．第三十頌
॥ ३० ॥
譯文: 第四章第三十頌（未達到第 55 頌）。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", Some(4), Some(55), Some(50));
    assert_eq!(res.verdict, "FAIL");
    assert!(res.totality_errors.iter().any(|e| e.contains("末章經文截斷") && e.contains("55")));
}

#[test]
fn test_blind_camatkara_full_4_chapters_golden() {
    let mut verses_md = vec!["# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n".to_string()];
    for ch_name in &["第一道月光", "第二道月光", "第三道月光"] {
        verses_md.push(format!("\n## {ch_name}\n"));
        verses_md.push("Text 1 ．第一頌\n॥ १ ॥\n譯文: 測試開篇。\n".to_string());
    }

    verses_md.push("\n## 第四道月光\n".to_string());
    for v in 1..=55 {
        verses_md.push(format!("Text {v} ．第{v}頌\n॥ {v} ॥\n譯文: 測試第{v}頌。\n"));
    }

    let full_content = verses_md.join("\n");
    let res = inspect_markdown_content(&full_content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", Some(4), Some(55), Some(58));
    assert_eq!(res.verdict, "PASS");
    assert!(res.monotonicity_errors.is_empty());
    assert!(res.totality_errors.is_empty());
}

#[test]
fn test_extract_page_index_from_filename() {
    use rust_bricks::classic_continuity::extract_page_index_from_filename;
    assert_eq!(extract_page_index_from_filename("page_001.png"), Some(1));
    assert_eq!(extract_page_index_from_filename("Scan_042.jpg"), Some(42));
    assert_eq!(extract_page_index_from_filename("chapter1_p150.tif"), Some(150));
    assert_eq!(extract_page_index_from_filename("no_digits_here.png"), None);
}

#[test]
fn test_check_page_sequence_gaps_clean_and_defects() {
    use rust_bricks::classic_continuity::check_page_sequence_gaps;

    // 1. 完全連續 (0 空洞)
    let clean = vec![1, 2, 3, 4, 5];
    let rep = check_page_sequence_gaps(&clean, 1000).expect("應成功");
    assert_eq!(rep.total_files, 5);
    assert_eq!(rep.min_index, 1);
    assert_eq!(rep.max_index, 5);
    assert!(rep.missing_indices.is_empty());
    assert!(rep.duplicate_indices.is_empty());

    // 2. 存在漏頁空洞 (例如缺 3 與 4)
    let with_gaps = vec![1, 2, 5, 6];
    let rep_gap = check_page_sequence_gaps(&with_gaps, 1000).expect("應成功");
    assert_eq!(rep_gap.missing_indices, vec![3, 4]);

    // 3. 存在重複頁
    let with_dup = vec![1, 2, 2, 3];
    let rep_dup = check_page_sequence_gaps(&with_dup, 1000).expect("應成功");
    assert_eq!(rep_dup.duplicate_indices, vec![2]);
}

#[test]
fn test_colophon_anchor_matcher() {
    use rust_bricks::classic_continuity::match_terminal_colophon_anchor;

    // 梵文天城文結尾題記
    let s1 = "इति श्रीविश्वनाथचक्रवर्तिकृता चमत्कारचन्द्रिका सम्पूर्णा समाप्तम् ॥";
    let m1 = match_terminal_colophon_anchor(s1);
    assert!(m1.is_matched);
    assert_eq!(m1.matched_signature, "समाप्तम्");

    // 孟加拉文結尾題記
    let s2 = "ইতি শ্রীচমৎকারচন্দ্রিকা সমাপ্ত ॥";
    let m2 = match_terminal_colophon_anchor(s2);
    assert!(m2.is_matched);
    assert_eq!(m2.matched_signature, "সমাপ্ত");

    // 中文全書終
    let s3 = "此乃第四道月光最後一頌，全書終。願眾生吉祥！";
    let m3 = match_terminal_colophon_anchor(s3);
    assert!(m3.is_matched);
    assert_eq!(m3.matched_signature, "全書終");

    // 截斷無結尾
    let s4 = "這是未完結的句子...";
    let m4 = match_terminal_colophon_anchor(s4);
    assert!(!m4.is_matched);
}

#[test]
fn test_dynamic_zero_config_totality() {
    use rust_bricks::classic_continuity::check_book_totality;

    // 任何書籍：無章節配置時，自適應檢查首頌是否為 1
    // 正常從 1 開始
    let normal_tokens = vec![
        (-1, "第 1 章".to_string(), 1),
        (1, "Text 1".to_string(), 2),
        (2, "Text 2".to_string(), 3),
    ];
    let errs1 = check_book_totality(&normal_tokens, None, None, None);
    assert!(errs1.is_empty());

    // 瑕疵：第 1 章從第 27 頌開始 (掐頭漏洞)
    let truncated_tokens = vec![
        (-1, "第 1 章".to_string(), 1),
        (27, "Text 27".to_string(), 2),
        (28, "Text 28".to_string(), 3),
    ];
    let errs2 = check_book_totality(&truncated_tokens, None, None, None);
    assert_eq!(errs2.len(), 1);
    assert!(errs2[0].contains("篇章首頌異常") && errs2[0].contains("非第 1 頌"));
}
