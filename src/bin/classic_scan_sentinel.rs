// ## Map
// Role: app (實體來源端圖檔序號空洞、重複頁與破損檔案哨兵 CLI)
// Inputs: 來源圖檔目錄路徑、預期頁數下限、容許空洞上限。
// Outputs: 來源端圖檔完整度審查報告、漏頁清單、重複頁清單、損壞檔告警、剛性退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;
use rust_bricks::classic_continuity::{
    check_page_sequence_gaps, extract_page_index_from_filename,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
classic_scan_sentinel — 古典聖典實體來源端圖檔完整度與漏頁哨兵 (Rust 原生版)

用法:
  classic_scan_sentinel <source_dir> [選項]

選項:
  --min-pages <N>      全書最低頁數要求 (防分頁截斷)
  --check-empty        檢查小於 1024 bytes 之損壞或空圖檔 (預設啟用)
  -h, --help           顯示此說明訊息
"#);
}

struct SentinelOptions {
    source_dir: String,
    min_pages: Option<usize>,
    check_empty: bool,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<SentinelOptions, String> {
    if args.len() < 2 {
        return Err("未指定來源目錄".to_string());
    }

    let mut source_dir = None;
    let mut min_pages = None;
    let mut check_empty = true;

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--min-pages" && i + 1 < args.len() {
            min_pages = args[i + 1].parse().ok();
            i += 2;
        } else if arg == "--no-check-empty" {
            check_empty = false;
            i += 1;
        } else if !arg.starts_with('-') && source_dir.is_none() {
            source_dir = Some(arg.clone());
            i += 1;
        } else {
            i += 1;
        }
    }

    match source_dir {
        Some(d) => Ok(SentinelOptions {
            source_dir: d,
            min_pages,
            check_empty,
        }),
        None => Err("未提供目標來源目錄路徑".to_string()),
    }
}

/// 判斷檔案名稱是否為支援之圖檔副檔名
/// @outputs: 1
fn is_supported_image_filename(fname: &str) -> bool {
    let lower = fname.to_lowercase();
    lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".tif")
        || lower.ends_with(".tiff")
        || lower.ends_with(".webp")
        || lower.ends_with(".pgm")
        || lower.ends_with(".ppm")
}

/// 收集目錄下圖檔之序號與檔案大小
/// @outputs: 2
fn collect_image_files(dir_path: &str) -> Result<(Vec<u32>, Vec<String>), String> {
    let read_dir = fs::read_dir(dir_path)
        .map_err(|e| format!("無法讀取目錄 '{dir_path}': {e}"))?;

    let mut indices = Vec::new();
    let mut empty_files = Vec::new();

    for entry in read_dir.flatten() {
        let path = entry.path();
        let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_supported_image_filename(fname) {
            if let Ok(meta) = entry.metadata() {
                if meta.len() < 1024 {
                    empty_files.push(format!("{fname} (大小: {} bytes)", meta.len()));
                }
            }
            if let Some(idx) = extract_page_index_from_filename(fname) {
                indices.push(idx);
            }
        }
    }

    Ok((indices, empty_files))
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

    println!("=== [SCAN SENTINEL (RUST)] 來源端圖檔完整度審查: {} ===", opts.source_dir);

    let (indices, empty_files) = match collect_image_files(&opts.source_dir) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ {e}");
            process::exit(1);
        }
    };

    if indices.is_empty() {
        eprintln!("❌ 錯誤: 目錄 '{}' 中未偵測到任何帶序號之圖檔！", opts.source_dir);
        process::exit(1);
    }

    let report = match check_page_sequence_gaps(&indices, 65535) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("❌ 序號分析失敗: {e}");
            process::exit(1);
        }
    };

    println!("📊 實體圖檔總數: {} 頁，序號跨度: 第 {} 頁 .. 第 {} 頁", report.total_files, report.min_index, report.max_index);

    let mut has_defect = false;

    // 1. 檢驗損壞或空圖檔
    if opts.check_empty && !empty_files.is_empty() {
        has_defect = true;
        println!("❌ 偵測到 {} 個損壞或空圖檔 (< 1KB):", empty_files.len());
        for ef in &empty_files {
            println!("   • {ef}");
        }
    }

    // 2. 檢驗漏頁空洞 (Missing Pages)
    if !report.missing_indices.is_empty() {
        has_defect = true;
        println!("❌ 偵測到 {} 處漏頁空洞 (Missing Pages):", report.missing_indices.len());
        println!("   缺失頁碼: {:?}", report.missing_indices);
    }

    // 3. 檢驗重複頁 (Duplicate Pages)
    if !report.duplicate_indices.is_empty() {
        println!("⚠️ 偵測到重複頁碼: {:?}", report.duplicate_indices);
    }

    // 4. 檢驗最低頁數
    if let Some(min_p) = opts.min_pages {
        if report.total_files < min_p {
            has_defect = true;
            println!("❌ 圖檔總數不足: 實際僅 {} 頁，低於最低門檻 {} 頁 (疑似分頁抓取截斷)！", report.total_files, min_p);
        }
    }

    println!("----------------------------------------------------------------------");
    if has_defect {
        eprintln!("🚫 來源端圖檔存在剛性缺漏，哨兵判定: 【FAIL 紅燈阻斷】(禁止啟動翻譯)");
        process::exit(1);
    } else {
        println!("✅ 來源端實體圖檔完全連續 (0 漏頁、0 損壞檔、跨度自洽)！\n🎉 哨兵判定: 【PASS 綠燈放行】");
        process::exit(0);
    }
}
