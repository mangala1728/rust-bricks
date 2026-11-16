// ## Map
// Role: app (經文連續性檢驗單元測試原生二進位執行器)
// Inputs: 合成測試案例 (黃金樣件、首頌缺失、中間漏頌、標點未閉合、禁字違規、非雙語檔名、篇章缺失、末章截斷、逐字解殘缺)。
// Outputs: 測試報告、PASS / FAIL 狀態、剛性退出碼 (0 = 100% 通過, 1 = 失敗)。
// Invariants: 零外部第三方依賴、純 Rust 實現、禁止 unsafe、函式 <= 100 行、巢狀 <= 2、無弱斷言。

#![forbid(unsafe_code)]

use std::process;
use rust_bricks::classic_continuity::inspect_markdown_content;

/// 測試計數器結構體
struct TestRunner {
    passed: usize,
    failed: usize,
}

impl TestRunner {
    /// 建立全新測試計數器
    /// @outputs: 1
    fn new() -> Self {
        Self { passed: 0, failed: 0 }
    }

    /// 記錄測試結果
    /// @outputs: 1
    fn record(&mut self, test_name: &str, is_ok: bool, detail: &str) {
        if is_ok {
            self.passed += 1;
            println!("  ✅ [PASS] {test_name}");
        } else {
            self.failed += 1;
            eprintln!("  ❌ [FAIL] {test_name}: {detail}");
        }
    }
}

/// 測試 1: 標準連續詩節 (PASS)
/// @outputs: 1
fn test_golden_continuous_verses(runner: &mut TestRunner) {
    let content = r#"# Sample-Classic_示範經文_繁體中文全譯本

## 第一章

Text 1 ．第一頌
यावत् किम् एतद् इति ता अहाहेति नोचुर्
यावद् भृशं जहसुर एव स्व-हस्त-तालम् ।
यावत् त्रपा सहचरी प्रतिबोधम् आप
यावत् प्रमोद-लहरी गतम् उल्ललास ॥ १ ॥

譯文: 當她們尚未說出「啊！這是何物？」時，便已拍掌大笑。

Text 2 ．第二頌
यावन् निवारणम् अङ्गम् अनङ्ग-नक्रो
सर्वः कला-निधिर् अहो युगपच् चुचुम्ब ॥ २ ॥

譯文: 當愛欲巨鱷尚未攫取她抵抗的肢體之際，至尊主親吻了所有牧牛姑娘。

Text 3 ．第三頌
धन्यं भूषणम् अस्तु ते गृह-पतिर् धन्यो यद् आनीतवान्
धन्यं गेहम् इदं यद् एत्य निभृतं मञ्जूषिका खेलति ॥ ३ ॥

譯文: 願這件飾物享有榮耀！願為妳帶來這件飾物的夫婿享有榮耀！
"#;

    let res = inspect_markdown_content(content, "Sample-Classic_示範經文_繁體中文全譯本.md", Some(1), Some(3), Some(3));
    let ok = res.verdict == "PASS"
        && res.monotonicity_errors.is_empty()
        && res.closure_errors.is_empty()
        && res.banned_violations.is_empty()
        && res.totality_errors.is_empty();
    runner.record("test_golden_continuous_verses", ok, "標準連續詩節未通過");
}

/// 測試 2: 起始序號非第 1 頌時觸發完整度缺失 (FAIL)
/// @outputs: 1
fn test_incomplete_starting_verse_defect(runner: &mut TestRunner) {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一章

Text 27 ．第二十七頌
यावत् किम् एतद् इति ता अहाहेति नोचुर् ॥ २७ ॥
譯文: 測試二十七。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    let ok = res.verdict == "FAIL"
        && res.monotonicity_errors.iter().any(|e| e.contains("完整度缺失") && e.contains("第 1..26 頌"));
    runner.record("test_incomplete_starting_verse_defect", ok, "未抓出第 1..26 頌起始完整度缺失");
}

/// 測試 3: 刻意漏頌 (FAIL: 27 跳到 29 漏掉 28)
/// @outputs: 1
fn test_missing_verse_defect(runner: &mut TestRunner) {
    let content = r#"
Text 27 · 第二十七頌
यावत् किम् एतद् इति ता अहाहेति नोचुर् ॥ २७ ॥
譯文: 測試二十七。

Text 29 · 第二十九頌
धन्यं भूषणम् अस्तु ते ॥ २९ ॥
譯文: 測試二十九。
"#;

    let res = inspect_markdown_content(content, "Sample_Book_測試.md", None, None, None);
    let ok = res.verdict == "FAIL"
        && res.monotonicity_errors.iter().any(|e| e.contains("缺失第 [28] 頌"));
    runner.record("test_missing_verse_defect", ok, "未抓出漏掉第 28 頌");
}

