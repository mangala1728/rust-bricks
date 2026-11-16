// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/gate_log_lifecycle — Pure-Rust DECISIONS.md & LESSONS.md Governance Checker (transpiled from `decisions-status.py` & `lessons-gate.py`)
//! - Inputs: Markdown text of `context/DECISIONS.md` and `context/LESSONS.md`
//! - Outputs: Validated `DecisionRelation` / `LessonRow` entries and deterministic `DECISIONS-STATUS.md` text
//! - Invariants:
//!   1. `extract_decision_targets` ignores `D-NNNN` inside parentheses `()` or `（）` (D-0077 regression).
//!   2. Replacement/correction must always come strictly from newer `D-NNNN` (`src_num > tgt_num`).
//!   3. Zero external dependencies, zero recursion, max control-flow nesting depth <= 2, max fn lines <= 60.

use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct DecisionRelation {
    pub target: String,
    pub kind: String,
    pub source: String,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonRow {
    pub id: String,
    pub lesson: String,
    pub anchor: String,
    pub first_ref: String,
    pub repeat_ref: String,
}

/// Checks whether a slice starts with `D-NNNN` (4 ASCII digits).
/// @outputs: 2
pub fn is_d_id_at(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 6 && b[0] == b'D' && b[1] == b'-' && b[2..6].iter().all(|c| c.is_ascii_digit())
}

/// Parses numeric ID suffix of `D-NNNN` into `u32`.
/// @outputs: 2
fn parse_d_num(id: &str) -> Option<u32> {
    if !is_d_id_at(id) {
        return None;
    }
    id[2..6].parse::<u32>().ok()
}

/// Collects byte indices of top-level `D-NNNN` tokens outside parentheses `()` / `（）`.
/// @outputs: 1
fn find_top_level_d_indices(rest: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut depth: u32 = 0;
    for (idx, ch) in rest.char_indices() {
        if ch == '（' || ch == '(' {
            depth = depth.saturating_add(1);
        } else if ch == '）' || ch == ')' {
            depth = depth.saturating_sub(1);
        } else if depth == 0 && is_d_id_at(&rest[idx..]) {
            found.push(idx);
        }
    }
    found
}

/// Trims separator punctuation and outer full-width parentheses from a relation note.
/// @outputs: 1
fn clean_target_note(raw: &str) -> String {
    let trimmed = raw.trim_matches(|c: char| c.is_whitespace() || "、，,;；。".contains(c));
    let inner = trimmed.strip_prefix('（').and_then(|s| s.strip_suffix('）'));
    inner.unwrap_or(trimmed).trim().to_string()
}

/// Extracts `[(target_id, note)]` outside parentheses from a declaration line.
/// @outputs: 1
pub fn extract_decision_targets(rest: &str) -> Vec<(String, String)> {
    let found = find_top_level_d_indices(rest);
    let mut out = Vec::with_capacity(found.len());
    for (n, &start) in found.iter().enumerate() {
        let end = found.get(n + 1).copied().unwrap_or(rest.len());
        let id = rest[start..start + 6].to_string();
        let note = clean_target_note(&rest[start + 6..end]);
        out.push((id, note));
    }
    out
}

/// Collects all `D-NNNN` headers declared as `## D-NNNN ...` in DECISIONS.md.
/// @outputs: 1
pub fn extract_known_decisions(decisions_md: &str) -> BTreeSet<String> {
    let mut set = BTreeSet::new();
    for line in decisions_md.lines() {
        let rest = match line.strip_prefix("## ") {
            Some(r) => r,
            None => continue,
        };
        if is_d_id_at(rest) {
            set.insert(rest[..6].to_string());
        }
    }
    set
}

/// Extracts `- **取代**: ...` / `- **更正**: ...` declaration from a single bullet line.
/// @outputs: 2
fn parse_decl_line(line: &str) -> Option<(&'static str, &str)> {
    for (prefix, kind) in [
        ("- **取代**:", "取代"),
        ("- **取代**：", "取代"),
        ("- **更正**:", "更正"),
        ("- **更正**：", "更正"),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some((kind, rest.trim()));
        }
    }
    None
}

