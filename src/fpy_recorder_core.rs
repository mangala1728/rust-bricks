// SPDX-License-Identifier: MIT
// ## Map
// - Role: core/fpy_recorder_core — Zero-dependency pure Rust FPY Recorder engine (FPY-RECORDER-SPEC v1.0)
// - Inputs: Tool commands, exit codes, prompts, turn-end messages, JSONL chain lines, proposal trial metrics
// - Outputs: SHA-256 hash16, redacted UTF-16 clipped summaries, N1/N1b/N4/N6 flags, K3+K4 audit, P3/P22/Q5 transitions
// - Invariants: Zero external crates, zero recursion (R09-7), nesting <= 2 (R09-3), JPL <= 60 lines/fn, zero unproved unwrap

use crate::crypto_vault::sha256_hex;
use crate::stat_gate_core::stat_bayesian_trial_upgrade;

const MAX_SCAN_ITERS: usize = 4096;

/// Computes first 16 lowercase hex chars of SHA-256(UTF-8 bytes) per FPY P5.
/// @outputs: 1
pub fn fpy_hash16(input: &str) -> String {
    let hex = sha256_hex(input.as_bytes());
    hex.chars().take(16).collect()
}

/// Checks if byte before token start is a non-alphanumeric boundary (or start of string).
/// @outputs: 2
/// @domain: prefix_pos [0..1000000]
fn has_word_boundary_before(full: &str, prefix_pos: usize) -> bool {
    if prefix_pos == 0 {
        return true;
    }
    if prefix_pos > 1_000_000 || prefix_pos >= full.len() {
        return false;
    }
    let prev = full.as_bytes()[prefix_pos - 1];
    !prev.is_ascii_alphanumeric() && prev != b'_'
}

/// Checks if byte slice starts with valid JWT (`eyJ... . ... . ...`) with 2 dots.
/// @outputs: 2
fn is_valid_jwt_body(after_eyj: &str) -> bool {
    let tok_len = after_eyj.bytes().take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')).count();
    let tok = &after_eyj[..tok_len];
    tok.bytes().filter(|&b| b == b'.').count() == 2 && tok_len >= 20
}

/// Helper masking one prefixed token occurrence with boundary and structure checks.
/// @outputs: 2
/// @domain: min_body [4..256]
fn apply_token_mask_step<'a>(
    full: &'a str,
    pos: usize,
    prefix: &str,
    min_body: usize,
    label: &'static str,
    kinds: &mut Vec<&'static str>,
    out: &mut String,
) -> &'a str {
    let after = &full[pos + prefix.len()..];
    if min_body < 4 || min_body > 256 {
        out.push_str(prefix);
        return after;
    }
    let body_len = after.bytes().take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')).count();
    let need_boundary = matches!(prefix, "sk-" | "eyJ");
    let boundary_ok = !need_boundary || has_word_boundary_before(full, pos);
    let jwt_ok = prefix != "eyJ" || is_valid_jwt_body(after);
    let is_dummy_aiza = prefix == "AIza" && (after.starts_with("TEST") || after.starts_with("Dummy") || after.starts_with("SyDummy"));
    if body_len >= min_body && boundary_ok && jwt_ok && !is_dummy_aiza {
        out.push_str("[REDACTED:");
        out.push_str(label);
        out.push(']');
        if !kinds.contains(&label) {
            kinds.push(label);
        }
        return &after[body_len..];
    }
    out.push_str(prefix);
    after
}

