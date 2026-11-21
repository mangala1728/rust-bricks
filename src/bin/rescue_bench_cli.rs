// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/rescue_bench_cli — Pure Rust CLI for HARNESS-RESCUE-BENCH-PLAN v1.0
// - Inputs: Subcommands (`claimed-done`, `validate-outcome`, `arm-leak`, `chance-baseline`, `honest-metric`)
// - Outputs: Deterministic single-line JSON to stdout
// - Invariants: Zero external crates, nesting <= 2, JPL <= 60 lines/fn, zero unproved unwrap

use rust_bricks::rescue_bench_core::{
    rescue_chance_baseline_strict, rescue_check_arm_leak, rescue_classify_claimed_done,
    rescue_format_honest_metric, rescue_validate_outcome,
};
use std::env;
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

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rescue_bench_cli <claimed-done|validate-outcome|arm-leak|chance-baseline|honest-metric> ...");
        return ExitCode::FAILURE;
    }
    let cmd = args[1].as_str();
    let res = handle_guard_cmd(cmd, &args).or_else(|| handle_stat_cmd(cmd, &args));
    match res {
        Some(json) => {
            println!("{}", json);
            ExitCode::SUCCESS
        }
        None => ExitCode::FAILURE,
    }
}
