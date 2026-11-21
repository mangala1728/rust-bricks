// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Zero-dependency Python structural & R09 contract analyzer (`core` brick)
//! - Inputs: Python source text (`&str`), relative path (`&str`), `max_outputs`, `max_nest`, `max_lines`
//! - Outputs: `Vec<PyViolation>` (`OUTPUTS-MISSING`, `OUTPUTS-EXCEED`, `NEST-DEPTH`, `FUNC-LENGTH`, `GLOBAL`)
//! - Invariants:
//!   1. Zero external crates (`std` only) and zero `static mut` (`R09-2`).
//!   2. Zero recursion (`R09-7`), bounded line/indentation scans (`R09-8`), nesting depth <= 2 (`R09-3`).
//!   3. Explicit `@domain` bounds and runtime guards on all numeric parameters (`R09-4`).

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PyViolation {
    pub line: usize,
    pub rule: String,
    pub message: String,
}

/// Computes leading space indentation width of a Python source line.
// @outputs: 1
fn py_indent(line: &str) -> usize {
    let mut n = 0usize;
    for b in line.bytes() {
        if b == b' ' {
            n += 1;
        } else if b == b'\t' {
            n += 4;
        } else {
            break;
        }
    }
    n
}

/// Checks if a trimmed Python line opens a control-flow nesting level.
// @outputs: 2
fn is_py_control_header(trimmed: &str) -> bool {
    let kws = ["if ", "elif ", "for ", "while ", "try:", "with ", "match "];
    for kw in kws {
        if trimmed.starts_with(kw) {
            return true;
        }
    }
    false
}

/// Pops closed Python indentation blocks when `ind` dedents past stack tops and returns updated depth.
// @outputs: 1
// @domain: ind [0..1024]
// @domain: cur_depth [0..1024]
fn pop_closed_py_blocks(ind: usize, trimmed: &str, stack: &mut Vec<(usize, bool)>, cur_depth: usize) -> usize {
    if ind > 1024 {
        return cur_depth;
    }
    if cur_depth > 1024 {
        return 1024;
    }
    let mut d = cur_depth;
    let is_cont = trimmed.starts_with("elif ") || trimmed.starts_with("else:") || trimmed.starts_with("except");
    while let Some(&(top_ind, was_ctrl)) = stack.last() {
        if ind > top_ind || is_cont {
            break;
        }
        stack.pop();
        d = d.saturating_sub(usize::from(was_ctrl));
    }
    d
}

/// Computes maximum control-flow nesting depth across Python function body lines `[start_idx..end_idx)`.
// @domain: start_idx [0..1048576]
// @domain: end_idx [0..1048576]
// @outputs: 1
pub fn py_max_nesting(lines: &[&str], start_idx: usize, end_idx: usize) -> usize {
    if start_idx > 1_048_576 || end_idx > 1_048_576 || start_idx >= end_idx || end_idx > lines.len() {
        return 0;
    }
    let mut stack: Vec<(usize, bool)> = Vec::new();
    let mut max_depth = 0usize;
    let mut cur_depth = 0usize;

    for idx in start_idx..end_idx {
        let trimmed = lines[idx].trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let ind = py_indent(lines[idx]);
        cur_depth = pop_closed_py_blocks(ind, trimmed, &mut stack, cur_depth);
        if is_py_control_header(trimmed) && !trimmed.starts_with("elif ") {
            stack.push((ind, true));
            cur_depth += 1;
            max_depth = max_depth.max(cur_depth);
        }
    }
    max_depth
}

/// Parses `@outputs: N` from a single docstring line if present.
// @outputs: 2
fn parse_outputs_tag(trimmed: &str) -> Option<usize> {
    let pos = trimmed.find("@outputs:")?;
    let rest = trimmed[pos + 9..].trim();
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse::<usize>().ok()
}

