// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Zero-dependency Kotlin lexical stripper, nesting counter, guard matcher & R09 audit brick (`core` brick)
//! - Inputs: Kotlin source text (`&str`), relative path (`&str`), `max_outputs`, `max_nest`
//! - Outputs: Cleaned lexical string and `Vec<KtLintIssue>`
//! - Invariants:
//!   1. Zero external crates (`std` only) and zero `static mut` (`R09-2`).
//!   2. Zero recursion (`R09-7`), bounded token scans (`R09-8`), nesting depth <= 2 (`R09-3`).
//!   3. Explicit `@domain` bounds and runtime guards on all numeric parameters (`R09-4`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KtLintIssue {
    pub line: usize,
    pub rule: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Normal,
    LineComment,
    BlockComment,
    StringLit,
    RawString,
    CharLit,
}

/// Consumes a single normal-mode Kotlin character and transitions `state`, pushing `' '` for stripped content.
// @outputs: 7
// @domain: idx [0..16777216]
fn step_lex_normal(bytes: &[u8], idx: usize, state: &mut LexState, out: &mut Vec<u8>) -> usize {
    if idx >= bytes.len() || idx > 16_777_216 {
        return bytes.len();
    }
    if bytes[idx..].starts_with(b"//") {
        *state = LexState::LineComment;
        out.extend_from_slice(b"  ");
        return idx + 2;
    }
    if bytes[idx..].starts_with(b"/*") {
        *state = LexState::BlockComment;
        out.extend_from_slice(b"  ");
        return idx + 2;
    }
    if bytes[idx..].starts_with(b"\"\"\"") {
        *state = LexState::RawString;
        out.extend_from_slice(b"   ");
        return idx + 3;
    }
    if bytes[idx] == b'"' {
        *state = LexState::StringLit;
        out.push(b' ');
        return idx + 1;
    }
    if bytes[idx] == b'\'' {
        *state = LexState::CharLit;
        out.push(b' ');
        return idx + 1;
    }
    out.push(bytes[idx]);
    idx + 1
}

/// Consumes a single non-normal Kotlin lexical state character while preserving newlines (`1:1` byte/line alignment).
// @outputs: 7
// @domain: idx [0..16777216]
fn step_lex_masked(bytes: &[u8], idx: usize, state: &mut LexState, out: &mut Vec<u8>) -> usize {
    if idx >= bytes.len() || idx > 16_777_216 {
        return bytes.len();
    }
    let b = bytes[idx];
    if *state == LexState::LineComment {
        if b == b'\n' {
            *state = LexState::Normal;
            out.push(b'\n');
        } else {
            out.push(b' ');
        }
        return idx + 1;
    }
    if *state == LexState::BlockComment && bytes[idx..].starts_with(b"*/") {
        *state = LexState::Normal;
        out.extend_from_slice(b"  ");
        return idx + 2;
    }
    if *state == LexState::RawString && bytes[idx..].starts_with(b"\"\"\"") {
        *state = LexState::Normal;
        out.extend_from_slice(b"   ");
        return idx + 3;
    }
    if matches!(*state, LexState::StringLit | LexState::CharLit) && b == b'\\' && idx + 1 < bytes.len() {
        out.push(b' ');
        out.push(if bytes[idx + 1] == b'\n' { b'\n' } else { b' ' });
        return idx + 2;
    }
    if (*state == LexState::StringLit && b == b'"') || (*state == LexState::CharLit && b == b'\'') {
        *state = LexState::Normal;
        out.push(b' ');
        return idx + 1;
    }
    out.push(if b == b'\n' { b'\n' } else { b' ' });
    idx + 1
}

