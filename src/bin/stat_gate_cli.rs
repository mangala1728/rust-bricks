// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/stat_gate_cli — Pure Rust CLI wrapper for STAT-GATE-SPEC v1.0 Q8
// - Inputs: Subcommand + numeric/text CLI arguments
// - Outputs: Deterministic single-line JSON to stdout; exit 0 on success, 1 on invalid input
// - Invariants: Zero external crates, zero recursion, nesting <= 2, JPL <= 60 lines/fn, zero unproved unwrap

use rust_bricks::crypto_vault::sha256_hex;
use rust_bricks::encoding_base64_json::{JsonVal, parse_json, serialize_json_iterative};
use rust_bricks::stat_exam_ledger::{
    stat_claim_ledger_register, stat_exam_check_item, stat_exam_runs_consistent,
    stat_exam_validate_bank_counts, stat_receipt_canonical_hash16,
};
use rust_bricks::stat_gate_core::{
    stat_bayesian_ppv, stat_bayesian_trial_upgrade, stat_check_wording_level,
    stat_detect_t4_trigger, stat_holm_bonferroni_reject, stat_median_and_mad,
    stat_p_chart_limits, stat_paired_bootstrap_ci, stat_required_paired_sample_size,
    stat_robust_z_score, stat_rule_of_three_upper_bound, stat_stratified_permutation_p,
    stat_wilson_interval_95,
};
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const SCRIPT_VER: &str = "stat_gate_rs_v1.0";

/// Parses comma-separated finite f64 list.
/// @outputs: 2
fn parse_f64_csv(raw: &str) -> Option<Vec<f64>> {
    let mut out = Vec::new();
    for tok in raw.split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let v = t.parse::<f64>().ok()?;
        out.push(v);
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Parses comma-separated u16 list.
/// @outputs: 2
fn parse_u16_csv(raw: &str) -> Option<Vec<u16>> {
    let mut out = Vec::new();
    for tok in raw.split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let v = t.parse::<u16>().ok()?;
        out.push(v);
    }
    if out.is_empty() { None } else { Some(out) }
}

/// Escapes RFC 8259 JSON string characters cleanly (including tab and ASCII control chars).
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

/// Handles wilson, rule-of-three, bayesian-upgrade, bayesian-ppv, power, pchart.
/// @outputs: 2
fn handle_scalar_cmd(cmd: &str, args: &[String]) -> Option<String> {
    if cmd == "wilson" && args.len() >= 4 {
        let k = args[2].parse::<u64>().ok()?;
        let n = args[3].parse::<u64>().ok()?;
        let (lo, hi) = stat_wilson_interval_95(k, n)?;
        return Some(format!("{{\"k\":{},\"n\":{},\"lo\":{:.8},\"hi\":{:.8}}}", k, n, lo, hi));
    }
    if cmd == "rule-of-three" && args.len() >= 4 {
        let n = args[2].parse::<u64>().ok()?;
        let c = args[3].parse::<f64>().ok()?;
        let ub = stat_rule_of_three_upper_bound(n, c)?;
        return Some(format!("{{\"n\":{},\"recall_lower\":{:.6},\"upper_bound\":{:.8}}}", n, c, ub));
    }
    if cmd == "bayesian-upgrade" && args.len() >= 6 {
        let s = args[2].parse::<u32>().ok()?;
        let f = args[3].parse::<u32>().ok()?;
        let p0 = args[4].parse::<f64>().ok()?;
        let conf = args[5].parse::<f64>().ok()?;
        let (pass, u, j, b22) = stat_bayesian_trial_upgrade(s, f, p0, conf)?;
        return Some(format!(
            "{{\"pass\":{},\"p_uniform\":{:.8},\"p_jeffreys\":{:.8},\"p_beta22\":{:.8}}}",
            pass, u, j, b22
        ));
    }
    if cmd == "bayesian-ppv" && args.len() >= 5 {
        let br = args[2].parse::<f64>().ok()?;
        let rec = args[3].parse::<f64>().ok()?;
        let fpr = args[4].parse::<f64>().ok()?;
        let ppv = stat_bayesian_ppv(br, rec, fpr)?;
        return Some(format!("{{\"ppv\":{:.8}}}", ppv));
    }
    None
}