/// Helper replacing prefixed tokens with bounded iterations (R09-8).
/// @outputs: 2
/// @domain: min_body [4..256]
fn mask_prefixed_token(
    text: &str,
    prefix: &str,
    min_body: usize,
    label: &'static str,
    kinds: &mut Vec<&'static str>,
) -> String {
    if min_body < 4 || min_body > 256 {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut steps = 0usize;
    while steps < MAX_SCAN_ITERS {
        let Some(pos) = rest.find(prefix) else { break };
        out.push_str(&rest[..pos]);
        rest = apply_token_mask_step(rest, pos, prefix, min_body, label, kinds, &mut out);
        steps += 1;
    }
    out.push_str(rest);
    out
}

/// Helper masking one Bearer token occurrence cleanly.
/// @outputs: 2
/// @domain: pos [0..1000000]
fn apply_bearer_step<'a>(rest: &'a str, pos: usize, kinds: &mut Vec<&'static str>, out: &mut String) -> &'a str {
    if pos > 1_000_000 || pos + 6 > rest.len() {
        return rest;
    }
    let after_kw = &rest[pos + 6..];
    let ws_len = after_kw.bytes().take_while(|&b| b == b' ' || b == b'\t').count();
    let tok_slice = &after_kw[ws_len..];
    let tok_len = tok_slice.bytes().take_while(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'.')).count();
    if ws_len >= 1 && tok_len >= 12 {
        out.push_str("[REDACTED:bearer-token]");
        if !kinds.contains(&"bearer-token") {
            kinds.push("bearer-token");
        }
        return &tok_slice[tok_len..];
    }
    out.push_str(&rest[pos..pos + 6]);
    after_kw
}

/// Masks Bearer tokens case-insensitively across space or tab separators.
/// @outputs: 1
fn mask_bearer_tokens(text: &str, kinds: &mut Vec<&'static str>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut steps = 0usize;
    while steps < MAX_SCAN_ITERS {
        let lower = rest.to_ascii_lowercase();
        let Some(pos) = lower.find("bearer") else { break };
        out.push_str(&rest[..pos]);
        rest = apply_bearer_step(rest, pos, kinds, &mut out);
        steps += 1;
    }
    out.push_str(rest);
    out
}

/// Masks PEM private key blocks (and truncated BEGIN blocks without END) safely without self-matching BEGIN.
/// @outputs: 1
fn mask_structural_secrets(text: &str, kinds: &mut Vec<&'static str>) -> String {
    let pem_begin = concat!("-----", "BEGIN ");
    let pem_end = concat!("-----", "END ");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut steps = 0usize;
    while steps < 64 {
        let Some(s) = rest.find(pem_begin) else { break };
        out.push_str(&rest[..s]);
        let after_begin = &rest[s + pem_begin.len()..];
        if !kinds.contains(&"private-key-block") {
            kinds.push("private-key-block");
        }
        let Some(e_rel) = after_begin.find(pem_end) else {
            out.push_str("[REDACTED:private-key-truncated]");
            return out;
        };
        let tail_after_end = &after_begin[e_rel + pem_end.len()..];
        let close_dashes = tail_after_end.find("-----").map(|d| d + 5).unwrap_or(tail_after_end.len());
        out.push_str("[REDACTED:private-key-block]");
        rest = &tail_after_end[close_dashes..];
        steps += 1;
    }
    out.push_str(rest);
    out
}

/// Helper masking userinfo inside one URI authority segment (only before `/`, `?`, `#`).
/// @outputs: 1
fn mask_one_uri_seg(seg: &str, kinds: &mut Vec<&'static str>, out: &mut String) {
    let auth_end = seg.find(['/', '?', '#']).unwrap_or(seg.len());
    let auth = &seg[..auth_end];
    let Some(at) = auth.find('@') else {
        out.push_str(seg);
        return;
    };
    if !auth[..at].contains(':') {
        out.push_str(seg);
        return;
    }
    out.push_str("[REDACTED:uri-cred]@");
    out.push_str(&seg[at + 1..]);
    if !kinds.contains(&"uri-cred") {
        kinds.push("uri-cred");
    }
}