/// Strips Kotlin comments and string/char literals while preserving exact byte offsets and line numbers.
// @outputs: 1
pub fn strip_kotlin_lexical(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut state = LexState::Normal;
    let mut i = 0usize;
    while i < bytes.len() {
        if state == LexState::Normal {
            i = step_lex_normal(bytes, i, &mut state, &mut out);
        } else {
            i = step_lex_masked(bytes, i, &mut state, &mut out);
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Checks if byte offset `pos` starts a whole keyword `kw` inside `code`.
// @outputs: 2
// @domain: pos [0..16777216]
fn is_kw_at(code: &[u8], pos: usize, kw: &[u8]) -> bool {
    if pos > 16_777_216 || pos + kw.len() > code.len() || &code[pos..pos + kw.len()] != kw {
        return false;
    }
    let prev_ok = pos == 0 || !(code[pos - 1].is_ascii_alphanumeric() || code[pos - 1] == b'_');
    let next_pos = pos + kw.len();
    let next_ok = next_pos >= code.len() || !(code[next_pos].is_ascii_alphanumeric() || code[next_pos] == b'_');
    prev_ok && next_ok
}

/// Computes maximum control-flow nesting depth (`if`/`when`/`for`/`while`) inside a Kotlin function body.
// @outputs: 1
pub fn kt_max_nesting(body: &str) -> usize {
    let b = body.as_bytes();
    let mut depth = 0usize;
    let mut maxd = 0usize;
    let mut opens: Vec<bool> = Vec::new();
    for i in 0..b.len() {
        if b[i] == b'}' {
            let popped = opens.pop().unwrap_or(false);
            depth = depth.saturating_sub(usize::from(popped));
            continue;
        }
        if b[i] != b'{' {
            continue;
        }
        let win_start = i.saturating_sub(100);
        let is_ctrl = has_ctrl_before_brace(&b[win_start..i]);
        opens.push(is_ctrl);
        if is_ctrl {
            depth += 1;
            maxd = maxd.max(depth);
        }
    }
    maxd
}

/// Returns end index if `window[pos..]` starts a Kotlin control keyword (`if`/`when`/`for`/`while`).
// @outputs: 2
// @domain: pos [0..1024]
fn match_ctrl_kw_end(window: &[u8], pos: usize) -> Option<usize> {
    if pos > 1024 || pos >= window.len() {
        return None;
    }
    let kws: [&[u8]; 4] = [b"if", b"when", b"for", b"while"];
    for kw in kws {
        if is_kw_at(window, pos, kw) {
            return Some(pos + kw.len());
        }
    }
    None
}

/// Checks whether a control keyword (`if`/`when`/`for`/`while`) appears cleanly before `{` without `;` or `{`.
// @outputs: 2
fn has_ctrl_before_brace(window: &[u8]) -> bool {
    let mut last_ctrl: Option<usize> = None;
    for i in 0..window.len() {
        if let Some(end) = match_ctrl_kw_end(window, i) {
            last_ctrl = Some(end);
        }
    }
    let Some(pos) = last_ctrl else {
        return false;
    };
    !window[pos..].iter().any(|&c| c == b'{' || c == b';')
}

/// Checks if `body` contains a single-block runtime guard mentioning `param` with `return`/`throw`/`Result.failure`.
// @outputs: 2
pub fn kt_has_guard(body: &str, param: &str) -> bool {
    let bytes = body.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if !is_kw_at(bytes, i, b"if") {
            i += 1;
            continue;
        }
        let Some(open) = bytes[i..].iter().position(|&c| c == b'{').map(|p| i + p) else {
            break;
        };
        let Some(close) = bytes[open + 1..].iter().position(|&c| c == b'}').map(|p| open + 1 + p) else {
            break;
        };
        let cond = &body[i + 2..open];
        let cons = &body[open + 1..close];
        if contains_word(cond, param) && (contains_word(cons, "return") || contains_word(cons, "throw") || cons.contains("Result.failure")) {
            return true;
        }
        i = open + 1;
    }
    false
}

/// Checks if `haystack` contains `word` as a whole identifier.
// @outputs: 2
pub fn contains_word(haystack: &str, word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    let mut offset = 0usize;
    while let Some(pos) = haystack[offset..].find(word) {
        let abs = offset + pos;
        let prev_ok = abs == 0 || !(haystack.as_bytes()[abs - 1].is_ascii_alphanumeric() || haystack.as_bytes()[abs - 1] == b'_');
        let end = abs + word.len();
        let next_ok = end >= haystack.len() || !(haystack.as_bytes()[end].is_ascii_alphanumeric() || haystack.as_bytes()[end] == b'_');
        if prev_ok && next_ok {
            return true;
        }
        offset = abs + 1;
    }
    false
}

/// Audits a single Kotlin file for `@outputs` and `NEST-DEPTH` compliance (`R09-1`, `R09-3`).
// @domain: max_outputs [1..128]
// @domain: max_nest [1..16]
// @outputs: 1
pub fn audit_kotlin_source(_rel_path: &str, raw: &str, max_outputs: usize, max_nest: usize) -> Vec<KtLintIssue> {
    if max_outputs == 0 || max_outputs > 128 || max_nest == 0 || max_nest > 16 {
        return Vec::new();
    }
    let code = strip_kotlin_lexical(raw);
    let bytes = code.as_bytes();
    let mut issues = Vec::new();
    let mut i = 0usize;
    while i < bytes.len() {
        if !is_kw_at(bytes, i, b"fun") {
            i += 1;
            continue;
        }
        let line = raw[..i].bytes().filter(|&c| c == b'\n').count() + 1;
        let head_start = raw[..i].rfind("fun").unwrap_or(i);
        let cmt = extract_kt_comment(&raw[..head_start]);
        if !cmt.contains("@outputs:") {
            issues.push(KtLintIssue { line, rule: "OUTPUTS-MISSING", message: "Missing @outputs annotation".to_string() });
        }
        let Some(brace) = bytes[i..].iter().position(|&c| c == b'{').map(|p| i + p) else {
            i += 3;
            continue;
        };
        let end = find_matching_brace(bytes, brace);
        let body = &code[brace + 1..end];
        if kt_max_nesting(body) > max_nest {
            issues.push(KtLintIssue { line, rule: "NEST-DEPTH", message: "Nesting depth exceeds limit".to_string() });
        }
        i = end;
    }
    issues
}

/// Extracts contiguous leading `//` or `/** ... */` comment lines immediately before a Kotlin declaration.
// @outputs: 1
fn extract_kt_comment(head: &str) -> String {
    let mut lines: Vec<&str> = Vec::new();
    for ln in head.lines().rev() {
        let t = ln.trim();
        if t.is_empty() || t.starts_with('@') {
            continue;
        }
        if t.starts_with("//") || t.starts_with("/*") || t.starts_with('*') || t.ends_with("*/") {
            lines.push(t);
        } else {
            break;
        }
    }
    lines.reverse();
    lines.join("\n")
}

/// Finds the matching closing `}` index for an opening `{` at `open_idx`.
// @outputs: 1
// @domain: open_idx [0..16777216]
fn find_matching_brace(bytes: &[u8], open_idx: usize) -> usize {
    if open_idx >= bytes.len() || open_idx > 16_777_216 {
        return bytes.len();
    }
    let mut depth = 1usize;
    let mut pos = open_idx + 1;
    while pos < bytes.len() && depth > 0 {
        if bytes[pos] == b'{' {
            depth += 1;
        } else if bytes[pos] == b'}' {
            depth -= 1;
        }
        pos += 1;
    }
    pos.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // @outputs: 1
    fn test_strip_and_audit_kotlin() {
        let good = "/** @outputs: 1 */\nfun ok(x: Int): Int {\n  if (x < 0) { return 0 }\n  return x\n}\n";
        assert!(audit_kotlin_source("core/Ok.kt", good, 32, 2).is_empty());
        assert!(kt_has_guard("if (x < 0) { return 0 }", "x"));

        let bad = "fun broken(x: Int): Int {\n  if (x > 0) { if (x > 1) { if (x > 2) { return 3 } } }\n  return 0\n}\n";
        let errs = audit_kotlin_source("core/Bad.kt", bad, 32, 2);
        assert!(errs.iter().any(|e| e.rule == "OUTPUTS-MISSING"));
        assert!(errs.iter().any(|e| e.rule == "NEST-DEPTH"));
    }
}
