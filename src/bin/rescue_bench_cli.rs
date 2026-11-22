// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/rescue_bench_cli — Pure Rust CLI for HARNESS-RESCUE-BENCH-PLAN v1.0
// - Inputs: Subcommands (`claimed-done`, `validate-outcome`, `arm-leak`, `chance-baseline`, `honest-metric`, `init-campaign`, `validate-outcomes`, `analyze`, `cocoon-quiz`)
// - Outputs: Deterministic single-line JSON to stdout
// - Invariants: Zero external crates, nesting <= 2, JPL <= 60 lines/fn, zero unproved unwrap

use rust_bricks::crypto_vault::sha256_hex;
use rust_bricks::encoding_base64_json::{JsonVal, json_escape_str, parse_json, serialize_json_iterative};
use rust_bricks::rescue_bench_core::{
    rescue_chance_baseline_strict, rescue_check_arm_leak, rescue_classify_claimed_done,
    rescue_format_honest_metric, rescue_validate_outcome,
};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

/// Parses comma-separated 0/1 flags into `Vec<u8>`.
/// @outputs: 2
fn parse_u8_csv(raw: &str) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    for tok in raw.split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        out.push(t.parse::<u8>().ok()?);
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Handles `claimed-done`, `validate-outcome`, and `arm-leak`.
/// @outputs: 2
fn handle_guard_cmd(cmd: &str, args: &[String]) -> Option<String> {
    if cmd == "claimed-done" && args.len() >= 4 {
        let blind = args[3] == "true";
        let claimed = rescue_classify_claimed_done(&args[2], blind);
        return Some(format!("{{\"claimed_done\":{}}}", claimed));
    }
    if cmd == "validate-outcome" && args.len() >= 7 {
        let (ok, code) = rescue_validate_outcome(
            &args[2],
            args[3] == "true",
            args[4] == "true",
            args[5] == "true",
            args[6] == "true",
        );
        return Some(format!("{{\"valid\":{},\"code\":\"{}\"}}", ok, code));
    }
    if cmd == "arm-leak" && args.len() >= 4 {
        let labels: Vec<&str> = args[3].split(',').collect();
        let leaked = rescue_check_arm_leak(&args[2], &labels);
        return Some(format!("{{\"leaked\":{}}}", leaked));
    }
    None
}

/// Handles `chance-baseline` and `honest-metric`.
/// @outputs: 2
fn handle_stat_cmd(cmd: &str, args: &[String]) -> Option<String> {
    if cmd == "chance-baseline" && args.len() >= 9 {
        let a = parse_u8_csv(&args[2])?;
        let b = parse_u8_csv(&args[3])?;
        let c = parse_u8_csv(&args[4])?;
        let ci = parse_u8_csv(&args[5])?;
        let reps = args[6].parse::<usize>().ok()?;
        let n_perm = args[7].parse::<u32>().ok()?;
        let seed = args[8].parse::<u64>().ok()?;
        let (strict, rev, med, p95) = rescue_chance_baseline_strict(&a, &b, &c, &ci, reps, n_perm, seed)?;
        return Some(format!(
            "{{\"strict_rescue\":{},\"reverse_count\":{},\"chance_median\":{:.2},\"chance_p95\":{:.2}}}",
            strict, rev, med, p95
        ));
    }
    if cmd == "honest-metric" && args.len() >= 5 {
        let k = args[2].parse::<u64>().ok()?;
        let n = args[3].parse::<u64>().ok()?;
        let rec = args[4].parse::<f64>().ok()?;
        let text = rescue_format_honest_metric(k, n, rec)?;
        return Some(format!("{{\"formatted\":\"{}\"}}", text));
    }
    None
}

/// Parses `--flag value` from CLI args.
/// @outputs: 2
fn find_flag_val<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    let mut i = 2usize;
    while i + 1 < args.len() {
        if args[i] == flag {
            return Some(args[i + 1].as_str());
        }
        i += 1;
    }
    None
}

