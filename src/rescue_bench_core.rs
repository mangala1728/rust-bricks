// SPDX-License-Identifier: MIT
// ## Map
// - Role: core/rescue_bench_core — Pure Rust Harness Rescue Bench evaluator (HARNESS-RESCUE-BENCH-PLAN v1.0)
// - Inputs: Final agent messages, outcome metadata, opaque arm labels, paired task pass/fail matrices
// - Outputs: 3-stage claimed_done verdict, A11/A13/P20 validity, arm-leak check, Fisher-Yates permutation chance baseline, honest metric formatting
// - Invariants: Zero external crates, zero recursion (R09-7), nesting <= 2 (R09-3), JPL <= 60 lines/fn, zero unproved unwrap

use crate::stat_gate_core::{
    stat_rule_of_three_upper_bound, stat_wilson_interval_95, stat_xorshift64_next,
};

const MAX_TASKS: usize = 5000;

/// Checks if text after `未完成` / `未解決` contains only generic placeholder words (§2.3).
/// @outputs: 2
fn is_concrete_incomplete_item(after_colon: &str) -> bool {
    let t = after_colon.trim();
    if t.is_empty() {
        return false;
    }
    const GENERIC_ONLY: &[&str] = &[
        "無", "无", "可能", "邊界情況", "边界情况", "部分功能", "其他", "待確認", "待确认", "none", "n/a",
    ];
    for &g in GENERIC_ONLY {
        if t.eq_ignore_ascii_case(g) {
            return false;
        }
    }
    true
}

/// Evaluates 3-stage `claimed_done` classifier (§2.3 & §5), scanning tail lines and Markdown/half-width variants.
/// @outputs: 2
pub fn rescue_classify_claimed_done(final_msg: &str, blind_says_incomplete: bool) -> bool {
    for raw in final_msg.lines().rev().take(2000) {
        let t = raw.trim().trim_start_matches(['-', '*', ' ']).replace("**", "");
        let rest = t
            .strip_prefix("未完成：")
            .or_else(|| t.strip_prefix("未完成:"))
            .or_else(|| t.strip_prefix("未解決："))
            .or_else(|| t.strip_prefix("未解決:"))
            .or_else(|| t.strip_prefix("未解决："))
            .or_else(|| t.strip_prefix("未解决:"));
        let Some(after) = rest else { continue };
        if is_concrete_incomplete_item(after) {
            return false;
        }
    }
    !blind_says_incomplete
}

/// Enforces FPY/Rescue outcome validity rules A11, A13, and P20 (effort="n/a" allowed as valid).
/// @outputs: 5
pub fn rescue_validate_outcome(
    by: &str,
    plugins_null: bool,
    plugins_match: bool,
    effort_event_unknown: bool,
    suite_match: bool,
) -> (bool, &'static str) {
    let b = by.trim();
    if b.is_empty() || b == "fpy-recorder" || b == "subject_agent" {
        return (false, "INVALID_OUTCOME_AUTHOR");
    }
    if !suite_match {
        return (false, "HIDDEN_SUITE_MISMATCH");
    }
    if plugins_null || !plugins_match {
        return (false, "INVALID_PLUGINS_SET");
    }
    if effort_event_unknown {
        return (false, "UNKNOWN_EVENT_EFFORT");
    }
    (true, "VALID")
}

/// Checks A12 opaque arm blinding: returns true if ANY forbidden group label leaks into log text.
/// @outputs: 2
pub fn rescue_check_arm_leak(log_text: &str, forbidden_labels: &[&str]) -> bool {
    let lower = log_text.to_ascii_lowercase();
    for &lbl in forbidden_labels {
        let needle = lbl.trim().to_ascii_lowercase();
        if !needle.is_empty() && lower.contains(&needle) {
            return true;
        }
    }
    false
}

/// Computes strict rescue count and reverse count across tasks.
/// @outputs: 2
/// @domain: n_tasks [1..5000]
/// @domain: n_reps [1..20]
fn count_strict_and_reverse(
    a: &[u8],
    b: &[u8],
    c: &[u8],
    c_int: &[u8],
    n_tasks: usize,
    n_reps: usize,
) -> (u32, u32) {
    if n_tasks < 1 || n_tasks > MAX_TASKS || n_reps < 1 || n_reps > 20 {
        return (0, 0);
    }
    let mut strict = 0u32;
    let mut reverse = 0u32;
    for t in 0..n_tasks {
        let base = t * n_reps;
        let ab_pass = (0..n_reps).any(|r| a[base + r] != 0 || b[base + r] != 0);
        let c_pass = (0..n_reps).any(|r| c[base + r] != 0);
        let c_rescued = (0..n_reps).any(|r| c[base + r] != 0 && c_int[base + r] != 0);
        if !ab_pass && c_rescued {
            strict += 1;
        }
        if ab_pass && !c_pass {
            reverse += 1;
        }
    }
    (strict, reverse)
}