/// Parses a backfill line `- **補登**：D-XXXX ← 取代|更正 D-YYYY（note）`.
/// @outputs: 6
fn parse_backfill_line(line: &str) -> Option<DecisionRelation> {
    let rest = line
        .strip_prefix("- **補登**：")
        .or_else(|| line.strip_prefix("- **補登**:"))?
        .trim();
    if !is_d_id_at(rest) {
        return None;
    }
    let target = rest[..6].to_string();
    let after_arrow = rest.split_once('←')?.1.trim();
    let (kind, tail) = if let Some(s) = after_arrow.strip_prefix("取代") {
        ("取代", s.trim())
    } else if let Some(s) = after_arrow.strip_prefix("更正") {
        ("更正", s.trim())
    } else {
        return None;
    };
    if !is_d_id_at(tail) {
        return None;
    }
    Some(DecisionRelation {
        target,
        kind: kind.to_string(),
        source: tail[..6].to_string(),
        note: clean_target_note(&tail[6..]),
    })
}

/// Validates chronological monotonicity and existence of relation endpoints.
/// @outputs: 4
fn validate_relation(rel: &DecisionRelation, known: &BTreeSet<String>) -> Option<String> {
    if !known.contains(&rel.target) {
        return Some(format!("{} 宣告{} {}，但 {} 不存在", rel.source, rel.kind, rel.target, rel.target));
    }
    if !known.contains(&rel.source) {
        return Some(format!("補登的宣告者 {} 不存在", rel.source));
    }
    let src_n = parse_d_num(&rel.source).unwrap_or(0);
    let tgt_n = parse_d_num(&rel.target).unwrap_or(0);
    if src_n <= tgt_n {
        return Some(format!("{} {} {}：只能由較新的條目取代或更正較舊的條目", rel.source, rel.kind, rel.target));
    }
    None
}

/// Collects relations emitted from a single markdown bullet line within `cur_src`.
/// @outputs: 4
fn collect_line_relations(trimmed: &str, cur_src: &str, rels: &mut BTreeSet<DecisionRelation>) {
    if let Some(bf) = parse_backfill_line(trimmed) {
        rels.insert(bf);
    }
    if cur_src.is_empty() {
        return;
    }
    let (kind, rest) = match parse_decl_line(trimmed) {
        Some(pair) => pair,
        None => return,
    };
    for (tgt, note) in extract_decision_targets(rest) {
        rels.insert(DecisionRelation { target: tgt, kind: kind.to_string(), source: cur_src.to_string(), note });
    }
}

/// Parses all replacement/correction relations in `DECISIONS.md` and validates invariants.
/// @outputs: 2
pub fn parse_decision_relations(decisions_md: &str) -> Result<Vec<DecisionRelation>, Vec<String>> {
    let known = extract_known_decisions(decisions_md);
    let mut cur_src = String::new();
    let mut rels: BTreeSet<DecisionRelation> = BTreeSet::new();
    for line in decisions_md.lines() {
        let trimmed = line.trim();
        if let Some(h) = trimmed.strip_prefix("## ").filter(|s| is_d_id_at(s)) {
            cur_src = h[..6].to_string();
            continue;
        }
        collect_line_relations(trimmed, &cur_src, &mut rels);
    }
    let vec_rels: Vec<DecisionRelation> = rels.into_iter().collect();
    let probs: Vec<String> = vec_rels.iter().filter_map(|r| validate_relation(r, &known)).collect();
    if probs.is_empty() { Ok(vec_rels) } else { Err(probs) }
}