/// Handles power, pchart, and mad subcommands.
/// @outputs: 2
fn handle_spc_cmd(cmd: &str, args: &[String]) -> Option<String> {
    if cmd == "power" && args.len() >= 6 {
        let sd = args[2].parse::<f64>().ok()?;
        let delta = args[3].parse::<f64>().ok()?;
        let za = args[4].parse::<f64>().ok()?;
        let zb = args[5].parse::<f64>().ok()?;
        let n = stat_required_paired_sample_size(sd, delta, za, zb)?;
        return Some(format!("{{\"required_n\":{}}}", n));
    }
    if cmd == "pchart" && args.len() >= 5 {
        let pbar = args[2].parse::<f64>().ok()?;
        let ni = args[3].parse::<u64>().ok()?;
        let ksig = args[4].parse::<f64>().ok()?;
        let (lcl, ucl) = stat_p_chart_limits(pbar, ni, ksig)?;
        return Some(format!("{{\"lcl\":{:.8},\"ucl\":{:.8}}}", lcl, ucl));
    }
    if cmd == "mad" && args.len() >= 3 {
        let vals = parse_f64_csv(&args[2])?;
        let (med, mad) = stat_median_and_mad(&vals)?;
        let mut z_json = String::from("null");
        if args.len() >= 4 {
            let x = args[3].parse::<f64>().ok()?;
            z_json = stat_robust_z_score(x, med, mad).map(|z| format!("{:.8}", z)).unwrap_or_else(|| "null".to_string());
        }
        return Some(format!("{{\"median\":{:.8},\"mad\":{:.8},\"robust_z\":{}}}", med, mad, z_json));
    }
    None
}

/// Handles holm, perm, bootstrap, wording, and t4 subcommands.
/// @outputs: 2
fn handle_vector_cmd(cmd: &str, args: &[String]) -> Option<String> {
    if cmd == "holm" && args.len() >= 4 {
        let pvals = parse_f64_csv(&args[2])?;
        let alpha = args[3].parse::<f64>().ok()?;
        let rej = stat_holm_bonferroni_reject(&pvals, alpha)?;
        let items: Vec<&str> = rej.iter().map(|&b| if b { "true" } else { "false" }).collect();
        return Some(format!("{{\"rejected\":[{}]}}", items.join(",")));
    }
    if cmd == "perm" && args.len() >= 6 {
        let diffs = parse_f64_csv(&args[2])?;
        let strata = parse_u16_csv(&args[3])?;
        let n_perm = args[4].parse::<u32>().ok()?;
        let seed = args[5].parse::<u64>().ok()?;
        let p = stat_stratified_permutation_p(&diffs, &strata, n_perm, seed)?;
        return Some(format!("{{\"p_value\":{:.8},\"seed\":{}}}", p, seed));
    }
    if cmd == "bootstrap" && args.len() >= 6 {
        let diffs = parse_f64_csv(&args[2])?;
        let alpha = args[3].parse::<f64>().ok()?;
        let n_boot = args[4].parse::<u32>().ok()?;
        let seed = args[5].parse::<u64>().ok()?;
        let (mean, lo, hi) = stat_paired_bootstrap_ci(&diffs, alpha, n_boot, seed)?;
        return Some(format!("{{\"mean\":{:.8},\"lo\":{:.8},\"hi\":{:.8},\"seed\":{}}}", mean, lo, hi, seed));
    }
    if cmd == "wording" && args.len() >= 4 {
        let level = args[2].parse::<u8>().ok()?;
        let hits = stat_check_wording_level(level, &args[3]);
        let quoted: Vec<String> = hits.iter().map(|h| format!("\"{}\"", json_escape(h))).collect();
        return Some(format!("{{\"ok\":{},\"hits\":[{}]}}", hits.is_empty(), quoted.join(",")));
    }
    if cmd == "t4" && args.len() >= 3 {
        let triggered = stat_detect_t4_trigger(&args[2]);
        return Some(format!("{{\"triggered\":{}}}", triggered));
    }
    None
}

/// Dispatches core statistical math subcommands.
/// @outputs: 2
fn dispatch_inner_math(cmd: &str, args: &[String]) -> Option<String> {
    handle_scalar_cmd(cmd, args)
        .or_else(|| handle_spc_cmd(cmd, args))
        .or_else(|| handle_vector_cmd(cmd, args))
}

/// Computes 16-char SHA-256 of the running executable binary.
/// @outputs: 3
fn current_bin_sha16() -> String {
    let Ok(exe) = env::current_exe() else {
        return String::new();
    };
    let Ok(bytes) = fs::read(exe) else {
        return String::new();
    };
    sha256_hex(&bytes).chars().take(16).collect()
}

