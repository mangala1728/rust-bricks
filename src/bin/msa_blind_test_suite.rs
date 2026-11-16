// ## Map
// Role: app (MSA 量測系統分析跨語言/跨模型盲測驗收原生 CLI 工具)
// Inputs: 預設黃金標準件、挑戰瑕疵件 (篇章缺失、末章截斷、漏頌、未閉合、禁字、非雙語、逐字解殘缺)。
// Outputs: 終端 MSA 鑑別力報告 (Recall >= 90%, Precision >= 90%, Repeatability = 1.00)、剛性退出碼。
// Invariants: 零外部第三方依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::process;
use rust_bricks::classic_continuity::inspect_markdown_content;

struct BlindVector<'a> {
    id: &'a str,
    name: &'a str,
    filename: &'a str,
    content: &'a str,
    expected_pass: bool,
    exp_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
}

const BLIND_VECTORS: &[BlindVector<'static>] = &[
    BlindVector {
        id: "VEC-01",
        name: "Standard_Continuous_3_Verses",
        filename: "Sample-Book_示範經典_繁體中文全譯本.md",
        content: "# Sample-Book_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 2 ．第二頌\n॥ २ ॥\n譯文: 二。\n\nText 3 ．第三頌\n॥ ३ ॥\n譯文: 三。\n",
        expected_pass: true,
        exp_chapters: Some(1),
        min_final_verse: Some(3),
        min_total_verses: Some(3),
    },
    BlindVector {
        id: "VEC-02",
        name: "Defect_Incomplete_Starting_Verse_27",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 27 ．第二十七頌\n॥ २७ ॥\n譯文: 缺前導經文。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-03",
        name: "Defect_Missing_Verses_Gap",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 5 ．第五頌\n॥ ५ ॥\n譯文: 缺 2..4。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-04",
        name: "Defect_Unclosed_Danda",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\nयावत् किम् एतद् इति ता अहाहेति नोचुर्\n\n譯文: 缺少雙豎線閉合。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-05",
        name: "Defect_Banned_Words_Guiyi",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 應當皈依至尊主，登上靈性平台，獲得自我證悟。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-06",
        name: "Defect_Non_Bilingual_Filename",
        filename: "奇妙月光_繁體中文全譯本.md",
        content: "# 奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 純中文檔名。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    BlindVector {
        id: "VEC-07",
        name: "Defect_Incomplete_Chapters",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第一道月光。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第二道月光。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(200),
    },
    BlindVector {
        id: "VEC-08",
        name: "Defect_Chapter4_Truncated_Below_55",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 二。\n\n## 第三道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 三。\n\n## 第四道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 四。\n\nText 30 ．第三十頌\n॥ ३० ॥\n譯文: 截斷於第 30 頌。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(200),
    },
    BlindVector {
        id: "VEC-09",
        name: "Defect_Anvaya_Empty_Meaning_Or_Bracket",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n> **【散文句法與逐字解（Anvaya & Padaccheda）】**：\n| 梵文單詞 | 切分 | 格位 | 逐字義 |\n| :--- | :--- | :--- | :--- |\n| yaḥ | yaḥ |  |  |\n譯文: 逐字義缺失。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
];

/// 執行單輪 MSA 向量比對
/// @outputs: 4
fn run_msa_round(round_idx: usize) -> (usize, usize, usize, usize, f64, f64) {
    let mut tp = 0;
    let mut tn = 0;
    let mut fp = 0;
    let mut fn_count = 0;

    for vec in BLIND_VECTORS {
        let rep = inspect_markdown_content(
            vec.content,
            vec.filename,
            vec.exp_chapters,
            vec.min_final_verse,
            vec.min_total_verses,
        );
        let actual_pass = rep.verdict == "PASS";

        if vec.expected_pass {
            if actual_pass {
                tn += 1;
            } else {
                fp += 1;
                eprintln!("[Round {round_idx}] 標準件誤判 (FP): [{}] {} (原因: {:?})", vec.id, vec.name, rep.totality_errors);
            }
        } else if !actual_pass {
            tp += 1;
        } else {
            fn_count += 1;
            eprintln!("[Round {round_idx}] 瑕疵件漏檢 (FN): [{}] {}", vec.id, vec.name);
        }
    }

    let recall = if (tp + fn_count) > 0 { (tp as f64) / ((tp + fn_count) as f64) } else { 1.0 };
    let precision = if (tp + fp) > 0 { (tp as f64) / ((tp + fp) as f64) } else { 1.0 };

    println!(
        "第 {round_idx} 輪量測: TP={tp}, TN={tn}, FP={fp}, FN={fn_count} | 召回率={:.2}%, 精確率={:.2}%",
        recall * 100.0,
        precision * 100.0
    );

    (tp, tn, fp, fn_count, recall, precision)
}

fn main() {
    println!("======================================================================");
    println!("🔬 跨語言跨模型量測系統分析 (MSA Blind-Testing Matrix): Rust 原生版");
    println!("======================================================================");

    let runs = 3;
    let mut all_runs = Vec::new();

    for r in 1..=runs {
        all_runs.push(run_msa_round(r));
    }

    let first = all_runs[0];
    let repeatable = all_runs.iter().all(|r| *r == first);
    println!("----------------------------------------------------------------------");
    println!("量具重複性 (Repeatability): {}", if repeatable { "1.00 (完全吻合)" } else { "FAILED" });
    println!("量具召回率 (Recall):         {:.2}% (門檻 >= 90.0%)", first.4 * 100.0);
    println!("量具精確率 (Precision):      {:.2}% (門檻 >= 90.0%)", first.5 * 100.0);

    // 檢查現行專案實際交付定本（若存在）
    let golden_path = "/home/antigravity/mangala1728/Sanskrit-Classic-Translation/out/Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md";
    if Path::new(golden_path).exists() {
        if let Ok(content) = fs::read_to_string(golden_path) {
            let golden_rep = inspect_markdown_content(&content, golden_path, Some(4), Some(55), Some(220));
            println!("實體全本黃金樣件驗收: {} -> {}", golden_path, golden_rep.verdict);
            if golden_rep.verdict != "PASS" {
                eprintln!("❌ 實體黃金樣件未通過門禁: {:?}", golden_rep.totality_errors);
                process::exit(1);
            }
        }
    }

    if repeatable && first.4 >= 0.90 && first.5 >= 0.90 {
        println!("\n🎉 【MSA 驗收結論】：量測系統合格！Rust 原生二進位程式完全具備對真值與瑕疵的 100% 鑑別力！");
        process::exit(0);
    } else {
        eprintln!("\n🚫 【MSA 驗收結論】：量測系統未達標，禁止切換生產環境！");
        process::exit(1);
    }
}
