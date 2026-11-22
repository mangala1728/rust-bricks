// SPDX-License-Identifier: MIT
// ## Map
// - Role: core/stat_exam_ledger — Pure Rust C13 claim ledger, L-4 canonical receipt hash, and 114-item exam scorer
// - Inputs: Claim triples, CLI args/output JsonVal pairs, 114-item exam vectors & 2-run submissions
// - Outputs: Canonical claim registration, 16-char SHA-256 receipt hash, deterministic item/run validation
// - Invariants: Zero external crates, zero recursion (R09-7), nesting <= 2 (R09-3), JPL <= 60 lines/fn, zero unproved unwrap

use crate::crypto_vault::sha256_hex;
use crate::encoding_base64_json::{JsonVal, serialize_json_iterative};
use crate::stat_gate_core::stat_check_wording_level;

/// Escapes backslash and pipe characters in ledger key components.
/// @outputs: 1
fn escape_pipe_component(s: &str) -> String {
    s.replace('\\', "\\\\").replace('|', "\\|")
}

/// Registers or validates a bidirectional C13 claim_id mapping `(metric|arm_pair|unit) <-> claim_id`.
/// @outputs: 4
pub fn stat_claim_ledger_register(
    entries: &[(String, String)],
    metric: &str,
    arm_pair: &str,
    unit: &str,
    proposed_claim_id: &str,
) -> Result<(String, String), &'static str> {
    let m = metric.trim();
    let a = arm_pair.trim();
    let u = unit.trim();
    let cid = proposed_claim_id.trim();
    if m.is_empty() || a.is_empty() || u.is_empty() || cid.is_empty() {
        return Err("EMPTY_CLAIM_FIELD");
    }
    let triple = format!("{}|{}|{}", escape_pipe_component(m), escape_pipe_component(a), escape_pipe_component(u));
    for (existing_triple, existing_cid) in entries {
        if existing_triple == &triple && existing_cid != cid {
            return Err("C13_CLAIM_ID_ALIAS_REJECTED");
        }
        if existing_cid == cid && existing_triple != &triple {
            return Err("C13_CLAIM_ID_COLLISION");
        }
    }
    Ok((triple, cid.to_string()))
}

/// Computes canonical 16-char SHA-256 L-4 recomputation receipt hash matching Python sorted compact JSON.
/// @outputs: 1
pub fn stat_receipt_canonical_hash16(
    script_version: &str,
    bin_sha16: &str,
    cli_args: &[String],
    output_pairs_sorted: &[(String, JsonVal)],
) -> String {
    let mut filtered: Vec<(String, JsonVal)> = Vec::new();
    for (k, v) in output_pairs_sorted {
        if k != "cli_args" && k != "output_sha256_16" && k != "bin_sha256_16" {
            filtered.push((k.clone(), v.clone()));
        }
    }
    filtered.sort_by(|a, b| a.0.cmp(&b.0));
    let args_val = JsonVal::Arr(cli_args.iter().map(|s| JsonVal::Str(s.clone())).collect());
    let root = JsonVal::Obj(vec![
        ("args".to_string(), args_val),
        ("bin_sha256_16".to_string(), JsonVal::Str(bin_sha16.to_string())),
        ("output".to_string(), JsonVal::Obj(filtered)),
        ("script_version".to_string(), JsonVal::Str(script_version.to_string())),
    ]);
    let canonical = serialize_json_iterative(&root);
    let hex = sha256_hex(canonical.as_bytes());
    hex.chars().take(16).collect()
}

/// Validates 114-item exam bank counts (54 positive + 54 negative + 6 multi, unique IDs).
/// @outputs: 3
/// @domain: total [0..100000]
/// @domain: pos [0..100000]
/// @domain: neg [0..100000]
/// @domain: multi [0..100000]
/// @domain: unique_ids [0..100000]
pub fn stat_exam_validate_bank_counts(
    total: usize,
    pos: usize,
    neg: usize,
    multi: usize,
    unique_ids: usize,
) -> Result<(), &'static str> {
    if total > 100_000 || pos > 100_000 || neg > 100_000 || multi > 100_000 || unique_ids > 100_000 {
        return Err("DOMAIN_OUT_OF_RANGE");
    }
    if total != 114 || pos != 54 || neg != 54 || multi != 6 || unique_ids != total {
        return Err("BANK_COUNT_MISMATCH");
    }
    Ok(())
}

/// Maps Chinese evidence level label to numeric code (`0=描述`, `1=關聯`, `2=因果`).
/// @outputs: 3
pub fn stat_exam_level_code(level: &str) -> u8 {
    match level.trim() {
        "關聯" => 1,
        "因果" => 2,
        _ => 0,
    }
}

/// Evaluates a single 114-item exam submission against gold criteria and Rust wording rules.
/// @outputs: 2
pub fn stat_exam_check_item(
    gold_verdict: &str,
    gold_level: &str,
    polarity: &str,
    req_traps: &[String],
    rewrite_required: bool,
    sub_verdict: &str,
    sub_level: &str,
    got_traps: &[String],
    rewritten_claim: &str,
) -> bool {
    if sub_verdict != gold_verdict || sub_level != gold_level {
        return false;
    }
    if polarity == "negative" && !got_traps.is_empty() {
        return false;
    }
    if polarity != "negative" && (!req_traps.iter().all(|t| got_traps.contains(t)) || !got_traps.iter().all(|t| req_traps.contains(t))) {
        return false;
    }
    if rewrite_required {
        let rw = rewritten_claim.trim();
        if rw.chars().count() < 6 {
            return false;
        }
        let hits = stat_check_wording_level(stat_exam_level_code(gold_level), rw);
        return hits.is_empty();
    }
    true
}

/// Checks whether two runs produced identical verdict, level, sorted traps, and trimmed rewrite.
/// @outputs: 2
pub fn stat_exam_runs_consistent(
    v1: &str,
    l1: &str,
    t1: &[String],
    rw1: &str,
    v2: &str,
    l2: &str,
    t2: &[String],
    rw2: &str,
) -> bool {
    if v1 != v2 || l1 != l2 || rw1.trim() != rw2.trim() {
        return false;
    }
    let mut s1: Vec<&str> = t1.iter().map(|s| s.as_str()).collect();
    let mut s2: Vec<&str> = t2.iter().map(|s| s.as_str()).collect();
    s1.sort_unstable();
    s2.sort_unstable();
    s1 == s2
}
