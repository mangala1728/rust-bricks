// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/fpy_recorder — Pure Rust FPY Recorder CLI, Phase 0 benchmark & K1-K9 reporter (FPY-RECORDER-SPEC v1.0)
// - Inputs: Subcommands (`bench-phase0`, `record-tool`, `record-prompt`, `record-turn-end`, `hook-event`, `verify-chain`, `report`, `check-proposal`)
// - Outputs: RFC 8259 escaped fpy/1 JSONL lines, SHA-256/I-O benchmark JSON, K3+K4 audit, 【描述】 Markdown report
// - Invariants: Zero external crates, fail-open P-3 on record-*/hook-event, atomic lockfile guard, path-traversal safe, nesting <= 2, JPL <= 60 lines/fn

use rust_bricks::encoding_base64_json::{JsonVal, parse_json};
use rust_bricks::fpy_recorder_core::{
    fpy_clip_utf16, fpy_detect_n1_and_n1b, fpy_detect_n4_prompt, fpy_detect_n6_turn_end,
    fpy_evaluate_proposal_transition, fpy_hash16, fpy_normalize_tool_class, fpy_redact_secrets,
    fpy_verify_hash_chain,
};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const MAX_ROTATE_BYTES: u64 = 2 * 1024 * 1024;
const OVERSIZE_BYTES: usize = 65_536;
const MAX_HOOK_STDIN_BYTES: usize = 1_048_576;
const MAX_LOCK_RETRIES: u8 = 25;
const STALE_LOCK_SECS: u64 = 5;

/// Escapes RFC 8259 JSON string characters cleanly (quotes, backslash, newlines, tab, control chars).
/// @outputs: 1
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            _ => out.push(ch),
        }
    }
    out
}

/// Sanitizes session ID to `[A-Za-z0-9_-]` and appends a hash suffix when truncated (`> 55` chars).
/// @outputs: 2
fn sanitize_session_id(raw: &str) -> String {
    let clean: String = raw
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(55)
        .collect();
    let base = if clean.is_empty() { "default-session" } else { clean.as_str() };
    if raw.chars().count() > 55 {
        let tag: String = fpy_hash16(raw).chars().take(8).collect();
        return format!("{}_{}", base, tag);
    }
    base.to_string()
}

/// Resolves FPY storage root honoring `FPY_DATA_DIR` (P4).
/// @outputs: 1
fn resolve_data_dir() -> PathBuf {
    if let Ok(v) = env::var("FPY_DATA_DIR") {
        if !v.trim().is_empty() {
            return PathBuf::from(v.trim());
        }
    }
    env::temp_dir().join("fpy-recorder")
}

/// Runs Phase 0 benchmark (P5 SHA-256 <= 0.2ms, P1 2MiB read-append-write).
/// @outputs: 1
fn run_bench_phase0() -> ExitCode {
    let s120 = "a".repeat(120);
    let s4kb = "b".repeat(4096);
    let t0 = Instant::now();
    for _ in 0..1000 {
        std::hint::black_box(fpy_hash16(&s120));
    }
    let sha120_ms = t0.elapsed().as_secs_f64() * 1000.0 / 1000.0;
    let t1 = Instant::now();
    for _ in 0..1000 {
        std::hint::black_box(fpy_hash16(&s4kb));
    }
    let sha4kb_ms = t1.elapsed().as_secs_f64() * 1000.0 / 1000.0;

    let tmp_file = env::temp_dir().join("fpy_phase0_2mib_bench.jsonl");
    let seed_data = format!("{}\n", "x".repeat(255)).repeat(8192);
    let _ = fs::write(&tmp_file, &seed_data);
    let mut io_times = Vec::with_capacity(20);
    for _ in 0..20 {
        let ts = Instant::now();
        let mut buf = fs::read_to_string(&tmp_file).unwrap_or_default();
        buf.push_str("{\"v\":\"fpy/1\",\"seq\":1}\n");
        let _ = fs::write(&tmp_file, &buf);
        io_times.push(ts.elapsed().as_secs_f64() * 1000.0);
    }
    let _ = fs::remove_file(&tmp_file);
    io_times.sort_by(|a, b| a.total_cmp(b));
    let io_p95 = io_times.get(18).copied().unwrap_or(0.0);
    println!(
        "{{\"sha_120b_avg_ms\":{:.5},\"sha_4kb_avg_ms\":{:.5},\"io_2mib_p95_ms\":{:.3},\"p5_pass\":{},\"p1_amortized_20_ms\":{:.3}}}",
        sha120_ms, sha4kb_ms, io_p95, sha4kb_ms <= 0.2, io_p95 / 20.0
    );
    ExitCode::SUCCESS
}

