// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/stat_gate_cli — Pure Rust CLI wrapper for STAT-GATE-SPEC v1.0 Q8
// - Inputs: Subcommand + numeric/text CLI arguments
// - Outputs: Deterministic single-line JSON to stdout; exit 0 on success, 1 on invalid input
// - Invariants: Zero external crates, zero recursion, nesting <= 2, JPL <= 60 lines/fn, zero unproved unwrap

use rust_bricks::stat_gate_core::{
    stat_bayesian_ppv, stat_bayesian_trial_upgrade, stat_check_wording_level,
    stat_detect_t4_trigger, stat_holm_bonferroni_reject, stat_median_and_mad,
    stat_p_chart_limits, stat_paired_bootstrap_ci, stat_required_paired_sample_size,
    stat_robust_z_score, stat_rule_of_three_upper_bound, stat_stratified_permutation_p,
    stat_wilson_interval_95,
};
use std::env;
use std::process::ExitCode;

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

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: stat_gate_cli <wilson|rule-of-three|bayesian-upgrade|bayesian-ppv|holm|mad|power|pchart|perm|bootstrap|wording|t4> ...");
        return ExitCode::FAILURE;
    }
    let cmd = args[1].as_str();
    let res = handle_scalar_cmd(cmd, &args)
        .or_else(|| handle_spc_cmd(cmd, &args))
        .or_else(|| handle_vector_cmd(cmd, &args));
    match res {
        Some(json) => {
            println!("{}", json);
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("{{\"error\":\"invalid arguments or domain guard rejected\"}}");
            ExitCode::FAILURE
        }
    }
}