/// Masks URI credentials (`://user:pass@host`) with bounded iterations (R09-8).
/// @outputs: 1
fn mask_uri_credentials(text: &str, kinds: &mut Vec<&'static str>) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut steps = 0usize;
    while steps < MAX_SCAN_ITERS {
        let Some(scheme) = rest.find("://") else { break };
        out.push_str(&rest[..scheme + 3]);
        let tail = &rest[scheme + 3..];
        let seg_end = tail.find(|c: char| c.is_whitespace() || c == '"' || c == '\'').unwrap_or(tail.len());
        mask_one_uri_seg(&tail[..seg_end], kinds, &mut out);
        rest = &tail[seg_end..];
        steps += 1;
    }
    out.push_str(rest);
    out
}

/// Redacts all built-in secret patterns (matching secret-redactor) and returns (redacted, kinds).
/// @outputs: 1
pub fn fpy_redact_secrets(text: &str) -> (String, Vec<&'static str>) {
    let mut kinds = Vec::new();
    let mut s = mask_structural_secrets(text, &mut kinds);
    s = mask_uri_credentials(&s, &mut kinds);
    s = mask_bearer_tokens(&s, &mut kinds);
    let rules: &[(&str, usize, &str)] = &[
        ("AKIA", 16, "aws-access-key"),
        ("ASIA", 16, "aws-access-key"),
        ("sk-ant-api", 20, "anthropic-api-key"),
        ("sk-", 20, "openai-api-key"),
        ("ghp_", 36, "github-token"),
        ("gho_", 36, "github-token"),
        ("ghu_", 36, "github-token"),
        ("ghs_", 36, "github-token"),
        ("ghr_", 36, "github-token"),
        ("github_pat_", 22, "github-token"),
        ("AIza", 35, "google-api-key"),
        ("sk_live_", 24, "stripe-key"),
        ("sk_test_", 24, "stripe-key"),
        ("rk_live_", 24, "stripe-key"),
        ("rk_test_", 24, "stripe-key"),
        ("xoxb-", 10, "slack-token"),
        ("xoxp-", 10, "slack-token"),
        ("xoxa-", 10, "slack-token"),
        ("xoxr-", 10, "slack-token"),
        ("eyJ", 20, "jwt"),
    ];
    for &(pfx, min_len, lbl) in rules {
        s = mask_prefixed_token(&s, pfx, min_len, lbl, &mut kinds);
    }
    (s, kinds)
}

