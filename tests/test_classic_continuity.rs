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
