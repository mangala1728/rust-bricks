// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/gate_acceptance — Pure-Rust R13 Traceability Matrix & 6-Sample Coverage Verifier (transpiled from `acceptance-gate.py`)
//! - Inputs: TSV traceability matrix text and `(spec_file, case_name, sample_type)` reference pairs
//! - Outputs: Verified `MatrixRow` records, requirement count, and anti-degenerate mutation checks
//! - Invariants:
//!   1. Every `REQ_ID` MUST include all 6 mandatory classes: `GoldenSample`, `LowerBoundPass`, `UpperBoundPass`, `LowerBoundFail`, `UpperBoundFail`, `NoiseAdversarial`.
//!   2. Every row must have `STATUS == "PASS"` and non-empty `VERIFIED_COMMIT`.
//!   3. Zero external dependencies, nesting depth <= 2, `@domain` guards enforced at entry.

use std::collections::{BTreeMap, BTreeSet};

pub const REQUIRED_HEADERS: &[&str] = &[
    "REQ_ID",
    "REQ_DESC",
    "SPEC_FILE",
    "CASE_NAME",
    "SAMPLE_TYPE",
    "STATUS",
    "VERIFIED_COMMIT",
];

pub const MANDATORY_SAMPLE_TYPES: &[&str] = &[
    "GoldenSample",
    "LowerBoundPass",
    "UpperBoundPass",
    "LowerBoundFail",
    "UpperBoundFail",
    "NoiseAdversarial",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    pub req_id: String,
    pub req_desc: String,
    pub spec_file: String,
    pub case_name: String,
    pub sample_type: String,
    pub status: String,
    pub verified_commit: String,
}

/// Parses a single data row using header index lookup.
/// @outputs: 2
fn build_matrix_row(header: &[&str], fields: &[&str]) -> MatrixRow {
    let mut map: BTreeMap<&str, &str> = BTreeMap::new();
    for (idx, col) in header.iter().enumerate() {
        let val = fields.get(idx).copied().unwrap_or("").trim();
        map.insert(col.trim(), val);
    }
    MatrixRow {
        req_id: map.get("REQ_ID").copied().unwrap_or("").to_string(),
        req_desc: map.get("REQ_DESC").copied().unwrap_or("").to_string(),
        spec_file: map.get("SPEC_FILE").copied().unwrap_or("").to_string(),
        case_name: map.get("CASE_NAME").copied().unwrap_or("").to_string(),
        sample_type: map.get("SAMPLE_TYPE").copied().unwrap_or("").to_string(),
        status: map.get("STATUS").copied().unwrap_or("").to_string(),
        verified_commit: map.get("VERIFIED_COMMIT").copied().unwrap_or("").to_string(),
    }
}

/// Parses TSV traceability matrix content and verifies column completeness.
/// @outputs: 6
/// @domain: max_lines [10..10000]
pub fn parse_matrix_tsv(tsv_content: &str, max_lines: usize) -> Result<Vec<MatrixRow>, Vec<String>> {
    if max_lines < 10 || max_lines > 10_000 {
        return Err(vec![format!("max_lines 超出範圍 [10..10000]: {}", max_lines)]);
    }
    let lines: Vec<&str> = tsv_content.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Err(vec!["追溯矩陣為空檔案".to_string()]);
    }
    let header: Vec<&str> = lines[0].split('\t').map(str::trim).collect();
    let missing: Vec<&str> = REQUIRED_HEADERS.iter().copied().filter(|c| !header.contains(c)).collect();
    if !missing.is_empty() {
        return Err(vec![format!("矩陣缺少必要欄位: {}", missing.join(", "))]);
    }
    let mut rows = Vec::new();
    for line in &lines[1..] {
        if rows.len() >= max_lines {
            return Err(vec![format!("超過最大行數上限 {}", max_lines)]);
        }
        let fields: Vec<&str> = line.split('\t').collect();
        rows.push(build_matrix_row(&header, &fields));
    }
    if rows.is_empty() {
        return Err(vec!["追溯矩陣無有效資料行".to_string()]);
    }
    Ok(rows)
}