/// 測試 4: 標點未閉合 (FAIL: 原文未附雙豎線)
/// @outputs: 1
fn test_unclosed_danda_defect(runner: &mut TestRunner) {
    let content = r#"
Text 27 · 第二十七頌
यावत् किम् एतद् इति ता अहाहेति नोचुर्
譯文: 測試未閉合。
"#;

    let res = inspect_markdown_content(content, "Sample_Book_測試.md", None, None, None);
    let ok = res.verdict == "FAIL"
        && res.closure_errors.iter().any(|e| e.contains("缺乏雙豎線「॥」閉合標點"));
    runner.record("test_unclosed_danda_defect", ok, "未抓出原文未閉合雙豎線");
}

/// 測試 5: 命中剛性禁字 (FAIL: 皈依、達斯)
/// @outputs: 1
fn test_banned_words_defect(runner: &mut TestRunner) {
    let content = r#"
Text 1 ．第一頌
ईशा वास्यमिदं सर्वम् ॥ १ ॥
譯文: 我們應當皈依至尊主，成為祂的忠誠達斯。
"#;

    let res = inspect_markdown_content(content, "Sample_Book_測試.md", None, None, None);
    let matched_words: Vec<&str> = res.banned_violations.iter().map(|v| v.matched.as_str()).collect();
    let ok = res.verdict == "FAIL"
        && matched_words.contains(&"皈依")
        && matched_words.contains(&"達斯");
    runner.record("test_banned_words_defect", ok, "未正確抓出皈依與達斯禁字");
}

/// 測試 6: 檔名雙語規範 (PASS / FAIL)
/// @outputs: 1
fn test_filename_bilingual_gate(runner: &mut TestRunner) {
    let content = "Text 1 ．第一頌\n॥ १ ॥\n譯文: 譯文。\n";

    // 1. 純中文檔名 (缺少羅馬題名)
    let res_bad = inspect_markdown_content(content, "奇妙月光_繁體中文全譯本.md", None, None, None);
    let bad_ok = res_bad.verdict == "FAIL"
        && res_bad.filename_errors.iter().any(|e| e.contains("缺乏原典羅馬拼音/外文題名"));

    // 2. 雙語合規檔名
    let res_good = inspect_markdown_content(content, "Sample-Book_示範經典_繁體中文全譯本.md", None, None, None);
    let good_ok = res_good.filename_errors.is_empty();

    runner.record("test_filename_bilingual_gate", bad_ok && good_ok, "檔名雙語門禁判準異常");
}

/// 測試 7: 篇章全體性缺失 (FAIL: Camatkara-candrika 僅有前兩章)
/// @outputs: 1
fn test_incomplete_chapters_defect(runner: &mut TestRunner) {
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

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    let ok = res.verdict == "FAIL"
        && res.totality_errors.iter().any(|e| e.contains("篇章全體性缺失") && e.contains("預期全書應收錄 4 個篇章"));
    runner.record("test_incomplete_chapters_defect", ok, "未抓出 Camatkara-candrika 僅收錄前兩章之缺失");
}