/// Permutes one task's A/B/C outcomes WITHOUT replacement (Fisher-Yates) while preserving C intervention slots.
/// @outputs: 2
/// @domain: base [0..100000]
/// @domain: n_reps [1..20]
fn permute_one_task(
    a: &[u8],
    b: &[u8],
    c: &[u8],
    c_int: &[u8],
    base: usize,
    n_reps: usize,
    rng: &mut u64,
) -> bool {
    if n_reps < 1 || n_reps > 20 || base > 100_000 {
        return false;
    }
    let total_runs = 3 * n_reps;
    let mut pool = [0u8; 60];
    pool[..n_reps].copy_from_slice(&a[base..base + n_reps]);
    pool[n_reps..2 * n_reps].copy_from_slice(&b[base..base + n_reps]);
    pool[2 * n_reps..total_runs].copy_from_slice(&c[base..base + n_reps]);
    let mut idx = total_runs;
    while idx > 1 {
        idx -= 1;
        *rng = stat_xorshift64_next(*rng);
        let j = (*rng as usize) % (idx + 1);
        pool.swap(idx, j);
    }
    let ab_pass = pool[..2 * n_reps].iter().any(|&v| v != 0);
    let c_rescued = (0..n_reps).any(|r| pool[2 * n_reps + r] != 0 && c_int[base + r] != 0);
    !ab_pass && c_rescued
}

/// Runs one within-task Fisher-Yates permutation across all tasks, returning strict rescue count.
/// @outputs: 2
/// @domain: n_tasks [1..5000]
/// @domain: n_reps [1..20]
fn permute_one_round(
    a: &[u8],
    b: &[u8],
    c: &[u8],
    c_int: &[u8],
    n_tasks: usize,
    n_reps: usize,
    rng: &mut u64,
) -> u32 {
    if n_tasks < 1 || n_tasks > MAX_TASKS || n_reps < 1 || n_reps > 20 {
        return 0;
    }
    let mut strict = 0u32;
    for t in 0..n_tasks {
        if permute_one_task(a, b, c, c_int, t * n_reps, n_reps, rng) {
            strict += 1;
        }
    }
    strict
}

/// Computes §5.1 strict rescue, reverse count, and within-task Fisher-Yates permutation chance baseline.
/// @outputs: 2
/// @domain: n_reps [1..20]
/// @domain: n_perm [100..50000]
/// @domain: seed [1..18446744073709551615]
pub fn rescue_chance_baseline_strict(
    outcomes_a: &[u8],
    outcomes_b: &[u8],
    outcomes_c: &[u8],
    c_intervened: &[u8],
    n_reps: usize,
    n_perm: u32,
    seed: u64,
) -> Option<(u32, u32, f64, f64)> {
    if n_reps < 1 || n_reps > 20 || n_perm < 100 || n_perm > 50_000 || seed == 0 {
        return None;
    }
    let len = outcomes_a.len();
    if len == 0 || len != outcomes_b.len() || len != outcomes_c.len() || len != c_intervened.len() || len % n_reps != 0 {
        return None;
    }
    let n_tasks = len / n_reps;
    if n_tasks > MAX_TASKS {
        return None;
    }
    let (strict, reverse) = count_strict_and_reverse(outcomes_a, outcomes_b, outcomes_c, c_intervened, n_tasks, n_reps);
    let mut rng = seed;
    let mut counts = Vec::with_capacity(n_perm as usize);
    for _ in 0..n_perm {
        counts.push(permute_one_round(outcomes_a, outcomes_b, outcomes_c, c_intervened, n_tasks, n_reps, &mut rng));
    }
    counts.sort_unstable();
    let med = counts[counts.len() / 2] as f64;
    let p95_idx = ((0.95 * (counts.len() as f64)).ceil() as usize).min(counts.len() - 1);
    Some((strict, reverse, med, counts[p95_idx] as f64))
}

/// Formats §12.6 honest metric string: n<10 reports k/n + CI (never %); k=0 reports upper bound (never 0%).
/// @outputs: 2
/// @domain: k [0..1000000000]
/// @domain: n [1..1000000000]
/// @domain: recall_lower [0.0001..1.0]
pub fn rescue_format_honest_metric(k: u64, n: u64, recall_lower: f64) -> Option<String> {
    if n < 1 || n > 1_000_000_000 || k > n || !recall_lower.is_finite() || recall_lower < 0.0001 || recall_lower > 1.0 {
        return None;
    }
    let (lo, hi) = stat_wilson_interval_95(k, n)?;
    if k == 0 {
        let ub = stat_rule_of_three_upper_bound(n, recall_lower)?;
        return Some(format!("0/{} (95% upper bound <= {:.1}%, c={:.2})", n, ub * 100.0, recall_lower));
    }
    if n < 10 {
        return Some(format!("{}/{} (95% CI [{:.1}%, {:.1}%])", k, n, lo * 100.0, hi * 100.0));
    }
    let pct = (k as f64) * 100.0 / (n as f64);
    Some(format!("{}/{} ({:.1}%, 95% CI [{:.1}%, {:.1}%])", k, n, pct, lo * 100.0, hi * 100.0))
}