/// Handles `run <subcmd> [args...]` with L-4 recomputation receipt metadata.
/// @outputs: 2
fn handle_run_cmd(args: &[String]) -> Option<(bool, String)> {
    if args.len() < 3 {
        return None;
    }
    let cli_args: Vec<String> = args[2..].to_vec();
    let mut inner_args = vec![args[0].clone()];
    inner_args.extend(cli_args.iter().cloned());
    let inner_raw = dispatch_inner_math(args[2].as_str(), &inner_args)?;
    let JsonVal::Obj(mut pairs) = parse_json(&inner_raw).ok()? else {
        return None;
    };
    let bin_sha = current_bin_sha16();
    let out_hash = stat_receipt_canonical_hash16(SCRIPT_VER, &bin_sha, &cli_args, &pairs);
    let j_args = JsonVal::Arr(cli_args.iter().map(|s| JsonVal::Str(s.clone())).collect());
    pairs.push(("cli_args".to_string(), j_args));
    pairs.push(("bin_sha256_16".to_string(), JsonVal::Str(bin_sha)));
    pairs.push(("output_sha256_16".to_string(), JsonVal::Str(out_hash)));
    Some((true, serialize_json_iterative(&JsonVal::Obj(pairs))))
}

/// Handles `recompute <receipt.json>` L-4 non-submitter verification.
/// @outputs: 4
fn handle_recompute_cmd(args: &[String]) -> Option<(bool, String)> {
    if args.len() < 3 {
        return None;
    }
    let text = fs::read_to_string(&args[2]).ok()?;
    let doc = parse_json(&text).ok()?;
    let arr = doc.get("cli_args")?.as_arr()?;
    let expected = doc.get("output_sha256_16")?.as_json_str()?.to_string();
    let cli_args: Vec<String> = arr.iter().filter_map(|v| v.as_json_str().map(str::to_string)).collect();
    if cli_args.is_empty() || expected.is_empty() {
        return Some((false, "{\"ok\":false,\"reason\":\"missing cli_args or output_sha256_16\"}".to_string()));
    }
    let bin_sha = current_bin_sha16();
    if let Some(rec_bin) = doc.get("bin_sha256_16").and_then(|v| v.as_json_str()) {
        if !rec_bin.is_empty() && rec_bin != bin_sha {
            return Some((false, "{\"ok\":false,\"reason\":\"BIN_HASH_MISMATCH\"}".to_string()));
        }
    }
    let mut inner_args = vec![args[0].clone()];
    inner_args.extend(cli_args.iter().cloned());
    let inner_raw = dispatch_inner_math(cli_args[0].as_str(), &inner_args)?;
    let JsonVal::Obj(pairs) = parse_json(&inner_raw).ok()? else {
        return None;
    };
    let actual = stat_receipt_canonical_hash16(SCRIPT_VER, &bin_sha, &cli_args, &pairs);
    let ok = actual == expected;
    Some((ok, format!("{{\"ok\":{},\"expected\":\"{}\",\"actual\":\"{}\"}}", ok, expected, actual)))
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

/// Loads existing C13 ledger strictly; rejects non-object or non-string entries.
/// @outputs: 2
fn load_ledger_strict(ledger_path: &str) -> Result<Vec<(String, String)>, &'static str> {
    if !Path::new(ledger_path).exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(ledger_path).map_err(|_| "LEDGER_READ_FAILED")?;
    let JsonVal::Obj(pairs) = parse_json(&raw).map_err(|_| "INVALID_LEDGER_FORMAT")? else {
        return Err("INVALID_LEDGER_FORMAT");
    };
    let mut entries = Vec::with_capacity(pairs.len());
    for (k, v) in pairs {
        let Some(s) = v.as_json_str() else {
            return Err("INVALID_LEDGER_VALUE_TYPE");
        };
        entries.push((k, s.to_string()));
    }
    Ok(entries)
}

/// Ensures parent directory exists for a file path.
/// @outputs: 2
fn ensure_parent_dir(path: &str) -> bool {
    let Some(parent) = Path::new(path).parent() else {
        return true;
    };
    parent.as_os_str().is_empty() || fs::create_dir_all(parent).is_ok()
}