/// Finds or rotates session JSONL path (2 MiB limit) using sanitized session ID.
/// @outputs: 1
fn resolve_session_file(base_dir: &Path, safe_session: &str) -> PathBuf {
    let mut seq = 1u32;
    while seq < 999 {
        let candidate = base_dir.join(format!("{}-{}.jsonl", safe_session, seq));
        let len = fs::metadata(&candidate).map(|m| m.len()).unwrap_or(0);
        if len < MAX_ROTATE_BYTES {
            return candidate;
        }
        seq += 1;
    }
    base_dir.join(format!("{}-999.jsonl", safe_session))
}

/// Returns `true` if lockfile modification age exceeds `STALE_LOCK_SECS` (or clock skewed > 60s).
/// @outputs: 2
fn is_lock_stale(lock_path: &Path) -> bool {
    let Ok(meta) = fs::metadata(lock_path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    match modified.elapsed() {
        Ok(d) => d.as_secs() >= STALE_LOCK_SECS,
        Err(e) => e.duration().as_secs() >= 60,
    }
}

/// Acquires an exclusive cross-platform lockfile with bounded retries (`<= 50ms`).
/// @outputs: 2
fn acquire_session_lock(base_dir: &Path, safe_session: &str) -> Option<PathBuf> {
    let lock_path = base_dir.join(format!(".{}.lock", safe_session));
    for attempt in 0..MAX_LOCK_RETRIES {
        if OpenOptions::new().write(true).create_new(true).open(&lock_path).is_ok() {
            return Some(lock_path);
        }
        if is_lock_stale(&lock_path) {
            let _ = fs::remove_file(&lock_path);
        }
        if attempt + 1 < MAX_LOCK_RETRIES {
            thread::sleep(Duration::from_millis(2));
        }
    }
    None
}

/// Appends one `fpy/1` line cleanly under atomic lockfile guard, returning `(seq, prev)` first.
/// @outputs: 2
fn append_session_line(safe_session: &str, build_line: impl FnOnce(usize, &str) -> String) {
    let dir = resolve_data_dir();
    let _ = fs::create_dir_all(&dir);
    let Some(lock_path) = acquire_session_lock(&dir, safe_session) else {
        return;
    };
    let file = resolve_session_file(&dir, safe_session);
    let raw_bytes = fs::read(&file).unwrap_or_default();
    let existing = String::from_utf8_lossy(&raw_bytes);
    let lines: Vec<&str> = existing.lines().filter(|l| !l.trim().is_empty()).collect();
    let seq = lines.len() + 1;
    let prev = lines.last().map(|l| fpy_hash16(l)).unwrap_or_else(|| "0".to_string());
    let line = build_line(seq, &prev);
    let wrote = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)
        .and_then(|mut fh| writeln!(fh, "{}", line))
        .is_ok();
    let _ = fs::remove_file(&lock_path);
    if wrote {
        println!("{}", line);
    }
}

