// ## Map
// Role: app (CLI 全體性門禁端點)
// Inputs: 經文 Markdown 檔案路徑、預期篇章數、末章最低頌數、全書總頌數下限、來源圖檔目錄。
// Outputs: 全體性審查回饋、實體頁碼空洞報告、結尾題記審查、退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;
use rust_bricks::classic_continuity::{
    check_book_totality_advanced, check_page_sequence_gaps,
    extract_page_index_from_filename, extract_verse_numbers_from_text,
    parse_registry_expectations,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
check_book_totality — 古典聖典篇章全體性與末章閉合門禁 (Rust 原生版)

用法:
  check_book_totality <markdown_file> [選項]

選項:
  --source-dir <dir>   來源圖檔目錄 (自動檢驗實體檔案序號空洞與漏頁)
  --colophon           強制要求經文末尾必須命中印度古典結尾題記 (Iti Samāptam)
  --chapters <N>       預期全書應收錄之篇章總數 (選填，無配置時採註冊表/文獻推導)
  --final-verse <N>    末章最高頌號下限 (選填)
  --min-verses <N>     全書經文總頌數下限 (選填)
  --registry <path>    指定 classics_title_registry.json 路徑
  -h, --help           顯示此說明訊息
"#);
}

struct TotalityOptions {
    target_file: String,
    source_dir: Option<String>,
    require_colophon: bool,
    expected_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
    registry_path: Option<String>,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<TotalityOptions, String> {
    if args.len() < 2 {
        return Err("缺少參數".to_string());
    }

    let mut target_file = None;
    let mut source_dir = None;
    let mut require_colophon = false;
    let mut expected_chapters = None;
    let mut min_final_verse = None;
    let mut min_total_verses = None;
    let mut registry_path = None;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--source-dir" && i + 1 < args.len() {
            source_dir = Some(args[i + 1].clone());
            i += 2;
        } else if arg == "--colophon" {
            require_colophon = true;
            i += 1;
        } else if arg == "--chapters" && i + 1 < args.len() {
            expected_chapters = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--final-verse" && i + 1 < args.len() {
            min_final_verse = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--min-verses" && i + 1 < args.len() {
            min_total_verses = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--registry" && i + 1 < args.len() {
            registry_path = Some(args[i + 1].clone());
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
            source_dir,
            require_colophon,
            expected_chapters,
            min_final_verse,
            min_total_verses,
            registry_path,
        }),
        None => Err("未提供目標 Markdown 檔案路徑".to_string()),
    }
}

/// 自動搜尋並讀取 classics_title_registry.json
/// @outputs: 2
fn resolve_registry_expectations(
    registry_opt: Option<&str>,
    target_file: &str,
) -> (Option<usize>, Option<i32>, Option<usize>) {
    let mut candidates = Vec::new();
    if let Some(p) = registry_opt {
        candidates.push(p.to_string());
    }
    candidates.push("reference_rules/classics_title_registry.json".to_string());
    candidates.push("../agent-workflows/workflows/sanskrit-classic-translation/reference_rules/classics_title_registry.json".to_string());
    candidates.push("workflows/sanskrit-classic-translation/reference_rules/classics_title_registry.json".to_string());
    candidates.push("/home/antigravity/mangala1728/agent-workflows/workflows/sanskrit-classic-translation/reference_rules/classics_title_registry.json".to_string());

    for cand in candidates {
        if let Ok(reg_text) = fs::read_to_string(&cand) {
            return parse_registry_expectations(&reg_text, target_file);
        }
    }
    (None, None, None)
}

/// 掃描實體來源圖檔目錄並檢驗序號空洞
/// @outputs: 2
fn audit_source_directory_gaps(dir_path: &str) -> Result<String, String> {
    let read_dir = fs::read_dir(dir_path)
        .map_err(|e| format!("無法讀取圖檔目錄 '{dir_path}': {e}"))?;

    let mut indices = Vec::new();
    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let lower = fname.to_lowercase();
        let is_img = lower.ends_with(".png")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
            || lower.ends_with(".tif")
            || lower.ends_with(".webp");

        if is_img {
            if let Some(idx) = extract_page_index_from_filename(fname) {
                indices.push(idx);
            }
        }
    }

    if indices.is_empty() {
        return Ok(format!("⚠️ 來源目錄 '{dir_path}' 內未偵測到帶序號之圖檔，跳過實體序號檢查。"));
    }

    let report = check_page_sequence_gaps(&indices, 65535)
        .map_err(|e| format!("實體序號檢驗失敗: {e}"))?;

    if !report.missing_indices.is_empty() {
        return Err(format!(
            "❌ 來源端偵測到 {} 處漏頁空洞 (Missing Pages): {:?} (實體掃描總檔數: {}, 序號跨度: {}..{})！",
            report.missing_indices.len(),
            report.missing_indices,
            report.total_files,
            report.min_index,
            report.max_index
        ));
    }

    Ok(format!(
        "✅ 實體圖檔序號連續 (0 漏頁): 共 {} 個檔案，頁碼連續無空洞 ({}..{})",
        report.total_files, report.min_index, report.max_index
    ))
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

    println!("=== [BOOK TOTALITY GATE (RUST)] 篇章全體性審查: {} ===", opts.target_file);

    // 1. 若有指定來源目錄，先執行實體檔案空洞守門員 (Gap Sentinel)
    if let Some(ref dir) = opts.source_dir {
        println!("🔍 [1/2] 正在檢驗來源端實體圖檔序號連續性: {dir} ...");
        match audit_source_directory_gaps(dir) {
            Ok(msg) => println!("{msg}"),
            Err(e) => {
                eprintln!("{e}\n🚫 來源圖檔存在漏頁，全體性門禁判定: 【FAIL 紅燈阻斷】");
                process::exit(1);
            }
        }
    }

    // 2. 檢驗經文文本內容
    let content = match fs::read_to_string(&opts.target_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取檔案 '{}': {e}", opts.target_file);
            process::exit(1);
        }
    };

    let verse_tokens = extract_verse_numbers_from_text(&content);
    let total_verses = verse_tokens.iter().filter(|v| v.0 != -1).count();
    println!("📊 實際偵測經文總頌數: {total_verses} 頌");

    let (reg_ch, reg_fv, reg_tv) = resolve_registry_expectations(
        opts.registry_path.as_deref(),
        &opts.target_file,
    );
    let exp_ch = opts.expected_chapters.or(reg_ch);
    let min_fv = opts.min_final_verse.or(reg_fv);
    let min_tv = opts.min_total_verses.or(reg_tv);

    let errors = check_book_totality_advanced(
        &verse_tokens,
        &content,
        exp_ch,
        min_fv,
        min_tv,
        opts.require_colophon,
    );

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