/// 測試 8: 第四章截斷於第 30 頌 (FAIL: 未達第 55 頌)
/// @outputs: 1
fn test_chapter4_truncated_below_55_defect(runner: &mut TestRunner) {
    let content = r#"# Camatkāra-candrikā_奇妙月光_繁體中文全譯本

## 第一道月光
Text 1 ．第一頌
श्लोक १ ॥ १ ॥
譯文: 第一章。

## 第二道月光
Text 1 ．第一頌
श्लोक १ ॥ १ ॥
譯文: 第二章。

## 第三道月光
Text 1 ．第一頌
श्लोक १ ॥ १ ॥
譯文: 第三章。

## 第四道月光
Text 1 ．第一頌
श्लोक १ ॥ १ ॥
譯文: 第四章第一頌。

Text 30 ．第三十頌
श्लोक ३० ॥ ३० ॥
譯文: 第四章第三十頌（未達到第 55 頌）。
"#;

    let res = inspect_markdown_content(content, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", None, None, None);
    let ok = res.verdict == "FAIL"
        && res.totality_errors.iter().any(|e| e.contains("末章經文截斷") && e.contains("未達到原著全本預期之第 55 頌"));
    runner.record("test_chapter4_truncated_below_55_defect", ok, "未抓出第四章截斷於第 30 頌");
}

/// 測試 9: 4 章齊全且第四章達第 55 頌之全本黃金樣件 (PASS)
/// @outputs: 1
fn test_camatkara_full_4_chapters_golden(runner: &mut TestRunner) {
    let mut verses_md = String::from("# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n");
    for ch_name in &["第一道月光", "第二道月光", "第三道月光"] {
        verses_md.push_str(&format!("\n## {ch_name}\nText 1 ．第一頌\nश्लोक १ ॥ १ ॥\n譯文: 測試開篇。\n"));
    }

    verses_md.push_str("\n## 第四道月光\n");
    for v in 1..=55 {
        verses_md.push_str(&format!("Text {v} ．第{v}頌\nश्लोक {v} ॥ {v} ॥\n譯文: 測試第{v}頌。\n"));
    }

    let res = inspect_markdown_content(&verses_md, "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md", Some(4), Some(55), Some(58));
    let ok = res.verdict == "PASS" && res.monotonicity_errors.is_empty() && res.totality_errors.is_empty();
    runner.record("test_camatkara_full_4_chapters_golden", ok, "四章連續全本黃金樣件未通過");
}

/// 測試 10: 逐字解殘缺或空中括號瑕疵檢驗 (FAIL)
/// @outputs: 1
fn test_anvaya_defective_bracket_and_empty_meaning(runner: &mut TestRunner) {
    let table_md = r#"# Sample-Classic_示範經典_繁體中文全譯本

## 第一章
Text 1 ．第一頌
॥ १ ॥
> **【散文句法與逐字解（Anvaya & Padaccheda）】**：
| 梵文單詞 | 切分 | 格位 | 逐字義 |
| :--- | :--- | :--- | :--- |
| yaḥ | yaḥ | [陽性單數主格] |  |
譯文: 測試。
"#;

    let res_table = inspect_markdown_content(table_md, "Sample-Classic_示範經典_繁體中文全譯本.md", Some(1), Some(1), Some(1));
    let table_ok = res_table.verdict == "FAIL"
        && res_table.totality_errors.iter().any(|e| e.contains("逐字義為空") || e.contains("逐字義空白未填"));

    let list_md = r#"# Sample-Classic_示範經典_繁體中文全譯本

## 第一章
Text 1 ．第一頌
॥ १ ॥
> **【散文句法與逐字解（Anvaya & Padaccheda）】**：
- yaḥ (yaḥ) [] ➔
譯文: 測試。
"#;

    let res_list = inspect_markdown_content(list_md, "Sample-Classic_示範經典_繁體中文全譯本.md", Some(1), Some(1), Some(1));
    let list_ok = res_list.verdict == "FAIL"
        && res_list.totality_errors.iter().any(|e| e.contains("空中括號") || e.contains("未填入中文逐字義"));

    runner.record("test_anvaya_defective_bracket_and_empty_meaning", table_ok && list_ok, "未抓出逐字解空括號或逐字義空白");
}

fn main() {
    println!("======================================================================");
    println!("🧪 經文連續性、標點閉合、禁字與逐字解門禁單元測試: Rust 原生版");
    println!("======================================================================");

    let mut runner = TestRunner::new();

    test_golden_continuous_verses(&mut runner);
    test_incomplete_starting_verse_defect(&mut runner);
    test_missing_verse_defect(&mut runner);
    test_unclosed_danda_defect(&mut runner);
    test_banned_words_defect(&mut runner);
    test_filename_bilingual_gate(&mut runner);
    test_incomplete_chapters_defect(&mut runner);
    test_chapter4_truncated_below_55_defect(&mut runner);
    test_camatkara_full_4_chapters_golden(&mut runner);
    test_anvaya_defective_bracket_and_empty_meaning(&mut runner);

    println!("----------------------------------------------------------------------");
    println!("測試統計: 總計 {} 項測試 | 通過 {} 項 | 失敗 {} 項",
        runner.passed + runner.failed,
        runner.passed,
        runner.failed
    );

    if runner.failed == 0 {
        println!("🎉 【測試全數通過】：Rust 原生測試套件 100% 綠燈！");
        process::exit(0);
    } else {
        eprintln!("🚫 【測試失敗】：存在 {} 項失敗，請檢查演算法！", runner.failed);
        process::exit(1);
    }
}
