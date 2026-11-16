// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/gate_anti_hardcode — Pure-Rust Anti-Hardcode & Float-Axiom Static Gauge (transpiled from `anti_hardcode_gauge.py`)
//! - Inputs: Web HTML/JS text (`web/index.html`) and Rust adapter/core/GUI source text (`src/**/*.rs`)
//! - Outputs: Deterministic `Vec<AntiHardcodeFinding>` defect reports and MSA calibration results
//! - Invariants:
//!   1. Zero external regex crates (`regex`-free ASCII/UTF-8 deterministic scanner).
//!   2. Control-flow nesting depth <= 2 per function; zero recursion; zero `.unwrap()`.
//!   3. Zero `f32`/`f64` leakage check strips `//` line comments before scanning identifiers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AntiHardcodeFinding {
    pub category: &'static str,
    pub message: String,
}

/// Checks if byte is an ASCII identifier character (`[A-Za-z0-9_]`).
/// @outputs: 2
/// @domain: b [0..255]
fn is_ident_byte(b: u8) -> bool {
    if !b.is_ascii() {
        return false;
    }
    b.is_ascii_alphanumeric() || b == b'_'
}

/// Checks if a standalone token appears with non-ident boundaries in `haystack`.
/// @outputs: 2
fn contains_standalone_token(haystack: &str, token: &str) -> bool {
    if token.is_empty() || token.len() > haystack.len() {
        return false;
    }
    let bytes = haystack.as_bytes();
    let mut start = 0usize;
    while let Some(rel) = haystack[start..].find(token) {
        let idx = start + rel;
        let left_ok = idx == 0 || !is_ident_byte(bytes[idx - 1]);
        let right_idx = idx + token.len();
        let right_ok = right_idx >= bytes.len() || !is_ident_byte(bytes[right_idx]);
        if left_ok && right_ok {
            return true;
        }
        start = idx + 1;
    }
    false
}

/// Checks whether `<script ... src="http(s)://...">` appears in HTML.
/// @outputs: 2
fn has_external_cdn_script(html_lower: &str) -> bool {
    let mut pos = 0usize;
    while let Some(rel) = html_lower[pos..].find("<script") {
        let s_idx = pos + rel;
        let end_rel = match html_lower[s_idx..].find('>') {
            Some(e) => e,
            None => break,
        };
        let tag = &html_lower[s_idx..=s_idx + end_rel];
        if tag.contains("src=") && (tag.contains("https://") || tag.contains("http://")) {
            return true;
        }
        pos = s_idx + 7;
    }
    false
}

/// Splits HTML into (non_script_html, concatenated_inline_js) cleanly without recursion.
/// @outputs: 1
fn split_html_and_inline_js(html_text: &str) -> (String, String) {
    let lower = html_text.to_ascii_lowercase();
    let mut html_only = String::with_capacity(html_text.len());
    let mut inline_js = String::new();
    let mut cursor = 0usize;
    while let Some(rel) = lower[cursor..].find("<script") {
        let open_start = cursor + rel;
        html_only.push_str(&html_text[cursor..open_start]);
        let gt_rel = match lower[open_start..].find('>') {
            Some(g) => g,
            None => break,
        };
        let body_start = open_start + gt_rel + 1;
        let close_rel = match lower[body_start..].find("</script>") {
            Some(c) => c,
            None => break,
        };
        inline_js.push_str(&html_text[body_start..body_start + close_rel]);
        inline_js.push('\n');
        cursor = body_start + close_rel + 9;
    }
    html_only.push_str(&html_text[cursor..]);
    (html_only, inline_js)
}

/// Extracts quoted `value="..."` attribute from inside an `<option ...>` tag.
/// @outputs: 6
fn extract_option_value(tag: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let v_idx = lower.find("value")?;
    let after_key = tag[v_idx + 5..].trim_start();
    if !after_key.starts_with('=') {
        return None;
    }
    let after_eq = after_key[1..].trim_start();
    let quote = after_eq.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let rest = &after_eq[1..];
    let end = rest.find(quote)?;
    Some(rest[..end].trim().to_string())
}

/// Checks tag for non-numeric option value and pushes if unique.
/// @outputs: 3
fn push_unique_non_numeric_option(values: &mut Vec<String>, tag: &str) {
    let val = match extract_option_value(tag) {
        Some(v) if !v.is_empty() => v,
        _ => return,
    };
    let is_digits = val.bytes().all(|b| b.is_ascii_digit());
    if !is_digits && !values.contains(&val) {
        values.push(val);
    }
}