/// Handles `claim-register` bidirectional C13 claim_id registry check.
/// @outputs: 4
fn handle_claim_register_cmd(args: &[String]) -> Option<(bool, String)> {
    let metric = find_flag_val(args, "--metric")?;
    let arm_pair = find_flag_val(args, "--arm-pair")?;
    let unit = find_flag_val(args, "--unit")?;
    let cid = find_flag_val(args, "--claim-id")?;
    let ledger_path = find_flag_val(args, "--ledger").unwrap_or(".ai-memory/stat-claims-ledger.json");
    let mut entries = match load_ledger_strict(ledger_path) {
        Ok(e) => e,
        Err(reason) => return Some((false, format!("{{\"ok\":false,\"reason\":\"{}\"}}", reason))),
    };
    let (triple, canonical) = match stat_claim_ledger_register(&entries, metric, arm_pair, unit, cid) {
        Ok(pair) => pair,
        Err(reason) => return Some((false, format!("{{\"ok\":false,\"reason\":\"{}\"}}", reason))),
    };
    if !entries.iter().any(|(k, _)| k == &triple) {
        entries.push((triple.clone(), canonical.clone()));
    }
    if !ensure_parent_dir(ledger_path) {
        return Some((false, "{\"ok\":false,\"reason\":\"LEDGER_DIR_CREATE_FAILED\"}".to_string()));
    }
    let obj = JsonVal::Obj(entries.into_iter().map(|(k, v)| (k, JsonVal::Str(v))).collect());
    fs::write(ledger_path, format!("{}\n", serialize_json_iterative(&obj))).ok()?;
    Some((true, format!("{{\"ok\":true,\"triple\":\"{}\",\"canonical_claim_id\":\"{}\"}}", json_escape(&triple), json_escape(&canonical))))
}

/// Extracts a `Vec<String>` from an array field on a `JsonVal`.
/// @outputs: 1
fn extract_str_arr(obj: &JsonVal, key: &str) -> Vec<String> {
    let Some(arr) = obj.get(key).and_then(|v| v.as_arr()) else {
        return Vec::new();
    };
    arr.iter().filter_map(|v| v.as_json_str().map(str::to_string)).collect()
}

/// Scores one exam item across 2 submission runs.
/// @outputs: 1
fn eval_exam_item_runs(item: &JsonVal, run0: &JsonVal, run1: &JsonVal) -> (bool, bool) {
    let iid = item.get("id").and_then(|v| v.as_json_str()).unwrap_or("");
    let r1 = run0.get(iid);
    let r2 = run1.get(iid);
    let (v1, l1, t1, rw1) = r1.map_or(("", "", Vec::new(), ""), |r| (
        r.get("verdict").and_then(|v| v.as_json_str()).unwrap_or(""),
        r.get("level").and_then(|v| v.as_json_str()).unwrap_or(""),
        extract_str_arr(r, "traps"),
        r.get("rewritten_claim").and_then(|v| v.as_json_str()).unwrap_or(""),
    ));
    let (v2, l2, t2, rw2) = r2.map_or(("", "", Vec::new(), ""), |r| (
        r.get("verdict").and_then(|v| v.as_json_str()).unwrap_or(""),
        r.get("level").and_then(|v| v.as_json_str()).unwrap_or(""),
        extract_str_arr(r, "traps"),
        r.get("rewritten_claim").and_then(|v| v.as_json_str()).unwrap_or(""),
    ));
    let consistent = stat_exam_runs_consistent(v1, l1, &t1, rw1, v2, l2, &t2, rw2);
    let gv = item.get("expected_verdict").and_then(|v| v.as_json_str()).unwrap_or("");
    let gl = item.get("expected_level").and_then(|v| v.as_json_str()).unwrap_or("");
    let pol = item.get("polarity").and_then(|v| v.as_json_str()).unwrap_or("");
    let req = extract_str_arr(item, "required_traps");
    let rw_req = item.get("rewrite_required").and_then(|v| v.as_bool()).unwrap_or(false);
    let ok1 = stat_exam_check_item(gv, gl, pol, &req, rw_req, v1, l1, &t1, rw1);
    let ok2 = stat_exam_check_item(gv, gl, pol, &req, rw_req, v2, l2, &t2, rw2);
    (consistent, ok1 && ok2)
}

/// Increments one of `(pos, neg, multi)` counters by polarity string.
/// @outputs: 4
fn bump_polarity(pol: &str, pos: &mut usize, neg: &mut usize, mul: &mut usize) {
    if pol == "positive" {
        *pos += 1;
    } else if pol == "negative" {
        *neg += 1;
    } else if pol == "multi" {
        *mul += 1;
    }
}