/// Handles `init-campaign --out-dir <dir> --campaign-id <id> [--template <path>]`.
/// @outputs: 3
fn handle_init_campaign_cmd(args: &[String]) -> Option<(bool, String)> {
    let out_dir = Path::new(find_flag_val(args, "--out-dir")?);
    let cid = find_flag_val(args, "--campaign-id")?.trim();
    if cid.is_empty() {
        return Some((false, "{\"ok\":false,\"reason\":\"EMPTY_CAMPAIGN_ID\"}".to_string()));
    }
    let _ = fs::create_dir_all(out_dir);
    let prereg_path = out_dir.join("prereg.json");
    if prereg_path.exists() {
        let p_str = json_escape_str(&prereg_path.to_string_lossy());
        return Some((false, format!("{{\"ok\":false,\"reason\":\"PREREG_IMMUTABLE_EXISTS\",\"path\":\"{}\"}}", p_str)));
    }
    let tmpl_raw = match find_flag_val(args, "--template") {
        Some(p) => fs::read_to_string(p).ok()?,
        None => "{\"campaign_id\":\"\",\"cocoon_quiz\":[\"Q1\",\"Q2\"]}".to_string(),
    };
    let JsonVal::Obj(mut pairs) = parse_json(&tmpl_raw).ok()? else {
        return None;
    };
    pairs.retain(|(k, _)| k != "campaign_id");
    pairs.insert(0, ("campaign_id".to_string(), JsonVal::Str(cid.to_string())));
    let out_raw = format!("{}\n", serialize_json_iterative(&JsonVal::Obj(pairs)));
    fs::write(&prereg_path, out_raw.as_bytes()).ok()?;
    let digest: String = sha256_hex(out_raw.as_bytes()).chars().take(16).collect();
    let p_str = json_escape_str(&prereg_path.to_string_lossy());
    Some((true, format!("{{\"ok\":true,\"campaign_id\":\"{}\",\"prereg_sha256_16\":\"{}\",\"path\":\"{}\"}}", json_escape_str(cid), digest, p_str)))
}

/// Validates one parsed outcome row against A11/A12/A13/P20 and arm-leak rules.
/// @outputs: 3
fn check_single_outcome_row(row: &JsonVal, labels: &[&str]) -> (bool, String) {
    let req_bools = ["plugins_null", "plugins_match", "effort_event_unknown", "suite_match"];
    let mut bool_vals = [false; 4];
    for (idx, key) in req_bools.iter().enumerate() {
        let Some(b) = row.get(key).and_then(|v| v.as_bool()) else {
            return (false, "MISSING_REQUIRED_FIELDS".to_string());
        };
        bool_vals[idx] = b;
    }
    let run_id = row.get("run_id").and_then(|v| v.as_json_str()).unwrap_or("").trim();
    let arm = row.get("arm").and_then(|v| v.as_json_str()).unwrap_or("").trim();
    let effort = row.get("effort").and_then(|v| v.as_json_str()).unwrap_or("").trim();
    if run_id.is_empty() || arm.is_empty() || effort.is_empty() {
        return (false, "MISSING_REQUIRED_FIELDS".to_string());
    }
    let by = row.get("by").and_then(|v| v.as_json_str()).unwrap_or("");
    let (valid, code) = rescue_validate_outcome(by, bool_vals[0], bool_vals[1], bool_vals[2], bool_vals[3]);
    let arm_name = row.get("arm_name").and_then(|v| v.as_json_str()).unwrap_or("");
    let notes = row.get("notes").and_then(|v| v.as_json_str()).unwrap_or("");
    let multi_labels: Vec<&str> = labels.iter().copied().filter(|l| l.trim().chars().count() >= 2).collect();
    let leaked = rescue_check_arm_leak(arm, labels)
        || rescue_check_arm_leak(arm_name, labels)
        || rescue_check_arm_leak(by, labels)
        || rescue_check_arm_leak(run_id, labels)
        || rescue_check_arm_leak(effort, labels)
        || (!multi_labels.is_empty() && rescue_check_arm_leak(notes, &multi_labels));
    (valid && !leaked, if leaked { "ARM_LEAK".to_string() } else { code.to_string() })
}

/// Handles `validate-outcomes --outcomes <jsonl> [--forbidden-arms <csv>]`.
/// @outputs: 3
fn handle_validate_outcomes_cmd(args: &[String]) -> Option<(bool, String)> {
    let path = find_flag_val(args, "--outcomes")?;
    let forbidden = find_flag_val(args, "--forbidden-arms").unwrap_or("甲,乙,丙1,丙2,arm_a,arm_b,arm_c1,arm_c2");
    let labels: Vec<&str> = forbidden.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
    if labels.is_empty() {
        return Some((false, "{\"ok\":false,\"reason\":\"EMPTY_FORBIDDEN_ARMS\"}".to_string()));
    }
    let raw = fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = raw.lines().filter(|l| !l.trim().is_empty()).collect();
    if lines.is_empty() {
        return Some((false, "{\"ok\":false,\"reason\":\"EMPTY_OUTCOMES_FILE\",\"total\":0,\"valid\":0,\"invalid\":[]}".to_string()));
    }
    let (mut valid_cnt, mut invalid_cnt) = (0usize, 0usize);
    for ln in &lines {
        let Ok(row) = parse_json(ln) else {
            return Some((false, "{\"ok\":false,\"reason\":\"INVALID_JSONL_ROW\"}".to_string()));
        };
        let (ok, _) = check_single_outcome_row(&row, &labels);
        valid_cnt += usize::from(ok);
        invalid_cnt += usize::from(!ok);
    }
    let ok = invalid_cnt == 0;
    Some((ok, format!("{{\"ok\":{},\"total\":{},\"valid\":{},\"invalid_count\":{}}}", ok, lines.len(), valid_cnt, invalid_cnt)))
}

