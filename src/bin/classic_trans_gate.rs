// ## Map
// Role: app (Master QA Gate 整合端點)
// Inputs: 待審 Word .docx 檔案、待審 Markdown .md 檔案 (選填)、--upload 旗標、雲端資料夾 ID。
// Outputs: 雙模態驗收審查報告、Google Drive 同步回饋、剛性退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部第三方依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use rust_bricks::classic_continuity::{
    check_banned_words_and_leakage, check_filename_bilingual, inspect_markdown_content,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
classic_trans_gate — 古典聖典翻譯工作流統一品質閘門 (Rust 原生版)

用法:
  classic_trans_gate <docx_file> [選項]

選項:
  --md <md_file>                 待審之 Markdown .md 檔案路徑
  --upload                       門禁通過後自動同步至 Google Drive「翻譯完成」資料夾
  --drive-parent <folder_id>     Google Drive 目標資料夾 ID
  -h, --help                     顯示此說明訊息
"#);
}

/// 透過系統 unzip 工具萃取 Word docx 的 word/document.xml 純文字
/// @outputs: 4
fn extract_docx_xml(docx_path: &str) -> Result<String, String> {
    let output = Command::new("unzip")
        .args(["-p", docx_path, "word/document.xml"])
        .output()
        .map_err(|e| format!("執行 unzip 失敗: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "無法自 '{docx_path}' 提取 word/document.xml: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("XML 編碼非有效 UTF-8: {e}"))
}

/// 從單一 XML 段落區塊中萃取文字
/// @outputs: 1
fn extract_text_from_p_chunk(p_chunk: &str) -> String {
    let mut p_text = String::new();
    let mut t_search = 0;
    while let Some(t_start) = p_chunk[t_search..].find("<w:t") {
        let abs_t_start = t_search.saturating_add(t_start);
        let tag_close = match p_chunk[abs_t_start..].find('>') {
            Some(tc) => tc,
            None => break,
        };
        let text_begin = abs_t_start.saturating_add(tag_close).saturating_add(1);
        let t_end = match p_chunk[text_begin..].find("</w:t>") {
            Some(te) => te,
            None => break,
        };
        let text_end = text_begin.saturating_add(t_end);
        p_text.push_str(&p_chunk[text_begin..text_end]);
        t_search = text_end.saturating_add("</w:t>".len());
    }
    p_text
}

/// 依序萃取 XML 中所有 <w:p> 段落純文字（控制流扁平化，深度 <= 2）
/// @outputs: 1
fn extract_paragraphs_from_xml(xml: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut search_from = 0;

    while let Some(start) = xml[search_from..].find("<w:p") {
        let abs_start = search_from.saturating_add(start);
        let end_tag = match xml[abs_start..].find("</w:p>") {
            Some(e) => e,
            None => break,
        };
        let abs_end = abs_start.saturating_add(end_tag).saturating_add("</w:p>".len());
        let p_chunk = &xml[abs_start..abs_end];
        let p_text = extract_text_from_p_chunk(p_chunk);
        paragraphs.push(p_text);
        search_from = abs_end;
    }

    paragraphs
}

/// 解析 XML 中的段落與表格並審核禁字與空白譯文
/// @outputs: 2
fn check_docx_integrity(docx_path: &str) -> (bool, usize, usize, Vec<String>) {
    let mut errors = Vec::new();
    let fn_errs = check_filename_bilingual(docx_path);
    errors.extend(fn_errs);

    let xml = match extract_docx_xml(docx_path) {
        Ok(x) => x,
        Err(e) => {
            errors.push(format!("Docx 讀取錯誤: {e}"));
            return (false, 0, 0, errors);
        }
    };

    let table_count = xml.match_indices("<w:tbl").count();
    let paragraphs = extract_paragraphs_from_xml(&xml);
    let p_count = paragraphs.len();

    for (idx, p) in paragraphs.iter().enumerate() {
        let trimmed = p.trim();
        let is_empty_trans = (trimmed == "譯文:" || trimmed == "譯文：")
            && (idx.saturating_add(1) >= paragraphs.len() || paragraphs[idx + 1].trim().is_empty());
        if is_empty_trans {
            errors.push(format!("段落 {idx}: 存在空白未填之「譯文:」單元"));
        }
        let is_defective_anvaya = trimmed.starts_with("- ")
            && (trimmed.contains("[] ➔")
                || trimmed.contains("[] ->")
                || trimmed.ends_with("➔")
                || trimmed.ends_with("->"));
        if is_defective_anvaya {
            errors.push(format!("段落 {idx}: 逐字解包含空中括號或未填入中文逐字義: '{trimmed}'"));
        }
    }

    let all_doc_text = paragraphs.join("\n");
    let violations = check_banned_words_and_leakage(&all_doc_text);
    for v in violations {
        errors.push(format!("Docx 命中禁字「{}」— {}", v.matched, v.reason));
    }

    (errors.is_empty(), p_count, table_count, errors)
}

/// 同步單一檔案至 Google Drive
/// @outputs: 2
fn upload_file_to_drive(file_path: &str, parent_id: &str) {
    if !Path::new(file_path).exists() {
        eprintln!("⚠️ 找不到待上傳檔案: {file_path}");
        return;
    }
    let fname = Path::new(file_path)
        .file_name()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    println!("☁️ [GDRIVE] 正在同步定本至雲端硬碟 (目錄 ID: {parent_id}) -> {fname}");
    let status = Command::new("gdrive")
        .args(["mutate", "upload", file_path, "--parent", parent_id])
        .status();

    match status {
        Ok(s) if s.success() => println!("☁️ [GDRIVE] 成功同步最新定本至雲端硬碟 -> {fname}"),
        Ok(_) => eprintln!("⚠️ [GDRIVE] gdrive 上傳回傳非零狀態碼"),
        Err(e) => eprintln!("⚠️ [GDRIVE] 無法執行 gdrive 指令: {e}"),
    }
}

struct GateOptions {
    docx_path: String,
    md_path: Option<String>,
    upload: bool,
    drive_parent: String,
}

/// 解析 Gate 命令列參數
/// @outputs: 2
fn parse_gate_args(args: &[String]) -> Result<GateOptions, String> {
    if args.len() < 2 {
        return Err("缺少參數".to_string());
    }

    let mut docx_path = None;
    let mut md_path = None;
    let mut upload = false;
    let mut drive_parent = "1xresTeMn_ddALkgFg3sUtxFEI_hpib30".to_string();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--md" && i + 1 < args.len() {
            md_path = Some(args[i + 1].clone());
            i += 2;
        } else if arg == "--upload" {
            upload = true;
            i += 1;
        } else if arg == "--drive-parent" && i + 1 < args.len() {
            drive_parent = args[i + 1].clone();
            i += 2;
        } else if !arg.starts_with('-') && docx_path.is_none() {
            docx_path = Some(arg.clone());
            i += 1;
        } else {
            i += 1;
        }
    }

    match docx_path {
        Some(p) => Ok(GateOptions {
            docx_path: p,
            md_path,
            upload,
            drive_parent,
        }),
        None => Err("未提供待審查之 Word .docx 檔案路徑".to_string()),
    }
}