/// Collects distinct non-numeric `<option value="...">` identifiers from HTML markup.
/// @outputs: 1
fn collect_option_values(html_only: &str) -> Vec<String> {
    let lower = html_only.to_ascii_lowercase();
    let mut values: Vec<String> = Vec::new();
    let mut pos = 0usize;
    while let Some(rel) = lower[pos..].find("<option") {
        let s_idx = pos + rel;
        let end_rel = match lower[s_idx..].find('>') {
            Some(e) => e,
            None => break,
        };
        push_unique_non_numeric_option(&mut values, &html_only[s_idx..=s_idx + end_rel]);
        pos = s_idx + 7;
    }
    values
}

/// Checks HTML/JS payload for offline CDN traps, dead `<option>` controls, truncated Yoga list, and frozen Dasha.
/// @outputs: 2
pub fn scan_html_anti_hardcode(html_text: &str) -> Vec<AntiHardcodeFinding> {
    let mut findings = Vec::new();
    let lower = html_text.to_ascii_lowercase();
    if has_external_cdn_script(&lower) {
        findings.push(AntiHardcodeFinding {
            category: "OFFLINE-CDN-TRAP",
            message: "web HTML embeds external https:// <script src=...> CDN; fails offline inside standalone Win32/Edge desktop bundle".to_string(),
        });
    }
    let (html_only, inline_js) = split_html_and_inline_js(html_text);
    for val in collect_option_values(&html_only) {
        let sq = format!("'{}'", val);
        let dq = format!("\"{}\"", val);
        if val.len() >= 4 && !inline_js.contains(&sq) && !inline_js.contains(&dq) {
            findings.push(AntiHardcodeFinding {
                category: "DEAD-UI-OPTION",
                message: format!("<option value=\"{}\"> is defined in HTML UI but never handled in JS script", val),
            });
        }
    }
    if inline_js.contains(".slice(startIdx, startIdx + 4)") || inline_js.contains(".slice(0, 4)") {
        findings.push(AntiHardcodeFinding {
            category: "TRUNCATED-YOGA-LIST",
            message: "Yoga panel slices only 4 items (.slice(..., + 4)) instead of listing all scanned yogas".to_string(),
        });
    }
    let has_dasha_logic = inline_js.contains("computeDashaForProfile") || inline_js.contains("/api/calculate");
    if inline_js.contains("主:土星(Sa)") && !has_dasha_logic {
        findings.push(AntiHardcodeFinding {
            category: "HARDCODED-DASHA-TREE",
            message: "Vimshottari spotlight renders hardcoded HTML string '主:土星(Sa)' without dynamic profile calculation".to_string(),
        });
    }
    findings
}

/// Detects stub fixed planetary arrays `[1; 9]` / `[0; 12]` / `[0; 9]` ignoring ASCII spaces.
/// @outputs: 2
fn has_stub_planetary_array(rust_text: &str) -> bool {
    let compact: String = rust_text.chars().filter(|c| *c != ' ' && *c != '\t').collect();
    compact.contains("planetary_signs:[1;9]")
        || compact.contains("planetary_signs:[0;12]")
        || compact.contains("planetary_deg_milli:[0;9]")
}

/// Checks if non-comment lines contain `f32` or `f64` tokens.
/// @outputs: 2
fn has_float_in_code(rust_text: &str) -> bool {
    for line in rust_text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            continue;
        }
        if contains_standalone_token(trimmed, "f32") || contains_standalone_token(trimmed, "f64") {
            return true;
        }
    }
    false
}

