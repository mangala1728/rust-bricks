// ## Map
// Role: tests/msa_classic_continuity.rs — 古典聖典翻譯品質量具 MSA 盲測驗收測試。
// Inputs: msa/classic_continuity.rs 標準件 (2) 與挑戰件 (7)。
// Outputs: 鑑別力（Recall / Precision）、重複性（Repeatability = 1.00）檢定報告與斷言。
// Invariants: 零外部依賴、純 Rust 實現、斷言 Recall >= 0.90、Precision >= 0.90、Repeatability == 1.00。

#![forbid(unsafe_code)]

#[path = "../msa/classic_continuity.rs"]
mod parts;

use parts::{CHALLENGE_DEFECT_PARTS, STANDARD_PARTS};
use rust_bricks::classic_continuity::inspect_markdown_content;

#[test]
fn test_msa_classic_continuity_recall_precision_repeatability() {
    println!("\n=== [MSA GAUGE CALIBRATION] 古典聖典翻譯門禁量測系統分析 ===");

    let runs = 3;
    let mut all_run_results = Vec::new();

    for run_idx in 1..=runs {
        let mut tp = 0; // 成功攔截之挑戰件 (True Positive)
        let mut fn_count = 0; // 挑戰件漏放 (False Negative)
        let mut tn = 0; // 標準件綠燈放行 (True Negative)
        let mut fp = 0; // 標準件誤殺 (False Positive)

        // 1. 量測標準件 (Standard / Golden Parts) -> 預期全部 PASS
        for std_case in STANDARD_PARTS {
            let res = inspect_markdown_content(
                std_case.content,
                std_case.filename,
                std_case.exp_chapters,
                std_case.min_final_verse,
                std_case.min_total_verses,
            );
            if res.verdict == "PASS" {
                tn += 1;
            } else {
                fp += 1;
                eprintln!(
                    "[Run {run_idx}] 標準件誤殺: {} (Errors: {:?})",
                    std_case.name, res.monotonicity_errors
                );
            }
        }

        // 2. 量測挑戰瑕疵件 (Challenge / Defect Parts) -> 預期全部 FAIL
        for ch_case in CHALLENGE_DEFECT_PARTS {
            let res = inspect_markdown_content(
                ch_case.content,
                ch_case.filename,
                ch_case.exp_chapters,
                ch_case.min_final_verse,
                ch_case.min_total_verses,
            );
            if res.verdict == "FAIL" {
                tp += 1;
            } else {
                fn_count += 1;
                eprintln!("[Run {run_idx}] 挑戰件漏網: {}", ch_case.name);
            }
        }

        let recall = if (tp + fn_count) > 0 {
            (tp as f64) / ((tp + fn_count) as f64)
        } else {
            1.0
        };

        let precision = if (tp + fp) > 0 {
            (tp as f64) / ((tp + fp) as f64)
        } else {
            1.0
        };

        println!(
            "Run {run_idx}: TP={tp}, TN={tn}, FP={fp}, FN={fn_count} | Recall={:.2}%, Precision={:.2}%",
            recall * 100.0,
            precision * 100.0
        );

        all_run_results.push((tp, tn, fp, fn_count, recall, precision));
    }

    // 重複性檢定 (Repeatability): 3 輪結果必須 100% 相同
    let first = all_run_results[0];
    let repeatable = all_run_results.iter().all(|r| *r == first);
    println!("量具重複性 (Repeatability): {}", if repeatable { "1.00 (完全吻合)" } else { "FAILED" });

    assert!(repeatable, "量具重複性未達到 1.00！");
    assert!(first.4 >= 0.90, "Recall 未達標 (需 >= 90.0%): {:.2}%", first.4 * 100.0);
    assert!(first.5 >= 0.90, "Precision 未達標 (需 >= 90.0%): {:.2}%", first.5 * 100.0);

    println!("✅ 量測系統分析 (MSA) 全項合格！量具準確度與鑑別力達規，准予量產使用。");
}
