// ## Map
// Role: bin/astro_completeness_checker — 吠陀占星譯本完整度審查門禁原生 CLI 工具。
// Inputs: 待審 Markdown/文字檔路徑、--offline、--modal、--json。
// Outputs: 結構完整度清單、離線/雲端系統判定結果、JSON 報告與剛性退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部第三方庫依賴、零全域可變狀態、禁止 unsafe、遵守 09 號合約。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process::{self, Command};

use rust_bricks::astro_translation_audit::{
    analyze_astro_markdown, AstroBookType, AstroCompletenessReport,
};

const MODAL_ENDPOINT: &str =
    "https://mangala1728--nimble-9b-decision-service-serve.us-east.modal.run/v1/systemone";

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
astro_completeness_checker — 吠陀占星譯本完整度審查門禁 (Rust 原生版)

用法:
  astro_completeness_checker <file> [選項]

選項:
  --offline    僅執行本地結構與經文覆蓋率審查，不連線雲端模型
  --modal      強制嘗試呼叫雲端 Modal Bespoke-Nimble-9B（連線失敗仍採本地規則回退）
  --json       以 JSON 格式輸出審查結果
  -h, --help   顯示此說明訊息
"#);
}

struct CliOptions {
    file_path: String,
    offline: bool,
    force_modal: bool,
    as_json: bool,
}

/// 解析命令列參數
/// @outputs: 2
fn parse_cli_args(args: &[String]) -> Result<CliOptions, String> {
    if args.len() < 2 {
        return Err("缺少待審查檔案路徑".to_string());
    }
    let mut file_path = None;
    let mut offline = false;
    let mut force_modal = false;
    let mut as_json = false;

    for arg in args.iter().skip(1) {
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--offline" {
            offline = true;
        } else if arg == "--modal" {
            force_modal = true;
        } else if arg == "--json" {
            as_json = true;
        } else if !arg.starts_with('-') && file_path.is_none() {
            file_path = Some(arg.clone());
        }
    }

    match file_path {
        Some(f) => Ok(CliOptions {
            file_path: f,
            offline,
            force_modal,
            as_json,
        }),
        None => Err("未提供待審查檔案路徑".to_string()),
    }
}

/// 組合送往 Modal SystemOne 的 JSON 請求字串
/// @outputs: 1
fn build_modal_payload(rep: &AstroCompletenessReport) -> String {
    let book_type_criteria = match rep.book_type {
        AstroBookType::Vedanga => (
            "The translation includes the introductory essays, Table A (Annuary for 30 years), Table B (480-year cycle), mathematical calculation proofs, tables of parvas, and all 46 verses. missing_sections is empty.",
            "The translation is missing Table A, Table B, calculations, or major structural sections, or verses are incomplete.",
        ),
        AstroBookType::Garga => (
            "The translation includes the Foreword by Daya Shanker, scholarly Preface by K.K. Pathak, Saraswati invocation, Chapter 1 (Arishta and Arishta-bhanga yogas), Chapter 2 (12x12 House Lords matrix), Chapter 3 (9 planets in 12 houses), Navatara Chakra (9-group nakshatras for family karma), and comprehensive matrix tables. missing_sections is empty.",
            "The translation is missing Foreword, Preface, Chapter 1, Chapter 2 (12 House lords), Chapter 3 (9 planets), or Navatara tables.",
        ),
    };
    format!(
        r#"{{"model":"nimble","state":{{"book":{:?},"char_count":{},"line_count":{},"missing_sections":{:?},"has_tables":{},"table_line_count":{},"verses_found":{},"min_verses_required":{}}},"questions":{{"verdict":{{"type":"choice","instructions":"Determine if the translated book is structurally COMPLETE or INCOMPLETE.","criteria":{{"COMPLETE":{:?},"INCOMPLETE":{:?}}}}}}}}}"#,
        rep.book_title,
        rep.char_count,
        rep.line_count,
        rep.missing_sections,
        rep.has_tables,
        rep.table_line_count,
        rep.verses_found,
        rep.min_verses,
        book_type_criteria.0,
        book_type_criteria.1
    )
}

