// SPDX-License-Identifier: MIT
// ## Map
// - Role: core/redteam_receipt_core — Pure Rust Green-RedTeam TSV receipt formatter, parser, and verifier
// - Inputs: Red-team audit metadata (`scope`, `shots`, `confirmed_hits`, `verdict`) and TSV lines
// - Outputs: Sanitized TSV rows, parsed `RedteamRow` summaries, and `countersign` linkage status
// - Invariants: Zero external crates, zero recursion (R09-7), nesting <= 2 (R09-3), JPL <= 60 lines/fn, zero unproved unwrap

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedteamRow {
    pub ts: String,
    pub scope: String,
    pub patch_sha256: String,
    pub kid_model: String,
    pub shots: u32,
    pub confirmed_hits: u32,
    pub arbiter_model: String,
    pub verdict: String,
    pub promoted_challenges: String,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RedteamSummary {
    pub ok: bool,
    pub total_receipts: usize,
    pub matching_receipts: usize,
    pub verified_scopes: Vec<String>,
    pub latest: Option<RedteamRow>,
}

/// Sanitizes TSV field by collapsing tabs and newlines into spaces and trimming.
/// @outputs: 1
pub fn redteam_sanitize_tsv(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '\t' || ch == '\r' || ch == '\n' {
            out.push(' ');
        } else {
            out.push(ch);
        }
    }
    out.trim().to_string()
}

/// Formats one validated TSV row for `.ai-memory/redteam-receipts.tsv`.
/// @outputs: 4
/// @domain: shots [0..10000]
/// @domain: confirmed_hits [0..10000]
pub fn redteam_format_tsv_row(
    ts: &str,
    scope: &str,
    patch_sha: &str,
    kid_model: &str,
    shots: u32,
    confirmed_hits: u32,
    arbiter_model: &str,
    verdict: &str,
    promoted: &str,
    note: &str,
) -> Result<String, &'static str> {
    if shots > 10_000 || confirmed_hits > 10_000 {
        return Err("DOMAIN_OUT_OF_RANGE");
    }
    let s_scope = redteam_sanitize_tsv(scope);
    let s_verdict = redteam_sanitize_tsv(verdict);
    if s_scope.is_empty() {
        return Err("EMPTY_SCOPE");
    }
    if !matches!(s_verdict.as_str(), "PASS" | "DEGRADED" | "FAIL") {
        return Err("INVALID_VERDICT");
    }
    let s_prom = redteam_sanitize_tsv(promoted);
    let s_note = redteam_sanitize_tsv(note);
    Ok(format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        redteam_sanitize_tsv(ts),
        s_scope,
        redteam_sanitize_tsv(patch_sha),
        redteam_sanitize_tsv(kid_model),
        shots,
        confirmed_hits,
        redteam_sanitize_tsv(arbiter_model),
        s_verdict,
        if s_prom.is_empty() { "-" } else { &s_prom },
        if s_note.is_empty() { "-" } else { &s_note }
    ))
}

/// Parses one non-header TSV line into `RedteamRow` (strict CRLF trim & numeric parse).
/// @outputs: 2
pub fn redteam_parse_tsv_line(line: &str) -> Option<RedteamRow> {
    let clean = line.trim_end_matches(&['\r', '\n'][..]);
    let parts: Vec<&str> = clean.split('\t').collect();
    if parts.len() < 8 {
        return None;
    }
    let shots = parts[4].parse::<u32>().ok()?;
    let confirmed_hits = parts[5].parse::<u32>().ok()?;
    Some(RedteamRow {
        ts: parts[0].to_string(),
        scope: parts[1].to_string(),
        patch_sha256: parts[2].to_string(),
        kid_model: parts[3].to_string(),
        shots,
        confirmed_hits,
        arbiter_model: parts[6].to_string(),
        verdict: parts[7].to_string(),
        promoted_challenges: parts.get(8).copied().unwrap_or("-").to_string(),
        note: parts.get(9).copied().unwrap_or("-").to_string(),
    })
}

/// Summarizes parsed red-team rows where the LATEST receipt for a scope dictates its verdict.
/// @outputs: 1
pub fn redteam_summarize_rows(rows: &[RedteamRow], req_scope: &str) -> RedteamSummary {
    let req = req_scope.trim();
    let mut latest_by_scope: Vec<(String, &str)> = Vec::new();
    let mut matching: Vec<&RedteamRow> = Vec::new();
    for r in rows {
        if let Some(pos) = latest_by_scope.iter().position(|(s, _)| s == &r.scope) {
            latest_by_scope[pos].1 = r.verdict.as_str();
        } else {
            latest_by_scope.push((r.scope.clone(), r.verdict.as_str()));
        }
        if req.is_empty() || r.scope == req || r.scope == "ALL" {
            matching.push(r);
        }
    }
    let mut verified_scopes: Vec<String> = latest_by_scope
        .into_iter()
        .filter(|(_, v)| *v == "PASS")
        .map(|(s, _)| s)
        .collect();
    verified_scopes.sort();
    let latest = matching.last().copied().or_else(|| rows.last()).cloned();
    let ok = latest
        .as_ref()
        .map(|r| r.verdict == "PASS" || r.verdict == "DEGRADED")
        .unwrap_or(false);
    RedteamSummary { ok, total_receipts: rows.len(), matching_receipts: matching.len(), verified_scopes, latest }
}

/// Computes `countersign` red-team linkage status (`VERIFIED`, `SKIPPED`, or `MISSING`).
/// @outputs: 3
pub fn redteam_countersign_status(has_verified: bool, skip_reason: &str) -> &'static str {
    if has_verified {
        return "VERIFIED";
    }
    if !skip_reason.trim().is_empty() {
        return "SKIPPED";
    }
    "MISSING"
}