/// Checks whether a Python docstring after `def` contains `@outputs: N`.
// @outputs: 2
// @domain: def_idx [0..1048576]
fn extract_py_outputs_in_docstring(lines: &[&str], def_idx: usize) -> Option<usize> {
    if def_idx >= lines.len() || def_idx > 1_048_576 {
        return None;
    }
    let limit = (def_idx + 20).min(lines.len());
    for i in (def_idx + 1)..limit {
        let t = lines[i].trim();
        if let Some(n) = parse_outputs_tag(t) {
            return Some(n);
        }
        if i > def_idx + 1 && (t.starts_with("def ") || t.starts_with("return ")) {
            break;
        }
    }
    None
}

/// Finds the exclusive end line index of a Python `def` block starting at `def_idx`.
// @outputs: 1
// @domain: def_idx [0..1048576]
fn find_py_def_end(lines: &[&str], def_idx: usize) -> usize {
    if def_idx >= lines.len() || def_idx > 1_048_576 {
        return lines.len();
    }
    let def_ind = py_indent(lines[def_idx]);
    let mut i = def_idx + 1;
    while i < lines.len() {
        let t = lines[i].trim();
        if !t.is_empty() && !t.starts_with('#') && py_indent(lines[i]) <= def_ind {
            return i;
        }
        i += 1;
    }
    lines.len()
}

/// Audits a Python module for `OUTPUTS-MISSING`, `OUTPUTS-EXCEED`, `NEST-DEPTH`, and `FUNC-LENGTH` (`R09`).
// @domain: max_outputs [1..128]
// @domain: max_nest [1..16]
// @domain: max_lines [1..1000]
// @outputs: 1
pub fn audit_python_source(
    _rel_path: &str,
    raw: &str,
    max_outputs: usize,
    max_nest: usize,
    max_lines: usize,
) -> Vec<PyViolation> {
    if max_outputs == 0 || max_outputs > 128 || max_nest == 0 || max_nest > 16 || max_lines == 0 || max_lines > 1000 {
        return Vec::new();
    }
    let lines: Vec<&str> = raw.lines().collect();
    let mut out = Vec::new();
    for i in 0..lines.len() {
        let t = lines[i].trim();
        if !(t.starts_with("def ") || t.starts_with("async def ")) {
            continue;
        }
        let end = find_py_def_end(&lines, i);
        let line_no = i + 1;
        let declared = extract_py_outputs_in_docstring(&lines, i);
        if declared.is_none() {
            out.push(PyViolation { line: line_no, rule: "OUTPUTS-MISSING".to_string(), message: "Missing @outputs".to_string() });
        }
        if declared.unwrap_or(0) > max_outputs {
            out.push(PyViolation { line: line_no, rule: "OUTPUTS-EXCEED".to_string(), message: format!("@outputs > {}", max_outputs) });
        }
        if end.saturating_sub(i) > max_lines {
            out.push(PyViolation { line: line_no, rule: "FUNC-LENGTH".to_string(), message: "Function exceeds line limit".to_string() });
        }
        if py_max_nesting(&lines, i + 1, end) > max_nest {
            out.push(PyViolation { line: line_no, rule: "NEST-DEPTH".to_string(), message: "Control flow nesting exceeds limit".to_string() });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // @outputs: 1
    fn test_audit_python_source() {
        let ok = "def f(x: int) -> int:\n    \"\"\"Doc.\n\n    @outputs: 1\n    \"\"\"\n    if x < 0:\n        return 0\n    return x\n";
        assert!(audit_python_source("app/ok.py", ok, 32, 2, 100).is_empty());

        let bad = "def g(x: int) -> int:\n    if x > 0:\n        if x > 1:\n            if x > 2:\n                return 3\n    return 0\n";
        let errs = audit_python_source("app/bad.py", bad, 32, 2, 100);
        assert!(errs.iter().any(|v| v.rule == "OUTPUTS-MISSING"));
        assert!(errs.iter().any(|v| v.rule == "NEST-DEPTH"));
    }
}