/// 審查 Markdown 檔案
/// @outputs: 2
fn audit_markdown_file(md_file: &str) -> bool {
    println!("\n🔍 [2/2] 正在審查 Markdown (.md) 經文連續性、雙豎線閉合與雙語完整度...");
    match fs::read_to_string(md_file) {
        Ok(content) => {
            let md_report = inspect_markdown_content(&content, md_file, None, None, None);
            if md_report.verdict == "PASS" {
                println!(
                    "✅ Markdown 門禁通過 (共收錄 {} 頌，0 漏頌、0 殘缺斷句、0 禁字、保留原著英譯供大師校審)",
                    md_report.verses_found_count
                );
                true
            } else {
                println!("❌ Markdown 連續性與品質門禁發現缺陷:");
                for err in &md_report.monotonicity_errors {
                    println!("   • {err}");
                }
                for err in &md_report.closure_errors {
                    println!("   • {err}");
                }
                for v in &md_report.banned_violations {
                    println!("   • 行 {}: 命中「{}」— {}", v.line, v.matched, v.reason);
                }
                for err in &md_report.totality_errors {
                    println!("   • {err}");
                }
                false
            }
        }
        Err(e) => {
            println!("❌ 無法讀取 Markdown 檔案 '{md_file}': {e}");
            false
        }
    }
}

/// 主入口點
/// @outputs: 3
fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = match parse_gate_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("❌ 錯誤: {e}");
            print_usage();
            process::exit(1);
        }
    };

    println!("======================================================================");
    println!("🏛️  古典啟示聖典品質閘門審查 (Master QA Gate Rust 原生版): {}", opts.docx_path);
    println!("======================================================================");

    let mut all_passed = true;
    println!("🔍 [1/2] 正在審查 Word (.docx) 排版完整度與禁字...");
    let (docx_ok, p_count, tbl_count, docx_errors) = check_docx_integrity(&opts.docx_path);
    if docx_ok {
        println!("✅ Word 結構查驗通過 (總段落數: {p_count}, 表格數: {tbl_count})");
    } else {
        all_passed = false;
        println!("❌ Word 結構審查發現缺陷:");
        for err in &docx_errors {
            println!("   • {err}");
        }
    }

    if let Some(ref md_file) = opts.md_path {
        let md_ok = audit_markdown_file(md_file);
        if !md_ok {
            all_passed = false;
        }
    }

    println!("----------------------------------------------------------------------");
    if all_passed {
        println!("🎉 綜合裁決: 【PASS 綠燈】符合 Antigravity 古典聖典出版級高標準，准予發布交付！");
        if opts.upload {
            println!("\n☁️ 正在執行自動發布上傳至雲端硬碟...");
            upload_file_to_drive(&opts.docx_path, &opts.drive_parent);
            if let Some(ref md_file) = opts.md_path {
                upload_file_to_drive(md_file, &opts.drive_parent);
            }
        }
        process::exit(0);
    } else {
        println!("🚫 綜合裁決: 【FAIL 紅燈】存在剛性違規或漏頌漏句，禁止發布！");
        process::exit(1);
    }
}
