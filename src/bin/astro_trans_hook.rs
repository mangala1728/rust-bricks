// ## Map
// Role: bin/astro_trans_hook — 吠陀占星翻譯工作流原生高能 Rust 門禁與星盤工具 (Native Hook & CLI)。
// Inputs: 命令列參數 (qa / chart)、待審 docx/md 檔案路徑、命宮上升星座 (1..12)。
// Outputs: 終端審查報告、WordCraft 渲染回饋、SVG 向量星盤。
// Invariants: 零全域狀態、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::Path;
use std::process::{self, Command};

use rust_bricks::astro_kundali_chart;
use rust_bricks::astro_translation_audit::{
    check_astro_banned_words, check_astro_empty_translations, check_astro_english_leakage,
    extract_docx_paragraphs_from_xml,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
astro_trans_hook — 吠陀占星翻譯品質門禁與星盤原生工具 (Rust 原生版)

用法:
  astro_trans_hook qa <docx_file> [--md <md_file>] [--skip-wordcraft]
      執行 .docx/.md 禁字與半形句點掃描、英文層與空白譯文檢測，及 wordcraft-cli 閉環渲染
  astro_trans_hook chart <ascendant_sign_1_to_12> [--type south|north] [--svg <out.svg>]
      產生南印盤/北印盤 ASCII 拓撲或 SVG 向量圖
"#);
}

/// 透過 unzip 萃取 docx 內所有段落純文字
/// @outputs: 2
fn extract_docx_paragraphs(docx_path: &str) -> Result<Vec<String>, String> {
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

    let xml = String::from_utf8(output.stdout).map_err(|e| format!("XML 非有效 UTF-8: {e}"))?;
    Ok(extract_docx_paragraphs_from_xml(&xml))
}

/// 從 wordcraft-cli info 輸出 JSON 解析總頁數
/// @outputs: 1
fn parse_pages_from_json(info_stdout: &str) -> usize {
    let Some(pos) = info_stdout.find("\"pages\"") else {
        return 1;
    };
    let rest = &info_stdout[pos + 7..];
    let Some(colon) = rest.find(':') else {
        return 1;
    };
    let digits: String = rest[colon + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<usize>().unwrap_or(1).max(1)
}

/// 呼叫 wordcraft-cli 解析與渲染前 3 頁
/// @outputs: 2
fn run_wordcraft_loop(docx_path: &str) -> bool {
    println!("🔄 調用 wordcraft-cli 進行排版解析與前 3 頁閉環渲染...");
    let info_out = Command::new("wordcraft-cli")
        .args(["info", docx_path])
        .output();

    let pages = match info_out {
        Ok(output) if output.status.success() => {
            let out_str = String::from_utf8_lossy(&output.stdout);
            println!("✅ WordCraft 解析成功:\n{out_str}");
            parse_pages_from_json(&out_str)
        }
        Ok(output) => {
            eprintln!("❌ wordcraft-cli info 失敗: {}", String::from_utf8_lossy(&output.stderr));
            return false;
        }
        Err(e) => {
            eprintln!("❌ 無法呼叫 wordcraft-cli (請確認已加入 PATH): {e}");
            return false;
        }
    };

    let mut all_ok = true;
    let render_dir = "/tmp/astro_qa_renders";
    let _ = fs::create_dir_all(render_dir);
    for p_num in 1..=pages.min(3) {
        let render_png = format!("{render_dir}/page_{p_num}.png");
        let r_out = Command::new("wordcraft-cli")
            .args(["render", docx_path, &render_png, "--page", &p_num.to_string()])
            .output();
        match r_out {
            Ok(res) if res.status.success() => {
                println!("   🎨 第 {p_num} 頁視覺渲染通過 -> {render_png}");
            }
            Ok(res) => {
                eprintln!("   ❌ 第 {p_num} 頁渲染失敗: {}", String::from_utf8_lossy(&res.stderr));
                all_ok = false;
            }
            Err(e) => {
                eprintln!("   ❌ 無法執行 wordcraft-cli render: {e}");
                all_ok = false;
            }
        }
    }
    all_ok
}

/// 審查 docx 段落與選填 Markdown 文本之禁字、英文洩漏及空譯文
/// @outputs: 2
fn audit_text_layers(docx_path: &str, md_path: Option<&str>) -> bool {
    let mut passed = true;
    let paragraphs = match extract_docx_paragraphs(docx_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("❌ 讀取 docx 失敗: {e}");
            return false;
        }
    };

    let full_text = paragraphs.join("\n");
    let bw_errors = check_astro_banned_words(&full_text);
    if !bw_errors.is_empty() {
        passed = false;
        eprintln!("❌ Docx 發現禁字或標點違規 {} 處：", bw_errors.len());
        for err in &bw_errors {
            eprintln!("   [L{}] 命中「{}」: {}", err.line, err.matched, err.reason);
        }
    } else {
        println!("✅ Docx 禁字與全形句點掃描合格");
    }

    let eng_errors = check_astro_english_leakage(&full_text);
    if !eng_errors.is_empty() {
        passed = false;
        eprintln!("❌ Docx 發現英文層殘留 {} 處：", eng_errors.len());
        for err in &eng_errors {
            eprintln!("   {err}");
        }
    } else {
        println!("✅ Docx 英文層剔除查驗合格");
    }

    let empty_errors = check_astro_empty_translations(&paragraphs);
    if !empty_errors.is_empty() {
        passed = false;
        eprintln!("❌ Docx 發現未填之譯文單元 {} 處：", empty_errors.len());
        for err in &empty_errors {
            eprintln!("   {err}");
        }
    } else {
        println!("✅ Docx 譯文單元完整度合格");
    }

    if let Some(mp) = md_path {
        if !audit_optional_markdown(mp) {
            passed = false;
        }
    }
    passed
}