/// Parses one strict binary flag (`bool` or `0.0`/`1.0`) without coercing floats or negatives.
/// @outputs: 3
fn parse_strict_u8_flag(v: &JsonVal) -> Option<u8> {
    match v {
        JsonVal::Bool(true) => Some(1),
        JsonVal::Bool(false) => Some(0),
        JsonVal::Num(n) if *n == 0.0 => Some(0),
        JsonVal::Num(n) if *n == 1.0 => Some(1),
        _ => None,
    }
}

/// Extracts a strict binary `Vec<u8>` flag array from a `JsonVal` (nesting <= 2).
/// @outputs: 2
fn extract_u8_arr(doc: &JsonVal, key: &str) -> Option<Vec<u8>> {
    let arr = doc.get(key)?.as_arr()?;
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        out.push(parse_strict_u8_flag(v)?);
    }
    Some(out)
}

/// Handles `analyze --matrix <json> [--n-perm 500] [--seed 42]`.
/// @outputs: 3
fn handle_analyze_cmd(args: &[String]) -> Option<(bool, String)> {
    let mpath = find_flag_val(args, "--matrix")?;
    let n_perm = find_flag_val(args, "--n-perm").and_then(|s| s.parse::<u32>().ok()).unwrap_or(500);
    let seed = find_flag_val(args, "--seed").and_then(|s| s.parse::<u64>().ok()).unwrap_or(42);
    let doc = parse_json(&fs::read_to_string(mpath).ok()?).ok()?;
    let reps_f = doc.get("n_reps").and_then(|v| v.as_f64()).unwrap_or(-1.0);
    if !reps_f.is_finite() || reps_f.fract() != 0.0 || !(1.0..=20.0).contains(&reps_f) {
        return Some((false, "{\"ok\":false,\"reason\":\"INVALID_N_REPS\"}".to_string()));
    }
    let (Some(a), Some(b), Some(c), Some(ci)) = (
        extract_u8_arr(&doc, "a"), extract_u8_arr(&doc, "b"),
        extract_u8_arr(&doc, "c"), extract_u8_arr(&doc, "c_intervened"),
    ) else {
        return Some((false, "{\"ok\":false,\"reason\":\"INVALID_BINARY_FLAG_ARRAY\"}".to_string()));
    };
    let rec_lo = doc.get("recall_lower").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let (strict, rev, med, p95) = rescue_chance_baseline_strict(&a, &b, &c, &ci, reps_f as usize, n_perm, seed)?;
    let c_pass = c.iter().filter(|&&x| x != 0).count() as u64;
    let fmt = rescue_format_honest_metric(c_pass, c.len() as u64, rec_lo)?;
    Some((true, format!(
        "{{\"ok\":true,\"baseline\":{{\"strict_rescue\":{},\"reverse_count\":{},\"chance_median\":{:.2},\"chance_p95\":{:.2}}},\"honest_c_rate\":\"{}\"}}",
        strict, rev, med, p95, json_escape_str(&fmt)
    )))
}

/// Handles `cocoon-quiz --report-path <path>`.
/// @outputs: 2
fn handle_cocoon_quiz_cmd(args: &[String]) -> Option<(bool, String)> {
    let rpath = find_flag_val(args, "--report-path")?;
    let p = Path::new(rpath);
    let non_empty = fs::metadata(p).map(|m| m.len() > 0).unwrap_or(false);
    let ok = p.is_file() && non_empty;
    let p_str = json_escape_str(rpath);
    Some((ok, format!("{{\"ok\":{},\"report\":\"{}\"}}", ok, p_str)))
}

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rescue_bench_cli <claimed-done|validate-outcome|arm-leak|chance-baseline|honest-metric|init-campaign|validate-outcomes|analyze|cocoon-quiz> ...");
        return ExitCode::FAILURE;
    }
    let cmd = args[1].as_str();
    let routed = match cmd {
        "init-campaign" => handle_init_campaign_cmd(&args),
        "validate-outcomes" => handle_validate_outcomes_cmd(&args),
        "analyze" => handle_analyze_cmd(&args),
        "cocoon-quiz" => handle_cocoon_quiz_cmd(&args),
        _ => handle_guard_cmd(cmd, &args).or_else(|| handle_stat_cmd(cmd, &args)).map(|s| (true, s)),
    };
    match routed {
        Some((ok, json)) => {
            println!("{}", json);
            if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        None => ExitCode::FAILURE,
    }
}