/// Formats `DECISIONS-STATUS.md` identically to `decisions-status.py --write`.
/// @outputs: 1
pub fn render_decisions_status_md(rels: &[DecisionRelation]) -> String {
    let mut sorted = rels.to_vec();
    sorted.sort_by(|a, b| a.target.cmp(&b.target).then(a.source.cmp(&b.source)));
    let mut rows: Vec<String> = Vec::with_capacity(sorted.len());
    for r in &sorted {
        let is_partial = r.note.starts_with("部分") || (r.note.contains('第') && r.note.contains('項'));
        let prefix = if is_partial { "部分" } else { "" };
        let note_str = if r.note.is_empty() { "—" } else { r.note.as_str() };
        rows.push(format!("| {} | {}{} | {} | {} |", r.target, prefix, r.kind, r.source, note_str));
    }
    format!(
        "# 決策狀態索引（自動產生，勿手改）\n\n\
         > 由 `scripts/decisions-status.py --write` 依 DECISIONS.md 的「取代／更正／補登」宣告產生（R10-10，D-0076）。\n\
         > 史實層不改；讀到下列條目時，**以「見」那一筆為準**。\n\n\
         | 條目 | 狀態 | 見 | 說明 |\n|---|---|---|---|\n{}\n",
        rows.join("\n")
    )
}

/// Checks if raw row starts with `L-NNN` (3 digits).
/// @outputs: 2
fn is_l_id(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 5 && b[0] == b'L' && b[1] == b'-' && b[2..5].iter().all(|c| c.is_ascii_digit())
}

/// Parses a single markdown table row `| L-NNN | lesson | anchor | first | repeat |`.
/// @outputs: 5
fn parse_lesson_row_line(trimmed: &str) -> Option<LessonRow> {
    let inner = trimmed.strip_prefix('|')?.strip_suffix('|')?;
    let cells: Vec<&str> = inner.split('|').collect();
    if cells.len() != 5 {
        return None;
    }
    let lid = cells[0].trim();
    if !is_l_id(lid) {
        return None;
    }
    Some(LessonRow {
        id: lid.to_string(),
        lesson: cells[1].trim().to_string(),
        anchor: cells[2].trim().trim_matches('`').to_string(),
        first_ref: cells[3].trim().to_string(),
        repeat_ref: cells[4].trim().to_string(),
    })
}

/// Checks if a table line looks like a lesson entry (`| L-...`).
/// @outputs: 2
fn looks_like_lesson_row(trimmed: &str) -> bool {
    let after_pipe = match trimmed.strip_prefix('|') {
        Some(s) => s.trim_start(),
        None => return false,
    };
    after_pipe.starts_with("L-") && after_pipe.as_bytes().get(2).is_some_and(|b| b.is_ascii_digit())
}

/// Checks that all `D-NNNN` references inside a lesson row exist in `known_decisions`.
/// @outputs: 2
fn check_lesson_d_refs(row: &LessonRow, known: &BTreeSet<String>, errors: &mut Vec<String>) {
    let combined = format!("{} {}", row.first_ref, row.repeat_ref);
    for idx in find_top_level_d_indices(&combined) {
        let did = &combined[idx..idx + 6];
        if !known.contains(did) {
            errors.push(format!("{}：引用的 {} 不存在", row.id, did));
        }
    }
}

/// Parses `LESSONS.md` table rows, detects malformed rows, and verifies all `D-NNNN` refs exist.
/// @outputs: 2
pub fn parse_lessons_rows(lessons_md: &str, known_decisions: &BTreeSet<String>) -> Result<Vec<LessonRow>, Vec<String>> {
    let mut rows = Vec::new();
    let mut errors = Vec::new();
    for line in lessons_md.lines() {
        let trimmed = line.trim();
        if !looks_like_lesson_row(trimmed) {
            continue;
        }
        if let Some(row) = parse_lesson_row_line(trimmed) {
            check_lesson_d_refs(&row, known_decisions, &mut errors);
            rows.push(row);
        } else {
            errors.push(format!("格式不合的教訓列（不會被檢查）：{}", trimmed.chars().take(60).collect::<String>()));
        }
    }
    if rows.is_empty() && errors.is_empty() {
        errors.push("context/LESSONS.md 沒有任何教訓列".to_string());
    }
    if errors.is_empty() { Ok(rows) } else { Err(errors) }
}
