// ## Map
// Role: app (CLI 經文連續性檢驗端點)
// Inputs: 命令列參數 (Markdown 檔案路徑、--json、篇章/頌數約束)。
// Outputs: 終端審查報告、JSON 輸出、剛性退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;
use rust_bricks::classic_continuity::{inspect_markdown_content, QualityReport};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
classic_continuity_checker — 古典聖典經文連續性、漏頁、漏句與剛性禁字品質門禁 (Rust 原生版)

用法:
  classic_continuity_checker <markdown_file> [選項]

選項:
  --json                     以 JSON 格式輸出審查報告
  --expected-chapters <N>    預期全書應收錄之篇章總數 (例如 4)
  --min-final-verse <N>      末章最高頌號下限 (例如 55，防 5x 頌缺失)
  --min-total-verses <N>     全書經文總頌數下限 (例如 200)
  -h, --help                 顯示此說明訊息
"#);
}

struct CliOptions {
    target_file: String,
    as_json: bool,
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<CliOptions, String> {
    if args.len() < 2 {
        return Err("缺少參數".to_string());
    }

    let mut target_file = None;
    let mut as_json = false;
    let mut expected_chapters = None;
    let mut min_final_verse = None;
    let mut min_total_verses = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--json" {
            as_json = true;
            i += 1;
        } else if arg == "--expected-chapters" && i + 1 < args.len() {
            expected_chapters = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--min-final-verse" && i + 1 < args.len() {
            min_final_verse = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--min-total-verses" && i + 1 < args.len() {
            min_total_verses = args[i + 1].parse().ok();
            i += 2;
        } else if !arg.starts_with('-') && target_file.is_none() {
            target_file = Some(arg.clone());
            i += 1;
        } else {
            i += 1;
        }
    }

    match target_file {
        Some(f) => Ok(CliOptions {
            target_file: f,
            as_json,
            expected_chapters,
            min_final_verse,
            min_total_verses,
        }),
        None => Err("未提供目標 Markdown 檔案路徑".to_string()),
    }
}

/// 渲染終端或 JSON 報告
/// @outputs: 1
fn render_report(report: &QualityReport, as_json: bool) {
    if as_json {
        let mut violations_json = String::new();
        for (v_idx, v) in report.banned_violations.iter().enumerate() {
            if v_idx > 0 {
                violations_json.push_str(", ");
            }
            violations_json.push_str(&format!(
                r#"{{"line": {}, "matched": {:?}, "reason": {:?}, "snippet": {:?}}}"#,
                v.line, v.matched, v.reason, v.snippet
            ));
        }

        println!(
            r#"{{"file": {:?}, "verdict": {:?}, "verses_found_count": {}, "verse_sample": {:?}, "filename_errors": {:?}, "monotonicity_errors": {:?}, "closure_errors": {:?}, "banned_violations": [{}], "totality_errors": {:?}}}"#,
            report.file_name,
            report.verdict,
            report.verses_found_count,
            report.verse_sample,
            report.filename_errors,
            report.monotonicity_errors,
            report.closure_errors,
            violations_json,
            report.totality_errors
        );
        return;
    }

    println!("=== [CLASSIC QA HOOK (RUST)] 經文連續性與品質門禁審查: {} ===", report.file_name);
    println!("📊 偵測到經文詩節數: {} 頌", report.verses_found_count);
    if !report.verse_sample.is_empty() {
        println!("📑 詩節取樣: {:?} ...", report.verse_sample);
    }

    print_section_errors("檔名雙語規範", &report.filename_errors);
    print_section_errors("連續性與漏頌", &report.monotonicity_errors);
    print_section_errors("原文標點與閉合", &report.closure_errors);

    if !report.banned_violations.is_empty() {
        println!("\n❌ 【禁字與英文殘留違規】:");
        for v in &report.banned_violations {
            println!("   • 行 {}: 命中「{}」— {}", v.line, v.matched, v.reason);
        }
    } else {
        println!("✅ 禁字與英文殘留掃描: 通過 (0 皈依、0 平台、0 證悟、0 達斯、0 以太、0 英文殘留)");
    }

    print_section_errors("篇章全體性與末章閉合", &report.totality_errors);
}

/// 輔助印出單項檢驗結果
/// @outputs: 1
fn print_section_errors(title: &str, errors: &[String]) {
    if !errors.is_empty() {
        println!("\n❌ 【{title}缺陷】:");
        for err in errors {
            println!("   • {err}");
        }
    } else {
        println!("✅ {title}: 通過");
    }
}

/// CLI 主入口點
/// @outputs: 3
fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = match parse_cli_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("❌ 錯誤: {e}");
            print_usage();
            process::exit(1);
        }
    };

    let content = match fs::read_to_string(&opts.target_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取檔案 '{}': {e}", opts.target_file);
            process::exit(1);
        }
    };

    let lower_name = opts.target_file.to_lowercase();
    let is_camatkara = lower_name.contains("camatk") || opts.target_file.contains("奇妙月光");
    let exp_ch = opts.expected_chapters.or(if is_camatkara { Some(4) } else { None });
    let min_fv = opts.min_final_verse.or(if is_camatkara { Some(55) } else { None });
    let min_tv = opts.min_total_verses.or(if is_camatkara { Some(200) } else { None });

    let report = inspect_markdown_content(&content, &opts.target_file, exp_ch, min_fv, min_tv);
    render_report(&report, opts.as_json);

    println!("----------------------------------------------------------------------");
    if report.verdict == "PASS" {
        println!("🎉 審查結論: 【PASS 綠燈放行】");
        process::exit(0);
    } else {
        println!("🚫 審查結論: 【FAIL 紅燈阻斷】請依上述錯誤清單修正後重新驗收！");
        process::exit(1);
    }
}