/// Checks Rust GUI/adapter/core files for stub arrays, missing HTTP compute routes, and float-axiom leakage.
/// @outputs: 2
pub fn scan_rust_anti_hardcode(rel_path: &str, rust_text: &str) -> Vec<AntiHardcodeFinding> {
    let mut findings = Vec::new();
    let norm_rel = rel_path.replace('\\', "/");
    if has_stub_planetary_array(rust_text) {
        findings.push(AntiHardcodeFinding {
            category: "HARDCODE-STUB-ARRAY",
            message: format!("{}: contains stub placeholder planetary array [1; 9] / [0; 9] instead of real computed ephemeris", norm_rel),
        });
    }
    let has_calc_route = rust_text.contains("/api/calculate") || rust_text.contains("/api/compute") || rust_text.contains("/api/chart");
    if norm_rel.ends_with("cyber_jyotisha_gui.rs") && !has_calc_route {
        findings.push(AntiHardcodeFinding {
            category: "STUB-GUI-NO-API",
            message: format!("{}: embedded HTTP server only exposes /api/ping without /api/calculate backend binding", norm_rel),
        });
    }
    let is_pure_int_path = norm_rel.starts_with("src/core/") || norm_rel.ends_with("pl7_converter.rs");
    if is_pure_int_path && has_float_in_code(rust_text) {
        findings.push(AntiHardcodeFinding {
            category: "FLOAT-IN-PURE-INT",
            message: format!("{}: contains f32/f64 in pure-integer domain path (violates Pure Integer Axiom)", norm_rel),
        });
    }
    findings
}

/// Checks whether `expected` defect category appears in `findings`.
/// @outputs: 2
fn has_category(findings: &[AntiHardcodeFinding], expected: &str) -> bool {
    findings.iter().any(|f| f.category == expected)
}

/// MSA Calibration (--calibrate / --mutate): verifies 3/3 golden pass + 7/7 synthetic defect kills.
/// Returns `(all_ok, std_pass, std_total, chal_killed, chal_total)`.
/// @outputs: 2
pub fn run_anti_hardcode_calibration() -> (bool, usize, usize, usize, usize) {
    let golden_html = concat!(
        "<!DOCTYPE html><html><body>",
        "<select id=\"annualQueryTarget\"><option value=\"ekadashi\">E</option>",
        "<option value=\"rohini\">R</option><option value=\"shravana\">S</option></select>",
        "<script>const targets={ekadashi:1,rohini:4,shravana:22};",
        "async function computeChart(p){return fetch('/api/calculate');}",
        "function renderAllYogas(l){return l.map(y=>y.name).join(',');}",
        "function runTarget(v){return targets['rohini']+targets['shravana']+targets['ekadashi'];}</script></body></html>"
    );
    let golden_gui_rs = "fn handle_http() { if line.starts_with(\"POST /api/calculate\") {} }";
    let golden_pl7_rs = "pub struct Pl7 { pub lagna_milli: u32, pub lon_milli: i64 }";

    let std_pass = usize::from(scan_html_anti_hardcode(golden_html).is_empty())
        + usize::from(scan_rust_anti_hardcode("src/bin/cyber_jyotisha_gui.rs", golden_gui_rs).is_empty())
        + usize::from(scan_rust_anti_hardcode("src/adapters/pl7_converter.rs", golden_pl7_rs).is_empty());

    let chal_kills = [
        has_category(&scan_html_anti_hardcode("<script src=\"https://cdn.tailwindcss.com\"></script><script></script>"), "OFFLINE-CDN-TRAP"),
        has_category(&scan_html_anti_hardcode("<select><option value=\"rohini\">R</option></select><script>const x=1;</script>"), "DEAD-UI-OPTION"),
        has_category(&scan_html_anti_hardcode("<script>const y = YOGA_CATALOG.slice(startIdx, startIdx + 4);</script>"), "TRUNCATED-YOGA-LIST"),
        has_category(&scan_html_anti_hardcode("<script>const h = \"<span>主:土星(Sa)</span>\";</script>"), "HARDCODED-DASHA-TREE"),
        has_category(&scan_rust_anti_hardcode("src/adapters/pl7_converter.rs", "Pl7Profile { planetary_signs: [1; 9] }"), "HARDCODE-STUB-ARRAY"),
        has_category(&scan_rust_anti_hardcode("src/bin/cyber_jyotisha_gui.rs", "if first_line.starts_with(\"GET /api/ping\") {}"), "STUB-GUI-NO-API"),
        has_category(&scan_rust_anti_hardcode("src/adapters/pl7_converter.rs", "pub raw_jd: f64,"), "FLOAT-IN-PURE-INT"),
    ];
    let chal_killed = chal_kills.iter().filter(|&&k| k).count();
    let all_ok = std_pass == 3 && chal_killed == 7;
    (all_ok, std_pass, 3, chal_killed, 7)
}