/// Shared tool event recorder used by `record-tool` and `hook-event post-tool`.
/// @outputs: 1
fn record_tool_fields(safe_sess: &str, host: &str, tool: &str, raw_cmd: &str, exit_code: Option<i32>) {
    let tool_class = fpy_normalize_tool_class(tool);
    let oversize = raw_cmd.len() > OVERSIZE_BYTES;
    let slice = if oversize { &raw_cmd[..raw_cmd.floor_char_boundary(4096)] } else { raw_cmd };
    let (redacted, kinds) = fpy_redact_secrets(slice);
    let (summary, cut) = fpy_clip_utf16(&redacted, 120);
    let (n1, n1b, sw) = if tool_class == "shell" {
        fpy_detect_n1_and_n1b(slice, exit_code, exit_code.map(|c| c != 0).unwrap_or(false), false)
    } else {
        (false, false, false)
    };
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let nft = if n1 { "[\"N1\"]" } else { "[]" };
    append_session_line(safe_sess, |seq, prev| {
        format!(
            "{{\"v\":\"fpy/1\",\"id\":\"{}-{}\",\"seq\":{},\"mono_ms\":{},\"host\":\"{}\",\"session\":\"{}\",\"agent\":\"main\",\"kind\":\"tool\",\"prev\":\"{}\",\"tool\":\"{}\",\"tool_class\":\"{}\",\"oversize\":{},\"shell_write\":{},\"summary\":\"{}\",\"redacted_count\":{},\"cut\":{},\"exit\":{},\"masked\":{},\"nft\":{}}}",
            json_escape(safe_sess), seq, seq, now_ms, json_escape(host), json_escape(safe_sess), prev, json_escape(tool),
            tool_class, oversize, sw, json_escape(&summary), kinds.len(), cut,
            exit_code.map(|c| c.to_string()).unwrap_or_else(|| "null".to_string()), n1b, nft
        )
    });
}

/// Appends one `fpy/1` tool record with redaction, UTF-16 clip, shell-only N1/N1b, and prev chain (P-3).
/// @outputs: 2
fn run_record_tool(args: &[String]) -> ExitCode {
    if args.len() < 6 {
        return ExitCode::SUCCESS;
    }
    let safe_sess = sanitize_session_id(&args[2]);
    let exit_code = args[5].parse::<i32>().ok();
    record_tool_fields(&safe_sess, "rust", &args[3], &args[4], exit_code);
    ExitCode::SUCCESS
}

/// Appends one `fpy/1` prompt record (N4 detection).
/// @outputs: 2
fn run_record_prompt(args: &[String]) -> ExitCode {
    if args.len() < 5 {
        return ExitCode::SUCCESS;
    }
    let safe_sess = sanitize_session_id(&args[2]);
    let source = &args[3];
    let (tag, hits, has_q) = fpy_detect_n4_prompt(&args[4], source, false, false, false);
    let hits_json: Vec<String> = hits.iter().map(|h| format!("\"{}\"", h)).collect();
    append_session_line(&safe_sess, |seq, prev| {
        format!(
            "{{\"v\":\"fpy/1\",\"id\":\"{}-{}\",\"seq\":{},\"host\":\"rust\",\"session\":\"{}\",\"kind\":\"prompt\",\"source\":\"{}\",\"prev\":\"{}\",\"n4_tag\":\"{}\",\"n4_hits\":[{}],\"n4_q\":{}}}",
            json_escape(&safe_sess), seq, seq, json_escape(&safe_sess), json_escape(source), prev, tag, hits_json.join(","), has_q
        )
    });
    ExitCode::SUCCESS
}

/// Appends one `fpy/1` turn_end record (N6 detection).
/// @outputs: 2
fn run_record_turn_end(args: &[String]) -> ExitCode {
    if args.len() < 4 {
        return ExitCode::SUCCESS;
    }
    let safe_sess = sanitize_session_id(&args[2]);
    let (done, hedge) = fpy_detect_n6_turn_end(&args[3]);
    let done_j: Vec<String> = done.iter().map(|h| format!("\"{}\"", h)).collect();
    let hedge_j: Vec<String> = hedge.iter().map(|h| format!("\"{}\"", h)).collect();
    append_session_line(&safe_sess, |seq, prev| {
        format!(
            "{{\"v\":\"fpy/1\",\"id\":\"{}-{}\",\"seq\":{},\"host\":\"rust\",\"session\":\"{}\",\"kind\":\"turn_end\",\"prev\":\"{}\",\"done_hits\":[{}],\"hedge_hits\":[{}]}}",
            json_escape(&safe_sess), seq, seq, json_escape(&safe_sess), prev, done_j.join(","), hedge_j.join(",")
        )
    });
    ExitCode::SUCCESS
}

