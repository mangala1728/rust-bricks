// ## Map
// Role: bin/subtitle_qa_gate — 影視字幕 CPS 閱讀速度、殘留英文字與 VAD 邊界同步驗收原生 CLI。
// Inputs: subcommand (cps / vad-qa / check-eng)、SRT 字幕路徑、boundaries.json 路徑與門檻選項。
// Outputs: 終端審查報告、JSON 輸出、貼合修正後 SRT、剛性退出碼 (0 = PASS, 1 = FAIL)。
// Invariants: 零外部依賴、零全域可變狀態、禁止 unsafe 代碼、遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::process;

use rust_bricks::subtitle_qa_audit::{
    audit_srt_cps, evaluate_vad_boundaries, find_residual_english_lines, format_srt_timestamp_ms,
    parse_speech_boundaries_json, parse_srt_content, snap_subtitles_to_vad_boundaries,
    VadErrorKind,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
subtitle_qa_gate — 影視字幕多模態 QA 門禁統一入口 (Rust 原生版)

用法:
  subtitle_qa_gate cps <srt_file> [--red N] [--orange N] [--json] [--strict] [--warn-only]
      檢驗 SRT 字幕每秒字數 (CPS)、異常 <=0.55s 短幀與時間倒錯
  subtitle_qa_gate check-eng <srt_file> [--allow NAME,...] [--json]
      掃描尚未翻譯的殘留英文詞句（預設放行 Krishna, Balarama, KRISHNA）
  subtitle_qa_gate vad-qa <boundaries.json> <srt_file> [--tol-ms N] [--fix-out <out.srt>] [--json]
      檢驗字幕起訖時間與語音活動偵測邊界誤差是否 <= 100ms，並可選填輸出修正檔
"#);
}

/// 處理 cps 子指令
/// @outputs: 2
fn handle_cps(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ 錯誤: cps 需要 <srt_file>");
        process::exit(1);
    }
    let srt_path = &args[2];
    let mut red_tenths: u32 = 120;
    let mut orange_tenths: u32 = 90;
    let mut as_json = false;
    let mut strict = true;

    let mut i = 3;
    while i < args.len() {
        if args[i] == "--red" && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<f64>() {
                red_tenths = (v * 10.0).round() as u32;
            }
            i += 2;
        } else if args[i] == "--orange" && i + 1 < args.len() {
            if let Ok(v) = args[i + 1].parse::<f64>() {
                orange_tenths = (v * 10.0).round() as u32;
            }
            i += 2;
        } else if args[i] == "--json" {
            as_json = true;
            i += 1;
        } else if args[i] == "--strict" {
            strict = true;
            i += 1;
        } else if args[i] == "--warn-only" {
            strict = false;
            i += 1;
        } else {
            i += 1;
        }
    }

    let content = match fs::read_to_string(srt_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取 SRT '{}': {e}", srt_path);
            process::exit(1);
        }
    };
    let subs = parse_srt_content(&content);
    let Some(rep) = audit_srt_cps(&subs, red_tenths, orange_tenths) else {
        eprintln!("❌ 門檻數值超出合法範圍 [1.0..50.0]");
        process::exit(1);
    };

    if as_json {
        println!(
            r#"{{"file":{:?},"total":{},"red":{},"orange":{},"green":{},"inverted":{},"dummy":{}}}"#,
            srt_path, rep.total, rep.red, rep.orange, rep.green, rep.inverted, rep.dummy_half_sec
        );
    } else {
        println!("=== [SUBTITLE QA GATE: CPS (RUST)] {srt_path} ===");
        println!("總字幕數: {} 句", rep.total);
        println!("🔴 紅色危險區 (CPS > {:.1}): {} 句", red_tenths as f64 / 10.0, rep.red);
        println!("🟡 黃色偏快區 (CPS > {:.1}): {} 句", orange_tenths as f64 / 10.0, rep.orange);
        println!("🟢 綠色舒適區: {} 句", rep.green);
        println!("⚠️ 異常殘缺短幀 (<= 0.55s 且 >= 8 字): {} 處", rep.dummy_half_sec);
        println!("⛔ 時間倒錯/零時長: {} 處", rep.inverted);
    }

    if strict && (rep.red > 0 || rep.inverted > 0 || rep.dummy_half_sec > 0) {
        process::exit(1);
    }
    process::exit(0);
}