/// Clips text to `max_units` UTF-16 code units (including suffix `（已截斷）`), never splitting surrogate pairs.
/// @outputs: 3
/// @domain: max_units [10..4096]
pub fn fpy_clip_utf16(text: &str, max_units: usize) -> (String, bool) {
    if max_units < 10 || max_units > 4096 {
        return (String::new(), false);
    }
    let total_u16: usize = text.chars().map(|c| c.len_utf16()).sum();
    if total_u16 <= max_units {
        return (text.to_string(), false);
    }
    const SUFFIX: &str = "（已截斷）";
    let budget = max_units.saturating_sub(5);
    let mut used = 0usize;
    let mut out = String::new();
    for ch in text.chars() {
        let w = ch.len_utf16();
        if used + w > budget {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push_str(SUFFIX);
    (out, true)
}

/// Normalizes Claude Code and Antigravity tool names into canonical `tool_class`.
/// @outputs: 5
pub fn fpy_normalize_tool_class(tool: &str) -> &'static str {
    match tool.trim() {
        "Bash" | "PowerShell" | "run_command" => "shell",
        "Write" | "NotebookEdit" | "write_file" | "write_to_file" => "write",
        "Edit" | "MultiEdit" | "edit_file" | "replace_file_content" => "edit",
        "Read" | "read_file" | "view_file" => "read",
        _ => "other",
    }
}

/// Extracts last non-empty shell segment outside single/double quotes (splitting on `;`, `|`, `&&`, `\n`).
/// @outputs: 1
fn extract_last_segment(cmd: &str) -> &str {
    let trimmed = cmd.trim();
    let bytes = trimmed.as_bytes();
    let limit = bytes.len().min(65_536);
    let mut last_start = 0usize;
    let mut in_sq = false;
    let mut in_dq = false;
    let mut i = 0usize;
    while i < limit {
        let b = bytes[i];
        if b == b'\'' && !in_dq {
            in_sq = !in_sq;
        } else if b == b'"' && !in_sq {
            in_dq = !in_dq;
        } else if !in_sq && !in_dq && matches!(b, b';' | b'|' | b'\n' | b'\r') {
            last_start = i + 1;
        } else if !in_sq && !in_dq && b == b'&' && i + 1 < limit && bytes[i + 1] == b'&' {
            last_start = i + 2;
            i += 1;
        }
        i += 1;
    }
    trimmed[last_start.min(trimmed.len())..].trim()
}

/// Checks if the last command segment matches N1 exit-1 whitelist.
/// @outputs: 2
fn is_n1_whitelisted_exit1(last_seg: &str) -> bool {
    let first = last_seg.split_whitespace().next().unwrap_or("");
    let base = first.rsplit(['/', '\\']).next().unwrap_or(first);
    if matches!(base, "grep" | "rg" | "findstr" | "Select-String" | "test" | "[") {
        return true;
    }
    last_seg.starts_with("git diff ") && (last_seg.contains("--quiet") || last_seg.contains("--exit-code"))
}

/// Strips trailing `# ...` shell comment from a segment.
/// @outputs: 1
fn strip_trailing_comment(seg: &str) -> &str {
    match seg.find(" #") {
        Some(pos) => seg[..pos].trim(),
        None => seg.trim(),
    }
}

/// Detects N1b-1..N1b-13 exit-code masking patterns (including comment/paren/whitespace/case variants).
/// @outputs: 2
fn has_n1b_masking(trimmed_cmd: &str, last_seg: &str) -> bool {
    let seg_no_comment = strip_trailing_comment(last_seg).to_ascii_lowercase();
    let stripped_paren = seg_no_comment.trim_matches(|c| c == '(' || c == ')').trim();
    if matches!(stripped_paren, "true" | ":" | "exit 0" | "out-null") {
        return true;
    }
    let lower_full = strip_trailing_comment(trimmed_cmd).to_ascii_lowercase();
    if lower_full.contains("|| true")
        || lower_full.contains("|| :")
        || lower_full.contains("|| exit 0")
        || lower_full.contains("|| (exit 0)")
        || lower_full.contains("|| echo ")
        || lower_full.contains("|| echo\t")
        || lower_full.contains("| out-null")
    {
        return true;
    }
    lower_full.contains("-erroraction silentlycontinue")
        || lower_full.contains("-ea silentlycontinue")
        || lower_full.contains("-erroraction ignore")
        || lower_full.contains("2>$null")
        || lower_full.contains("2> $null")
}

/// Detects shell file-write patterns (spaced/unspaced redirects and case-insensitive PowerShell cmdlets).
/// @outputs: 2
fn has_shell_write(cmd: &str) -> bool {
    let lower = cmd
        .to_ascii_lowercase()
        .replace("2>$null", "")
        .replace("2> $null", "")
        .replace("2>/dev/null", "")
        .replace("2>&1", "")
        .replace("->", "")
        .replace("=>", "");
    if lower.contains("set-content") || lower.contains("out-file") || lower.contains("add-content") || lower.contains("sed -i") {
        return true;
    }
    if lower.contains("|tee ") || lower.contains("| tee ") || lower.contains(">>") {
        return true;
    }
    lower.contains('>')
}

/// Evaluates N1 (tool failure), N1b (masked exit), and shell_write on a shell command.
/// @outputs: 4
pub fn fpy_detect_n1_and_n1b(cmd: &str, exit_code: Option<i32>, err: bool, deny: bool) -> (bool, bool, bool) {
    let trimmed = cmd.trim();
    let last_seg = extract_last_segment(trimmed);
    let masked = has_n1b_masking(trimmed, last_seg);
    let sw = has_shell_write(trimmed);
    if deny {
        return (false, masked, sw);
    }
    let failed = err || matches!(exit_code, Some(c) if c != 0);
    if !failed {
        return (false, masked, sw);
    }
    if exit_code == Some(1) && is_n1_whitelisted_exit1(last_seg) {
        return (false, masked, sw);
    }
    (true, masked, sw)
}

/// Checks if an ASCII keyword appears as a standalone word (bounded by non-ASCII-alphanumeric chars).
/// @outputs: 2
fn contains_ascii_word(lower: &str, word: &str) -> bool {
    let bytes = lower.as_bytes();
    let mut rest = lower;
    let mut offset = 0usize;
    while let Some(pos) = rest.find(word) {
        let abs = offset + pos;
        let left_ok = abs == 0 || !bytes[abs - 1].is_ascii_alphanumeric();
        let right = abs + word.len();
        let right_ok = right >= bytes.len() || !bytes[right].is_ascii_alphanumeric();
        if left_ok && right_ok {
            return true;
        }
        offset = abs + word.len();
        rest = &lower[offset..];
    }
    false
}

/// Evaluates N4 user-correction signal on first 40 Unicode code points (with A14 sdk/bridge handling).
/// @outputs: 5
pub fn fpy_detect_n4_prompt(
    prompt: &str,
    source: &str,
    is_bench: bool,
    interrupt: bool,
    prev_user_deny: bool,
) -> (&'static str, Vec<&'static str>, bool) {
    if is_bench && source == "sdk" {
        return ("none", Vec::new(), false);
    }
    let head: String = prompt.trim().chars().take(40).collect();
    let lower = head.to_ascii_lowercase();
    let mut hits = Vec::new();
    for &(kw, code) in &[("不是", "not-this"), ("不對", "wrong-zh"), ("重做", "redo-zh"), ("搞錯", "mistake-zh"), ("錯了", "error-zh"), ("不要", "dont-zh")] {
        if lower.contains(kw) {
            hits.push(code);
        }
    }
    for &(kw, code) in &[("stop", "stop-en"), ("wrong", "wrong-en"), ("undo", "undo-en")] {
        if contains_ascii_word(&lower, kw) {
            hits.push(code);
        }
    }
    let has_q = head.contains('?') || head.contains('？') || head.contains('嗎') || head.contains('呢');
    if !interrupt && !prev_user_deny && hits.is_empty() {
        return ("none", hits, has_q);
    }
    if source == "bridge" {
        return ("n4_bridge", hits, has_q);
    }
    if !hits.is_empty() && has_q && !interrupt && !prev_user_deny {
        return ("n4_weak", hits, has_q);
    }
    ("N4", hits, has_q)
}

/// Checks one trimmed line for an explicit hedge prefix with non-empty content.
/// @outputs: 2
fn match_hedge_line(trimmed: &str) -> Option<&'static str> {
    for &(pfx, code) in &[("未驗證：", "unverified"), ("未完成：", "incomplete"), ("未解決：", "unresolved"), ("未完成:", "incomplete"), ("未解決:", "unresolved")] {
        let Some(rest) = trimmed.strip_prefix(pfx) else { continue };
        if !rest.trim().is_empty() {
            return Some(code);
        }
    }
    None
}