/// Verifies `seq` continuity (K3) and `prev` hash-chain (K4) of a JSONL file.
/// @outputs: 2
fn run_verify_chain(path_str: &str) -> ExitCode {
    let raw = match fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(_) => return ExitCode::FAILURE,
    };
    let lines: Vec<&str> = raw.lines().collect();
    let (total, broken) = fpy_verify_hash_chain(&lines);
    println!("{{\"total\":{},\"broken\":{},\"ok\":{}}}", total, broken, total > 0 && broken == 0);
    if total > 0 && broken == 0 { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

/// Evaluates a proposal transition via P3/P22/Q5 Bayesian rule.
/// @outputs: 2
fn run_check_proposal(args: &[String]) -> ExitCode {
    if args.len() < 10 {
        return ExitCode::FAILURE;
    }
    let state = &args[2];
    let s = args[3].parse::<u32>().unwrap_or(u32::MAX);
    let f = args[4].parse::<u32>().unwrap_or(u32::MAX);
    let tasks = args[5].parse::<u32>().unwrap_or(u32::MAX);
    let sess = args[6].parse::<u32>().unwrap_or(u32::MAX);
    let days = args[7].parse::<u32>().unwrap_or(u32::MAX);
    let pend = args[8].parse::<u32>().unwrap_or(u32::MAX);
    let relax = args[9] == "true";
    let (next_state, promoted) = fpy_evaluate_proposal_transition(state, s, f, tasks, sess, days, pend, relax);
    println!("{{\"next_state\":\"{}\",\"promoted\":{}}}", next_state, promoted);
    if next_state == "invalid" || next_state == "rejected" { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}

/// Generates a `【描述】`-level FPY Markdown report from a session JSONL file.
/// @outputs: 2
fn run_report(path_str: &str) -> ExitCode {
    let raw = match fs::read_to_string(path_str) {
        Ok(s) => s,
        Err(_) => return ExitCode::FAILURE,
    };
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    let (total, broken) = fpy_verify_hash_chain(&lines);
    let n1_cnt = lines.iter().filter(|l| l.contains("\"nft\":[\"N1\"]")).count();
    let n1b_cnt = lines.iter().filter(|l| l.contains("\"masked\":true")).count();
    let sw_cnt = lines.iter().filter(|l| l.contains("\"shell_write\":true")).count();
    let first_pass = total.saturating_sub(n1_cnt);
    let sample_note = if total < 30 { "樣本不足 (<30)" } else { "達標" };
    println!("# FPY 良率報表（【描述】等級）");
    println!("- 樣本數：{}（{}）、一次做對：{}、N1：{}、K4/K3 鏈異常：{}、K8 N1b：{}、K9 shell_write：{}", total, sample_note, first_pass, n1_cnt, broken, n1b_cnt, sw_cnt);
    ExitCode::SUCCESS
}

/// Reads bounded UTF-8 stdin payload (`<= MAX_HOOK_STDIN_BYTES`) for `hook-event`.
/// @outputs: 2
fn read_bounded_stdin() -> Option<String> {
    let mut buf = Vec::new();
    let mut handle = std::io::stdin().take(MAX_HOOK_STDIN_BYTES as u64);
    handle.read_to_end(&mut buf).ok()?;
    let text = String::from_utf8_lossy(&buf).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Extracts sanitized session ID from Antigravity/Claude hook JSON payload.
/// @outputs: 1
fn extract_hook_session(doc: &JsonVal) -> String {
    let raw = doc
        .get("trajectoryId")
        .or_else(|| doc.get("session_id"))
        .and_then(|v| v.as_json_str())
        .unwrap_or("default-session");
    sanitize_session_id(raw)
}

/// Extracts string field from `doc.args.<k1>`, `doc.tool_input.<k2>`, or `doc.toolInput.<k2>`.
/// @outputs: 2
fn pick_arg_str<'a>(doc: &'a JsonVal, k_args: &str, k_ti: &str) -> Option<&'a str> {
    doc.get("args")
        .and_then(|a| a.get(k_args))
        .and_then(|v| v.as_json_str())
        .or_else(|| doc.get("tool_input").and_then(|t| t.get(k_ti)).and_then(|v| v.as_json_str()))
        .or_else(|| doc.get("toolInput").and_then(|t| t.get(k_ti)).and_then(|v| v.as_json_str()))
}

/// Extracts non-truncating `i32` exit code (non-zero 64-bit values never truncate to 0).
/// @outputs: 2
fn extract_hook_exit_code(doc: &JsonVal) -> Option<i32> {
    let v = doc.get("exit_code").or_else(|| doc.get("exitCode"))?;
    let n = v.as_i64()?;
    if n == 0 {
        return Some(0);
    }
    Some(i32::try_from(n).unwrap_or(1))
}

/// Handles Antigravity `hook-event post-tool|prompt|stop` from stdin JSON (strict P-3 Fail-Open).
/// @outputs: 1
fn run_hook_event(args: &[String]) -> ExitCode {
    let kind = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let Some(raw) = read_bounded_stdin() else {
        return ExitCode::SUCCESS;
    };
    let Ok(doc) = parse_json(&raw) else {
        return ExitCode::SUCCESS;
    };
    let safe_sess = extract_hook_session(&doc);
    if kind == "post-tool" {
        let tool = doc.get("toolName").or_else(|| doc.get("tool_name")).and_then(|v| v.as_json_str()).unwrap_or("unknown");
        let cmd = pick_arg_str(&doc, "CommandLine", "command")
            .or_else(|| pick_arg_str(&doc, "Path", "file_path"))
            .or_else(|| pick_arg_str(&doc, "TargetFile", "TargetFile"))
            .or_else(|| pick_arg_str(&doc, "AbsolutePath", "AbsolutePath"))
            .unwrap_or("");
        record_tool_fields(&safe_sess, "antigravity", tool, cmd, extract_hook_exit_code(&doc));
    } else if kind == "prompt" {
        let p_txt = doc.get("prompt").or_else(|| doc.get("user_prompt")).and_then(|v| v.as_json_str())
            .or_else(|| pick_arg_str(&doc, "prompt", "prompt")).unwrap_or("");
        let _ = run_record_prompt(&["fpy".into(), "record-prompt".into(), safe_sess, "user".into(), p_txt.into()]);
    } else if kind == "stop" {
        let msg = doc.get("message").or_else(|| doc.get("last_assistant_message")).and_then(|v| v.as_json_str()).unwrap_or("");
        let _ = run_record_turn_end(&["fpy".into(), "record-turn-end".into(), safe_sess, msg.into()]);
    }
    ExitCode::SUCCESS
}

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: fpy_recorder <bench-phase0|record-tool|record-prompt|record-turn-end|hook-event|verify-chain|report|check-proposal>");
        return ExitCode::FAILURE;
    }
    match args[1].as_str() {
        "bench-phase0" => run_bench_phase0(),
        "record-tool" => run_record_tool(&args),
        "record-prompt" => run_record_prompt(&args),
        "record-turn-end" => run_record_turn_end(&args),
        "hook-event" => run_hook_event(&args),
        "verify-chain" if args.len() >= 3 => run_verify_chain(&args[2]),
        "report" if args.len() >= 3 => run_report(&args[2]),
        "check-proposal" => run_check_proposal(&args),
        _ => ExitCode::FAILURE,
    }
}