/// 處理 check-eng 子指令
/// @outputs: 2
fn handle_check_eng(args: &[String]) {
    if args.len() < 3 {
        eprintln!("❌ 錯誤: check-eng 需要 <srt_file>");
        process::exit(1);
    }
    let srt_path = &args[2];
    let mut allow_owned: Vec<String> = Vec::new();
    let mut as_json = false;

    let mut i = 3;
    while i < args.len() {
        if args[i] == "--allow" && i + 1 < args.len() {
            for item in args[i + 1].split(',') {
                let t = item.trim();
                if !t.is_empty() {
                    allow_owned.push(t.to_string());
                }
            }
            i += 2;
        } else if args[i] == "--json" {
            as_json = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let content = match fs::read_to_string(srt_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取 SRT '{}': {e}", srt_path);
            process::exit(1);
        }
    };
    let subs = parse_srt_content(&content);
    let allow_refs: Vec<&str> = allow_owned.iter().map(|s| s.as_str()).collect();
    let hits = find_residual_english_lines(&subs, &allow_refs);

    if as_json {
        println!(r#"{{"file":{:?},"residual_count":{},"indices":{:?}}}"#, srt_path, hits.len(), hits.iter().map(|h| h.idx).collect::<Vec<_>>());
    } else {
        println!("Checking lines with special / untranslated terms: {srt_path}");
        for h in &hits {
            println!(
                "[{}] {} --> {}: {:?} -> {}",
                h.idx,
                format_srt_timestamp_ms(h.start_ms),
                format_srt_timestamp_ms(h.end_ms),
                h.words,
                h.text.replace('\n', " ")
            );
        }
        println!("殘留未譯英文字行數: {}", hits.len());
    }
    process::exit(if hits.is_empty() { 0 } else { 1 });
}

/// 處理 vad-qa 子指令
/// @outputs: 2
fn handle_vad_qa(args: &[String]) {
    if args.len() < 4 {
        eprintln!("❌ 錯誤: vad-qa 需要 <boundaries.json> <srt_file>");
        process::exit(1);
    }
    let boundaries_path = &args[2];
    let srt_path = &args[3];
    let mut tol_ms: u32 = 100;
    let mut fix_out: Option<&str> = None;
    let mut as_json = false;

    let mut i = 4;
    while i < args.len() {
        if args[i] == "--tol-ms" && i + 1 < args.len() {
            tol_ms = args[i + 1].parse().unwrap_or(100);
            i += 2;
        } else if args[i] == "--fix-out" && i + 1 < args.len() {
            fix_out = Some(&args[i + 1]);
            i += 2;
        } else if args[i] == "--json" {
            as_json = true;
            i += 1;
        } else {
            i += 1;
        }
    }

    let b_json = match fs::read_to_string(boundaries_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取邊界檔案 '{boundaries_path}': {e}");
            process::exit(1);
        }
    };
    let srt_str = match fs::read_to_string(srt_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ 無法讀取 SRT 檔案 '{srt_path}': {e}");
            process::exit(1);
        }
    };

    let boundaries = parse_speech_boundaries_json(&b_json);
    let subs = parse_srt_content(&srt_str);
    let Some(rep) = evaluate_vad_boundaries(&boundaries, &subs, tol_ms) else {
        eprintln!("❌ tol_ms 超出合法範圍 [1..5000]");
        process::exit(1);
    };

    if let Some(out_path) = fix_out {
        if let Some(fixed) = snap_subtitles_to_vad_boundaries(&boundaries, &subs, tol_ms) {
            let mut buf = String::from("\u{FEFF}");
            for s in &fixed {
                buf.push_str(&format!(
                    "{}\n{} --> {}\n{}\n\n",
                    s.idx,
                    format_srt_timestamp_ms(s.start_ms),
                    format_srt_timestamp_ms(s.end_ms),
                    s.text
                ));
            }
            if let Err(e) = fs::write(out_path, buf) {
                eprintln!("❌ 寫出修正 SRT 失敗: {e}");
                process::exit(1);
            }
        }
    }

    if as_json {
        println!(
            r#"{{"turns":{},"subs":{},"start_pass":{},"end_pass":{},"discrepancies":{}}}"#,
            rep.total_turns, rep.total_subs, rep.turn_start_pass, rep.turn_end_pass, rep.discrepancies.len()
        );
    } else {
        println!("=== [SUBTITLE QA GATE: VAD HOOK (RUST)] ===");
        println!("語音句段數: {} | 字幕總數: {} | 容差 <= {}ms", rep.total_turns, rep.total_subs, tol_ms);
        println!("出現時間合格: {} | 消失時間合格: {} | 超標項數: {}", rep.turn_start_pass, rep.turn_end_pass, rep.discrepancies.len());
        for d in rep.discrepancies.iter().take(10) {
            let tag = match d.kind {
                VadErrorKind::StartError => "START_ERROR",
                VadErrorKind::EndError => "END_ERROR",
            };
            println!("  • [{tag}] group={} #{} err={}ms", d.group_id, d.sub_idx, d.error_ms);
        }
    }

    let ok = rep.discrepancies.is_empty() || fix_out.is_some();
    process::exit(if ok { 0 } else { 1 });
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
        "cps" => handle_cps(&args),
        "check-eng" => handle_check_eng(&args),
        "vad-qa" => handle_vad_qa(&args),
        "-h" | "--help" | "help" => print_usage(),
        other => {
            eprintln!("❌ 未知指令: {other}");
            print_usage();
            process::exit(1);
        }
    }
}