/// 審查選填 Markdown 檔案之禁字與英文殘留
/// @outputs: 2
fn audit_optional_markdown(md_path: &str) -> bool {
    if !Path::new(md_path).exists() {
        eprintln!("⚠️ 找不到指定的 Markdown 檔案: {md_path}");
        return true;
    }
    let content = match fs::read_to_string(md_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取 Markdown 檔案 '{md_path}': {e}");
            return false;
        }
    };
    let bw = check_astro_banned_words(&content);
    let eng = check_astro_english_leakage(&content);
    if !bw.is_empty() || !eng.is_empty() {
        for err in &bw {
            eprintln!("❌ Markdown [L{}] 命中「{}」: {}", err.line, err.matched, err.reason);
        }
        for err in &eng {
            eprintln!("❌ Markdown {err}");
        }
        return false;
    }
    println!("✅ Markdown 禁字、全形句點與英文層剔除查驗合格: {md_path}");
    true
}

/// 處理 qa 子指令
/// @outputs: 2
fn handle_qa(args: &[String]) {
    if args.len() < 3 {
        eprintln!("錯誤: 缺少 docx 檔案路徑");
        process::exit(1);
    }
    let docx_path = &args[2];
    let mut md_path: Option<&str> = None;
    let mut skip_wordcraft = false;
    let mut i = 3;
    while i < args.len() {
        if args[i] == "--md" && i + 1 < args.len() {
            md_path = Some(&args[i + 1]);
            i += 2;
        } else if args[i] == "--skip-wordcraft" {
            skip_wordcraft = true;
            i += 1;
        } else if !args[i].starts_with('-') && md_path.is_none() {
            md_path = Some(&args[i]);
            i += 1;
        } else {
            i += 1;
        }
    }

    println!("=== [ASTRO TRANS HOOK (RUST)] 開始品質查驗: {docx_path} ===");
    let text_ok = audit_text_layers(docx_path, md_path);
    let wc_ok = if skip_wordcraft { true } else { run_wordcraft_loop(docx_path) };

    if text_ok && wc_ok {
        println!("\n🟢 [ASTRO HOOK] 全項審查通過！");
        process::exit(0);
    } else {
        eprintln!("\n🔴 [ASTRO HOOK] 門禁未通過！");
        process::exit(1);
    }
}

/// 處理 chart 子指令
/// @outputs: 2
fn handle_chart(args: &[String]) {
    if args.len() < 3 {
        eprintln!("錯誤: 缺少上升星座序號 (1..12)");
        process::exit(1);
    }
    let asc: u8 = args[2].parse().unwrap_or(1);
    if !(1..=12).contains(&asc) {
        eprintln!("錯誤: 上升星座必須為 1 到 12");
        process::exit(1);
    }

    println!("=== 吠陀占星星盤產生器 (上升: {}) ===", astro_kundali_chart::rashi_name_zh(asc));
    for h in 1..=12 {
        let sign = astro_kundali_chart::calculate_house_sign(asc, h).unwrap_or(1);
        println!("- 第 {:2} 宮: {}", h, astro_kundali_chart::rashi_name_zh(sign));
    }

    if let Some(pos) = args.iter().position(|a| a == "--svg") {
        if pos + 1 < args.len() {
            let svg_path = &args[pos + 1];
            let svg_content = astro_kundali_chart::generate_north_indian_svg_base(500);
            if let Err(e) = fs::write(svg_path, svg_content) {
                eprintln!("無法寫入 SVG: {e}");
            } else {
                println!("✨ 成功輸出北印盤 SVG: {svg_path}");
            }
        }
    }
}

/// 主入口點
/// @outputs: 2
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "qa" => handle_qa(&args),
        "chart" => handle_chart(&args),
        "-h" | "--help" | "help" => print_usage(),
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