/// 透過系統 curl 呼叫雲端 Modal 決策服務（失敗回傳 None 以利離線規則回退）
/// @outputs: 2
fn query_modal_choice(rep: &AstroCompletenessReport) -> Option<String> {
    let payload = build_modal_payload(rep);
    let output = Command::new("curl")
        .args([
            "-sS",
            "-m",
            "8",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload,
            MODAL_ENDPOINT,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }
    let body = String::from_utf8_lossy(&output.stdout);
    if body.contains("\"COMPLETE\"") && !body.contains("\"INCOMPLETE\"") {
        return Some("COMPLETE".to_string());
    }
    if let Some(pos) = body.find("\"choice\"") {
        let slice = &body[pos..];
        if slice.contains("\"COMPLETE\"") {
            return Some("COMPLETE".to_string());
        }
        if slice.contains("\"INCOMPLETE\"") {
            return Some("INCOMPLETE".to_string());
        }
    }
    None
}

/// 輸出審查結果 (終端或 JSON)
/// @outputs: 1
fn render_completeness_report(rep: &AstroCompletenessReport, verdict_label: &str, passed: bool, as_json: bool) {
    if as_json {
        println!(
            r#"{{"file":{:?},"book_title":{:?},"char_count":{},"line_count":{},"table_line_count":{},"has_tables":{},"verses_found":{},"min_verses":{},"missing_sections":{:?},"verdict":{:?},"passed":{}}}"#,
            rep.file_path,
            rep.book_title,
            rep.char_count,
            rep.line_count,
            rep.table_line_count,
            rep.has_tables,
            rep.verses_found,
            rep.min_verses,
            rep.missing_sections,
            verdict_label,
            passed
        );
        return;
    }

    println!("==================================================");
    println!("🔍 吠陀占星譯本完整度審查門禁 (Completeness Gate RUST)");
    println!("==================================================");
    println!("待審查檔案: {}", rep.file_path);
    println!("\n📊 本地特徵解析:");
    println!("  - 識別典籍: {}", rep.book_title);
    println!("  - 總字數: {} 字", rep.char_count);
    println!("  - 總行數: {} 行", rep.line_count);
    println!("  - 表格行數: {} 行 (包含表格: {})", rep.table_line_count, rep.has_tables);
    println!("  - 覆蓋經文頌數: {} 頌 (最低要求: {} 頌)", rep.verses_found, rep.min_verses);

    println!("\n📑 各必備模組審核狀態:");
    for sec in &rep.sections {
        let mark = if sec.found { "✅ 具備" } else { "❌ 缺失" };
        println!("  [{mark}] {}", sec.name);
    }

    if passed {
        println!("\n🎉 審查通過 ({verdict_label}): 譯本結構完整（具備表格、算理、導言與經文）。");
    } else {
        println!("\n❌ 審查未通過 ({verdict_label}): 缺漏模組 {:?}", rep.missing_sections);
    }
}

/// 主入口點
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

    let content = match fs::read_to_string(&opts.file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取檔案 '{}': {e}", opts.file_path);
            process::exit(1);
        }
    };

    let rep = analyze_astro_markdown(&content, &opts.file_path);
    let modal_choice = if !opts.offline && opts.force_modal {
        query_modal_choice(&rep)
    } else {
        None
    };

    let (verdict_label, passed) = match modal_choice.as_deref() {
        Some("COMPLETE") => ("MODAL_COMPLETE", true),
        Some("INCOMPLETE") => ("MODAL_INCOMPLETE", false),
        _ => (if rep.local_passed { "LOCAL_PASS" } else { "LOCAL_FAIL" }, rep.local_passed),
    };

    render_completeness_report(&rep, verdict_label, passed, opts.as_json);
    process::exit(if passed { 0 } else { 1 });
}
