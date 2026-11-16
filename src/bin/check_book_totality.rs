// ## Map
// Role: app (CLI 全體性門禁端點)
// Inputs: 經文 Markdown 檔案路徑、預期篇章數、末章最低頌數、全書總頌數下限。
// Outputs: 全體性審查回饋、退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;
use rust_bricks::classic_continuity::{check_book_totality, extract_verse_numbers_from_text};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
check_book_totality — 古典聖典篇章全體性與末章閉合門禁 (Rust 原生版)

用法:
  check_book_totality <markdown_file> [選項]

選項:
  --chapters <N>       預期全書應收錄之篇章總數 (例如 4)
  --final-verse <N>    末章最高頌號下限 (例如 55，防 5x 頌缺失)
  --min-verses <N>     全書經文總頌數下限 (例如 220)
  -h, --help           顯示此說明訊息
"#);
}

struct TotalityOptions {
    target_file: String,
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<TotalityOptions, String> {
    if args.len() < 2 {
        return Err("缺少參數".to_string());
    }

    let mut target_file = None;
    let mut expected_chapters = None;
    let mut min_final_verse = None;
    let mut min_total_verses = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--chapters" && i + 1 < args.len() {
            expected_chapters = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--final-verse" && i + 1 < args.len() {
            min_final_verse = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--min-verses" && i + 1 < args.len() {
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
        Some(f) => Ok(TotalityOptions {
            target_file: f,
            expected_chapters,
            min_final_verse,
            min_total_verses,
        }),
        None => Err("未提供目標 Markdown 檔案路徑".to_string()),
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
    let min_tv = opts.min_total_verses.or(if is_camatkara { Some(220) } else { None });

    println!("=== [BOOK TOTALITY GATE (RUST)] 篇章全體性審查: {} ===", opts.target_file);
    let verse_tokens = extract_verse_numbers_from_text(&content);
    let total_verses = verse_tokens.iter().filter(|v| v.0 != -1).count();
    println!("📊 實際偵測經文總頌數: {total_verses} 頌");

    let errors = check_book_totality(&verse_tokens, exp_ch, min_fv, min_tv);
    println!("----------------------------------------------------------------------");
    if errors.is_empty() {
        println!("✅ 篇章全體性審查通過！\n🎉 全體性門禁判定: 【PASS 綠燈】");
        process::exit(0);
    } else {
        println!("❌ 篇章全體性審查發現缺陷:");
        for err in &errors {
            println!("   • {err}");
        }
        println!("🚫 全體性門禁判定: 【FAIL 紅燈阻斷】");
        process::exit(1);
    }
}