/// Counts unique IDs and polarity totals for the 114-item bank.
/// @outputs: 1
fn count_exam_bank(items: &[JsonVal]) -> (usize, usize, usize, usize) {
    let mut ids: Vec<&str> = Vec::with_capacity(items.len());
    let (mut pos_tot, mut neg_tot, mut mul_tot) = (0usize, 0usize, 0usize);
    for item in items {
        let id = item.get("id").and_then(|v| v.as_json_str()).unwrap_or("");
        if !id.is_empty() {
            ids.push(id);
        }
        let pol = item.get("polarity").and_then(|v| v.as_json_str()).unwrap_or("");
        bump_polarity(pol, &mut pos_tot, &mut neg_tot, &mut mul_tot);
    }
    ids.sort_unstable();
    ids.dedup();
    (ids.len(), pos_tot, neg_tot, mul_tot)
}

/// Handles `score-exam --exam <path> --submissions <path>`.
/// @outputs: 3
fn handle_score_exam_cmd(args: &[String]) -> Option<(bool, String)> {
    let exam_path = find_flag_val(args, "--exam")?;
    let sub_path = find_flag_val(args, "--submissions")?;
    let exam_doc = parse_json(&fs::read_to_string(exam_path).ok()?).ok()?;
    let sub_doc = parse_json(&fs::read_to_string(sub_path).ok()?).ok()?;
    let items = exam_doc.get("items")?.as_arr()?;
    let runs = sub_doc.get("runs")?.as_arr()?;
    let (uniq_cnt, pos_tot, neg_tot, mul_tot) = count_exam_bank(items);
    let runs_obj_ok = runs.len() >= 2 && matches!(runs[0], JsonVal::Obj(_)) && matches!(runs[1], JsonVal::Obj(_));
    if stat_exam_validate_bank_counts(items.len(), pos_tot, neg_tot, mul_tot, uniq_cnt).is_err() || !runs_obj_ok {
        let err = JsonVal::Obj(vec![("ok".into(), JsonVal::Bool(false)), ("reason".into(), JsonVal::Str("INVALID_EXAM_BANK_OR_RUNS".into()))]);
        return Some((false, serialize_json_iterative(&err)));
    }
    let (mut pos_ok, mut neg_fb, mut mul_ok, mut inc) = (0usize, 0usize, 0usize, 0usize);
    for item in items {
        let (cons, passed) = eval_exam_item_runs(item, &runs[0], &runs[1]);
        inc += usize::from(!cons);
        let pol = item.get("polarity").and_then(|v| v.as_json_str()).unwrap_or("");
        pos_ok += usize::from(pol == "positive" && passed);
        neg_fb += usize::from(pol == "negative" && !passed);
        mul_ok += usize::from(pol == "multi" && passed);
    }
    let ok = pos_ok == 54 && mul_ok == 6 && neg_fb == 0 && inc == 0;
    let out_obj = JsonVal::Obj(vec![
        ("ok".into(), JsonVal::Bool(ok)),
        ("total".into(), JsonVal::Num(114.0)),
        ("pos_passed".into(), JsonVal::Num(pos_ok as f64)),
        ("pos_total".into(), JsonVal::Num(54.0)),
        ("multi_passed".into(), JsonVal::Num(mul_ok as f64)),
        ("multi_total".into(), JsonVal::Num(6.0)),
        ("neg_false_blocks".into(), JsonVal::Num(neg_fb as f64)),
        ("neg_total".into(), JsonVal::Num(54.0)),
        ("inconsistent_runs".into(), JsonVal::Num(inc as f64)),
    ]);
    Some((ok, serialize_json_iterative(&out_obj)))
}

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: stat_gate_cli <wilson|...|run|recompute|claim-register|score-exam> ...");
        return ExitCode::FAILURE;
    }
    let cmd = args[1].as_str();
    let routed = match cmd {
        "run" => handle_run_cmd(&args),
        "recompute" => handle_recompute_cmd(&args),
        "claim-register" => handle_claim_register_cmd(&args),
        "score-exam" => handle_score_exam_cmd(&args),
        _ => dispatch_inner_math(cmd, &args).map(|s| (true, s)),
    };
    match routed {
        Some((ok, json)) => {
            println!("{}", json);
            if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
        }
        None => {
            eprintln!("{{\"error\":\"invalid arguments or domain guard rejected\"}}");
            ExitCode::FAILURE
        }
    }
}
