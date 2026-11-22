// SPDX-License-Identifier: MIT
// ## Map
// - Role: app/bin/pr24_msa_runner — Pure Rust MSA calibration runner for PR #24 & Red-Team cores
// - Inputs: Optional `--mutate` flag
// - Outputs: Confusion matrix + Wilson 95% CI across 3 repetitions (Recall >= 90%, Precision >= 90%, Repeatability == 1.00)
// - Invariants: Zero external crates, zero recursion (R09-7), nesting <= 2 (R09-3), JPL <= 60 lines/fn, zero unproved unwrap

use rust_bricks::encoding_base64_json::JsonVal;
use rust_bricks::fpy_recorder_core::{
    fpy_detect_n1_and_n1b, fpy_detect_n4_prompt, fpy_evaluate_proposal_transition, fpy_redact_secrets,
};
use rust_bricks::redteam_receipt_core::{
    redteam_countersign_status, redteam_format_tsv_row, redteam_parse_tsv_line, redteam_summarize_rows,
};
use rust_bricks::rescue_bench_core::{
    rescue_check_arm_leak, rescue_classify_claimed_done, rescue_validate_outcome,
};
use rust_bricks::stat_exam_ledger::{stat_claim_ledger_register, stat_exam_check_item, stat_receipt_canonical_hash16};
use rust_bricks::stat_gate_core::{
    stat_check_wording_level, stat_detect_t4_trigger, stat_wilson_interval_95,
};
use std::env;
use std::process::ExitCode;

/// Runs one repetition of positive & negative MSA vectors for Stat-Gate & Ledger.
/// @outputs: 1
fn eval_stat_vectors(mutate: bool) -> Vec<(bool, bool)> {
    let mut pairs = Vec::new();
    let e1 = vec![
        ("m|a|u".to_string(), "C01".to_string()),
        ("acc\\|armA|armB|%".to_string(), "C10".to_string()),
    ];
    let c_ok = stat_claim_ledger_register(&e1, "m", "a", "u", "C01").is_ok();
    let c_pipe_distinct = stat_claim_ledger_register(&e1, "acc", "armA|armB", "%", "C11").is_ok();
    let c_alias = stat_claim_ledger_register(&e1, "m", "a", "u", "C02").is_err();
    let h1 = stat_receipt_canonical_hash16("v1", "abc", &["wilson".into()], &[("k".into(), JsonVal::Num(5.0))]);
    let w_exempt = stat_check_wording_level(0, "因為樣本不足無法下因果結論").is_empty();
    let w_reject = if mutate { false } else { !stat_check_wording_level(0, "因為樣本數夠多所以證明有效").is_empty() };
    let t4_non = !stat_detect_t4_trigger("v2 版本 C01 與 1.5 倍");
    let exam_ok = stat_exam_check_item("PASS", "描述", "negative", &[], false, "PASS", "描述", &[], "");
    let exam_spam = !stat_exam_check_item("PASS", "描述", "negative", &[], false, "PASS", "描述", &["C01".into()], "");
    let exam_pos_spam = !stat_exam_check_item("FAIL", "描述", "positive", &["C01".into()], false, "FAIL", "描述", &["C01".into(), "C02".into()], "");
    pairs.push((true, c_ok && c_pipe_distinct && !h1.is_empty()));
    pairs.push((true, w_exempt && t4_non && exam_ok));
    pairs.push((false, !c_alias));
    pairs.push((false, !w_reject));
    pairs.push((false, !exam_spam || !exam_pos_spam));
    pairs
}

/// Runs one repetition of positive & negative MSA vectors for FPY & Rescue Bench.
/// @outputs: 1
fn eval_fpy_rescue_vectors(mutate: bool) -> Vec<(bool, bool)> {
    let mut pairs = Vec::new();
    let (red, tags) = fpy_redact_secrets("-----BEGIN RSA PRIVATE KEY-----\nabc");
    let pem_ok = !red.contains("BEGIN RSA") && red.contains("private-key-truncated") && tags.contains(&"private-key-block");
    let (_, _, sw) = fpy_detect_n1_and_n1b("echo hi>out.txt", Some(0), false, false);
    let n4_desk = fpy_detect_n4_prompt("check desktop layout", "user", false, false, false).1.is_empty();
    let prop_ret = if mutate { false } else { fpy_evaluate_proposal_transition("retired", 10, 0, 5, 3, 7, 0, false).0 == "retired" };
    let res_ok = rescue_validate_outcome("runner-1", false, true, false, true).0;
    let res_no_leak = !rescue_check_arm_leak("甲狀腺功能", &["乙", "丙1"]);
    let res_real_leak = rescue_check_arm_leak("arm_a", &["arm_a", "arm_b"]);
    let res_done = !rescue_classify_claimed_done("- **未完成**: backend/app.py 測試失敗", false);
    pairs.push((true, pem_ok && sw && n4_desk && res_ok && res_no_leak && res_done));
    pairs.push((false, !prop_ret));
    pairs.push((false, !res_real_leak));
    pairs
}