/// Checks if a single requirement holds all 6 mandatory sample types.
/// @outputs: 1
fn find_missing_sample_types(present: &BTreeSet<String>) -> Vec<&'static str> {
    MANDATORY_SAMPLE_TYPES
        .iter()
        .copied()
        .filter(|req| !present.contains(*req))
        .collect()
}

/// Verifies every REQ_ID covers all 6 mandatory sample classes with PASS status and commit anchor.
/// Returns count of verified distinct `REQ_ID`s.
/// @outputs: 6
pub fn verify_req_sample_coverage(rows: &[MatrixRow]) -> Result<usize, Vec<String>> {
    if rows.is_empty() {
        return Err(vec!["追溯矩陣無資料行".to_string()]);
    }
    let mut req_samples: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for r in rows {
        if r.req_id.trim().is_empty() {
            return Err(vec!["存在未填寫 REQ_ID 的案例".to_string()]);
        }
        if r.status.trim() != "PASS" {
            return Err(vec![format!("需求 {} 案例 {} 狀態非 PASS: {}", r.req_id, r.case_name, r.status)]);
        }
        if r.verified_commit.trim().is_empty() {
            return Err(vec![format!("需求 {} 案例 {} 缺少 VERIFIED_COMMIT 錨點", r.req_id, r.case_name)]);
        }
        req_samples.entry(r.req_id.trim().to_string()).or_default().insert(r.sample_type.trim().to_string());
    }
    let mut errors = Vec::new();
    for (req_id, present) in &req_samples {
        let missing = find_missing_sample_types(present);
        if !missing.is_empty() {
            errors.push(format!("需求 {} 缺少必要驗收件類別: {}", req_id, missing.join(", ")));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(req_samples.len())
}

/// Verifies a single matrix row against spec entries.
/// @outputs: 3
fn check_single_case_link(row: &MatrixRow, spec_entries: &[(&str, &str, &str)]) -> Option<String> {
    let (_, _, spec_stype) = match spec_entries.iter().find(|(s, c, _)| *s == row.spec_file && *c == row.case_name) {
        Some(hit) => *hit,
        None => return Some(format!("規格檔 {} 中未找到案例: '{}'", row.spec_file, row.case_name)),
    };
    if !spec_stype.is_empty() && spec_stype != row.sample_type {
        return Some(format!(
            "案例 '{}' 類別不一致: 矩陣標為 {}，但規格檔標為 {}",
            row.case_name, row.sample_type, spec_stype
        ));
    }
    None
}

/// Verifies row `(spec_file, case_name, sample_type)` against loaded spec entries `(spec_file, case_name, sample_type)`.
/// @outputs: 2
pub fn verify_spec_case_links_pairs(
    rows: &[MatrixRow],
    spec_entries: &[(&str, &str, &str)],
) -> Result<(), Vec<String>> {
    let errors: Vec<String> = rows.iter().filter_map(|r| check_single_case_link(r, spec_entries)).collect();
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

/// Runs all 4 anti-degenerate synthetic defect mutations against loaded rows and spec entries.
/// @outputs: 2
pub fn run_acceptance_mutation_selftest(
    rows: &[MatrixRow],
    spec_entries: &[(&str, &str, &str)],
) -> bool {
    if rows.is_empty() {
        return false;
    }
    let mut1: Vec<MatrixRow> = rows.iter().filter(|r| r.sample_type != "LowerBoundPass").cloned().collect();
    if verify_req_sample_coverage(&mut1).is_ok() {
        return false;
    }
    let mut2: Vec<MatrixRow> = rows.iter().filter(|r| r.sample_type != "UpperBoundPass").cloned().collect();
    if verify_req_sample_coverage(&mut2).is_ok() {
        return false;
    }
    let mut mut3 = rows.to_vec();
    mut3[0].case_name = "不存在的幽靈測試案例".to_string();
    if verify_spec_case_links_pairs(&mut3, spec_entries).is_ok() {
        return false;
    }
    let mut mut4 = rows.to_vec();
    mut4[0].status = "FAIL".to_string();
    verify_req_sample_coverage(&mut4).is_err()
}