/// Detects N6 completion keywords (avoiding `未完成`/`abandoned`) and explicit hedge lines.
/// @outputs: 2
pub fn fpy_detect_n6_turn_end(msg: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    let scrubbed = msg.replace("未完成", "").replace("未解決", "").replace("未驗證", "");
    let lower = scrubbed.to_ascii_lowercase();
    let mut done_hits = Vec::new();
    for &(kw, code) in &[("已完成", "completed-zh"), ("全部通過", "all-pass-zh"), ("完成", "done-zh")] {
        if lower.contains(kw) {
            done_hits.push(code);
        }
    }
    if contains_ascii_word(&lower, "done") {
        done_hits.push("done-en");
    }
    if lower.contains("all tests pass") {
        done_hits.push("all-pass-en");
    }
    let mut hedge_hits = Vec::new();
    for raw in msg.lines().take(2000) {
        let Some(code) = match_hedge_line(raw.trim()) else { continue };
        if !hedge_hits.contains(&code) {
            hedge_hits.push(code);
        }
    }
    (done_hits, hedge_hits)
}

/// Extracts `"prev":"..."` string value from one JSONL line.
/// @outputs: 2
fn extract_prev_field(line: &str) -> Option<&str> {
    let idx = line.find("\"prev\":\"")?;
    let rest = &line[idx + 8..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Extracts `"seq":N` unsigned integer from one JSONL line (K3).
/// @outputs: 2
fn extract_seq_field(line: &str) -> Option<usize> {
    let idx = line.find("\"seq\":")?;
    let rest = &line[idx + 6..];
    let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits_len == 0 {
        return None;
    }
    rest[..digits_len].parse::<usize>().ok()
}

/// Verifies FPY K3 (`seq` continuity) and K4 (`prev` hash-chain), returning (total, broken_count).
/// @outputs: 2
pub fn fpy_verify_hash_chain(lines: &[&str]) -> (usize, usize) {
    let mut broken = 0usize;
    let mut prev_line: Option<&str> = None;
    let mut total = 0usize;
    let mut expected_seq = 1usize;
    for &raw in lines.iter().take(100_000) {
        let line = raw.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            continue;
        }
        total += 1;
        let expected_prev = match prev_line {
            None => "0".to_string(),
            Some(pl) => fpy_hash16(pl),
        };
        let prev_ok = extract_prev_field(line) == Some(expected_prev.as_str());
        let seq_ok = extract_seq_field(line) == Some(expected_seq);
        if !prev_ok || !seq_ok {
            broken += 1;
        }
        prev_line = Some(line);
        expected_seq += 1;
    }
    (total, broken)
}

/// Evaluates FPY §6.3 proposal state transitions under unified P3/P22/Q5 Bayesian rule & G2/K1 guards.
/// @outputs: 8
/// @domain: successes [0..100000]
/// @domain: failures [0..100000]
/// @domain: distinct_tasks [0..100000]
/// @domain: distinct_sessions [0..100000]
/// @domain: days_in_trial [0..10000]
/// @domain: pending_suggested [0..100000]
pub fn fpy_evaluate_proposal_transition(
    state: &str,
    successes: u32,
    failures: u32,
    distinct_tasks: u32,
    distinct_sessions: u32,
    days_in_trial: u32,
    pending_suggested: u32,
    is_goodhart_relax: bool,
) -> (&'static str, bool) {
    if successes > 100_000
        || failures > 100_000
        || distinct_tasks > 100_000
        || distinct_sessions > 100_000
        || days_in_trial > 10_000
        || pending_suggested > 100_000
    {
        return ("invalid", false);
    }
    if state == "retired" || state == "rejected" {
        return (if state == "retired" { "retired" } else { "rejected" }, false);
    }
    if is_goodhart_relax {
        return ("rejected", false);
    }
    if state == "trusted" {
        return if failures >= 1 { ("provisional", false) } else { ("trusted", true) };
    }
    if failures >= 2 {
        return ("retired", false);
    }
    let bayes_ok = stat_bayesian_trial_upgrade(successes, failures, 0.8, 0.9).map(|r| r.0).unwrap_or(false);
    if bayes_ok && distinct_tasks >= 3 && distinct_sessions >= 2 && pending_suggested == 0 {
        return ("trusted", true);
    }
    if days_in_trial > 30 {
        return ("provisional_overdue", false);
    }
    ("provisional", false)
}