/// Runs one repetition of Red-Team receipt core vectors.
/// @outputs: 1
fn eval_redteam_vectors(mutate: bool) -> Vec<(bool, bool)> {
    let mut pairs = Vec::new();
    let row_ok = redteam_format_tsv_row("2026-11-22T00:00:00Z", "stat-gate", "abc", "flash", 8, 7, "Argon", "PASS", "c1", "ok");
    let row_fail = redteam_format_tsv_row("2026-11-22T01:00:00Z", "stat-gate", "def", "flash", 8, 7, "Argon", "FAIL", "c1", "bad");
    let parsed_crlf = row_ok.as_deref().ok().and_then(|s| redteam_parse_tsv_line(&format!("{}\r\n", s)));
    let parsed_fail = row_fail.as_deref().ok().and_then(redteam_parse_tsv_line);
    let summary_ok = parsed_crlf.as_ref().map(|r| redteam_summarize_rows(std::slice::from_ref(r), "stat-gate").ok).unwrap_or(false);
    let pass_then_fail_rejected = match (parsed_crlf, parsed_fail) {
        (Some(p), Some(f)) => !redteam_summarize_rows(&[p, f], "stat-gate").ok,
        _ => false,
    };
    let cs_miss = if mutate { false } else { redteam_countersign_status(false, "") == "MISSING" };
    pairs.push((true, summary_ok));
    pairs.push((false, !pass_then_fail_rejected));
    pairs.push((false, !cs_miss));
    pairs
}

/// Computes TP, TN, FP, FN across 3 repetitions and evaluates MSA thresholds.
/// @outputs: 2
fn run_all_reps(mutate: bool) -> (bool, usize, usize, usize, usize, f64) {
    let mut all_runs: Vec<Vec<(bool, bool)>> = Vec::new();
    for _ in 0..3 {
        let mut r = eval_stat_vectors(mutate);
        r.extend(eval_fpy_rescue_vectors(mutate));
        r.extend(eval_redteam_vectors(mutate));
        all_runs.push(r);
    }
    let mut mismatches = 0usize;
    let (mut tp, mut tn, mut fp, mut fn_c) = (0usize, 0usize, 0usize, 0usize);
    for (idx, &(exp, got)) in all_runs[0].iter().enumerate() {
        mismatches += usize::from(all_runs[1][idx].1 != got || all_runs[2][idx].1 != got);
        tp += usize::from(exp && got);
        tn += usize::from(!exp && !got);
        fp += usize::from(!exp && got);
        fn_c += usize::from(exp && !got);
    }
    let rep = if mismatches == 0 { 1.0 } else { 0.0 };
    let rec = (tp as f64) / ((tp + fn_c).max(1) as f64);
    let pre = (tp as f64) / ((tp + fp).max(1) as f64);
    let ok = tp + fn_c > 0 && tn + fp > 0 && rep == 1.0 && rec >= 0.90 && pre >= 0.90;
    (ok, tp, tn, fp, fn_c, rep)
}

/// CLI entry point.
/// @outputs: 2
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mutate = args.iter().any(|a| a == "--mutate");
    let (ok, tp, tn, fp, fn_c, rep) = run_all_reps(mutate);
    let (r_lo, r_hi) = stat_wilson_interval_95(tp as u64, (tp + fn_c) as u64).unwrap_or((0.0, 1.0));
    let (p_lo, p_hi) = stat_wilson_interval_95(tp as u64, (tp + fp) as u64).unwrap_or((0.0, 1.0));
    let rec_pct = (tp as f64) * 100.0 / ((tp + fn_c).max(1) as f64);
    let pre_pct = (tp as f64) * 100.0 / ((tp + fp).max(1) as f64);
    println!(
        "PR24 PURE-RUST MSA{} | TP={}/{} TN={}/{} FP={} FN={} | Recall={:.1}% [{:.1}%,{:.1}%] Precision={:.1}% [{:.1}%,{:.1}%] Repeatability={:.2}",
        if mutate { " [MUTATE]" } else { "" },
        tp, tp + fn_c, tn, tn + fp, fp, fn_c, rec_pct, r_lo * 100.0, r_hi * 100.0, pre_pct, p_lo * 100.0, p_hi * 100.0, rep
    );
    if mutate {
        return if !ok { ExitCode::SUCCESS } else { ExitCode::FAILURE };
    }
    if ok { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
