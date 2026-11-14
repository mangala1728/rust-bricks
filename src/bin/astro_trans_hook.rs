// ## Map
// Role: bin/astro_trans_hook — 吠陀占星翻譯工作流原生高能 Rust 門禁與星盤工具 (Native Hook & CLI)。
// Inputs: 命令列參數 (qa / chart)、待審 docx/md 檔案路徑、命宮上升星座 (1..12)。
// Outputs: 終端審查報告、WordCraft 渲染回饋、SVG 向量星盤。
// Invariants: 零全域狀態、禁止 unsafe 代碼、100% 遵守 09 號合約、高敏捷無依賴。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process::{self, Command};
use rust_bricks::astro_kundali_chart;

fn print_usage() {
    eprintln!(r#"
astro_trans_hook — 吠陀占星翻譯品質門禁與星盤原生工具 (Rust 原生版)

用法:
  astro_trans_hook qa <docx_file> [--md <md_file>]
      執行禁字快速掃描、英文層檢測與調用 wordcraft-cli 閉環渲染驗證
  astro_trans_hook chart <ascendant_sign_1_to_12> [--type south|north] [--svg <out.svg>]
      產生南印盤/北印盤 ASCII 拓撲或 SVG 向量圖
"#);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "qa" => {
            if args.len() < 3 {
                eprintln!("錯誤: 缺少 docx 檔案路徑");
                process::exit(1);
            }
            let docx_path = &args[2];
            let mut md_path: Option<&str> = None;
            let mut i = 3;
            while i < args.len() {
                if args[i] == "--md" && i + 1 < args.len() {
                    md_path = Some(&args[i + 1]);
                    i += 2;
                } else {
                    i += 1;
                }
            }

            println!("=== [ASTRO TRANS HOOK (RUST)] 開始品質查驗: {docx_path} ===");
            let mut has_failure = false;

            // 1. 若有 md_path，直接讀取內容進行高敏捷禁字檢查
            if let Some(mp) = md_path {
                match fs::read_to_string(mp) {
                    Ok(content) => {
                        println!("🔍 正在掃描 Markdown 檔禁字: {mp}");
                        if let Some(reason) = astro_kundali_chart::validate_banned_words_slice(&content) {
                            eprintln!("❌ 命中禁字規則: {reason}");
                            has_failure = true;
                        } else {
                            println!("✅ Markdown 禁字掃描通過（0 皈依、0 平台、0 證悟、0 達斯、0 以太）");
                        }

                        // 英文層殘留檢查
                        if content.contains("Translation:") {
                            eprintln!("❌ 發現殘留英文標籤 'Translation:'！專案主人規定不留英文版");
                            has_failure = true;
                        } else {
                            println!("✅ 英文層剔除查驗通過");
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️ 無法讀取 Markdown 檔案: {e}");
                    }
                }
            }

            // 2. 調用 wordcraft-cli 進行資訊萃取與渲染驗證
            println!("🔄 調用 wordcraft-cli 進行排版解析與閉環渲染...");
            let info_out = Command::new("wordcraft-cli")
                .arg("info")
                .arg(docx_path)
                .output();

            match info_out {
                Ok(output) if output.status.success() => {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    println!("✅ WordCraft 解析成功:\n{out_str}");

                    // 渲染第一頁
                    let render_png = "/tmp/astro_qa_rust_page1.png";
                    let render_out = Command::new("wordcraft-cli")
                        .args(["render", docx_path, render_png, "--page", "1"])
                        .output();

                    match render_out {
                        Ok(r_out) if r_out.status.success() => {
                            let r_str = String::from_utf8_lossy(&r_out.stdout);
                            println!("🎨 第 1 頁視覺渲染通過: {r_str}");
                        }
                        Ok(r_out) => {
                            eprintln!("❌ 第 1 頁渲染失敗: {}", String::from_utf8_lossy(&r_out.stderr));
                            has_failure = true;
                        }
                        Err(e) => {
                            eprintln!("❌ 無法執行 wordcraft-cli render: {e}");
                            has_failure = true;
                        }
                    }
                }
                Ok(output) => {
                    eprintln!("❌ wordcraft-cli info 失敗: {}", String::from_utf8_lossy(&output.stderr));
                    has_failure = true;
                }
                Err(e) => {
                    eprintln!("❌ 無法呼叫 wordcraft-cli (請確認已加入 PATH): {e}");
                    has_failure = true;
                }
            }

            if has_failure {
                eprintln!("\n🔴 [ASTRO HOOK] 門禁未通過！");
                process::exit(1);
            } else {
                println!("\n🟢 [ASTRO HOOK] 全項審查通過！");
                process::exit(0);
            }
        }
        "chart" => {
            if args.len() < 3 {
                eprintln!("錯誤: 缺少上升星座序號 (1..12)");
                process::exit(1);
            }
            let asc: u8 = args[2].parse().unwrap_or(1);
            if asc < 1 || asc > 12 {
                eprintln!("錯誤: 上升星座必須為 1 到 12");
                process::exit(1);
            }

            println!("=== 吠陀占星星盤產生器 (上升: {}) ===", astro_kundali_chart::rashi_name_zh(asc));
            for h in 1..=12 {
                let sign = astro_kundali_chart::calculate_house_sign(asc, h).unwrap_or(1);
                println!("- 第 {:2} 宮: {}", h, astro_kundali_chart::rashi_name_zh(sign));
            }

            // 若有 --svg
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
        _ => {
            print_usage();
            process::exit(1);
        }
    }
}
