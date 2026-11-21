// SPDX-License-Identifier: MIT OR Apache-2.0
//! ## Map
//! - Role: core/sew_craft_qa_forensic — Pure-Integer Sewing Pattern, Brother M370 Hoop & Pure-Vector PDF QA Brick
//! - Inputs: `0.1mm` integer dimensions (`seam_01mm`, `punch_01mm`, `width_01mm`, `height_01mm`, `len_a_01mm`, `len_b_01mm`), PDF text, `pattern_spec.json` text
//! - Outputs: Boolean hardware invariant checks & bitmask audit (`0` = compliant)
//! - Invariants:
//!   1. Seam allowance >= 50 (5.0mm physical lower bound).
//!   2. Transparent acrylic punch hole == 25 (2.5mm locked constant).
//!   3. Brother M370 single hoop <= 980 x 980 (98.0mm x 98.0mm).
//!   4. Zero external dependencies, zero mutable globals (`R09-2`), zero recursion (`R09-7`), nesting depth <= 2 (`R09-3`).

pub const SEAM_MIN_01MM: i32 = 50;
pub const PUNCH_LOCK_01MM: i32 = 25;
pub const M370_HOOP_MAX_01MM: i32 = 980;

pub const VIOLATION_SEAM_UNDER_5MM: u32 = 1 << 0;
pub const VIOLATION_PUNCH_NOT_25: u32 = 1 << 1;
pub const VIOLATION_M370_HOOP_OVERFLOW: u32 = 1 << 2;

/// Validates that seam allowance is at least `5.0mm` (`50` units of `0.1mm`).
/// @outputs: 2
/// @domain: seam_01mm [0..5000]
pub fn validate_seam_allowance_01mm(seam_01mm: i32) -> bool {
    if seam_01mm < 0 || seam_01mm > 5000 {
        return false;
    }
    seam_01mm >= SEAM_MIN_01MM
}

/// Validates that transparent acrylic punch hole diameter is locked to `2.5mm` (`25` units of `0.1mm`).
/// @outputs: 2
/// @domain: punch_01mm [0..500]
pub fn validate_punch_hole_01mm(punch_01mm: i32) -> bool {
    if punch_01mm < 0 || punch_01mm > 500 {
        return false;
    }
    punch_01mm == PUNCH_LOCK_01MM
}

/// Validates that embroidery dimensions fit within Brother M370 safe hoop (`<= 980 x 980` units of `0.1mm`).
/// @outputs: 2
/// @domain: width_01mm [0..10000]
/// @domain: height_01mm [0..10000]
pub fn validate_m370_hoop_extents_01mm(width_01mm: i32, height_01mm: i32) -> bool {
    if width_01mm < 0 || width_01mm > 10000 || height_01mm < 0 || height_01mm > 10000 {
        return false;
    }
    width_01mm <= M370_HOOP_MAX_01MM && height_01mm <= M370_HOOP_MAX_01MM
}

/// Validates that paired seam lengths between two panels differ by at most `max_diff_01mm`.
/// @outputs: 2
/// @domain: len_a_01mm [0..100000]
/// @domain: len_b_01mm [0..100000]
/// @domain: max_diff_01mm [0..1000]
pub fn validate_seam_pairing_diff_01mm(len_a_01mm: i32, len_b_01mm: i32, max_diff_01mm: i32) -> bool {
    if len_a_01mm < 0 || len_a_01mm > 100_000 || len_b_01mm < 0 || len_b_01mm > 100_000 || max_diff_01mm < 0 || max_diff_01mm > 1000 {
        return false;
    }
    let diff = (len_a_01mm - len_b_01mm).abs();
    diff <= max_diff_01mm
}

/// Validates convenience-store pure-vector PDF invariants: `/PrintScaling /None` present and zero `/Font`.
/// @outputs: 2
pub fn validate_pure_vector_pdf_text(pdf_text: &str) -> bool {
    pdf_text.contains("/PrintScaling /None") && !pdf_text.contains("/Font")
}

/// Extracts an integer field value for `"<key>": <int>` from a JSON slice using pure `std`.
/// @outputs: 2
pub fn extract_json_int_field(json_text: &str, key: &str) -> Option<i32> {
    let pat = format!("\"{}\"", key);
    let pos = json_text.find(&pat)?;
    let after_key = &json_text[pos + pat.len()..];
    let colon = after_key.find(':')?;
    let rest = after_key[colon + 1..].trim_start();
    let num_str: String = rest
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .collect();
    if num_str.is_empty() {
        return None;
    }
    num_str.parse::<i32>().ok()
}

/// Extracts the `"embroidery": { ... }` object substring if present.
/// @outputs: 2
fn extract_embroidery_block(json_text: &str) -> Option<&str> {
    let pos = json_text.find("\"embroidery\"")?;
    let after = &json_text[pos..];
    let open = after.find('{')?;
    let sub = &after[open..];
    let close = sub.find('}')?;
    Some(&sub[..=close])
}

/// Audits a `pattern_spec.json` payload and returns a bitmask of violated hardware invariants (`0` = PASS).
/// @outputs: 8
pub fn audit_pattern_spec_json(json_text: &str) -> u32 {
    let mut mask = 0u32;
    let seam = extract_json_int_field(json_text, "seam_allowance_01mm").unwrap_or(100);
    if !validate_seam_allowance_01mm(seam) {
        mask |= VIOLATION_SEAM_UNDER_5MM;
    }
    let punch = extract_json_int_field(json_text, "punch_hole_diameter_01mm").unwrap_or(25);
    if !validate_punch_hole_01mm(punch) {
        mask |= VIOLATION_PUNCH_NOT_25;
    }
    if let Some(emb) = extract_embroidery_block(json_text) {
        let w = extract_json_int_field(emb, "width_01mm").unwrap_or(0);
        let h = extract_json_int_field(emb, "height_01mm").unwrap_or(0);
        if !validate_m370_hoop_extents_01mm(w, h) {
            mask |= VIOLATION_M370_HOOP_OVERFLOW;
        }
    }
    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_sew_craft_qa_golden_vectors() {
        assert!(validate_seam_allowance_01mm(50));
        assert!(!validate_seam_allowance_01mm(49));
        assert!(validate_punch_hole_01mm(25));
        assert!(!validate_punch_hole_01mm(14));
        assert!(validate_m370_hoop_extents_01mm(980, 980));
        assert!(!validate_m370_hoop_extents_01mm(981, 500));
        assert!(validate_seam_pairing_diff_01mm(1200, 1205, 5));
        assert!(!validate_seam_pairing_diff_01mm(1200, 1206, 5));
        assert!(validate_pure_vector_pdf_text("%PDF-1.4\n/PrintScaling /None\n%%EOF"));
        assert!(!validate_pure_vector_pdf_text("%PDF-1.4\n/PrintScaling /None\n/Font <<>>"));

        let clean = r#"{"seam_allowance_01mm": 50, "punch_hole_diameter_01mm": 25, "embroidery": {"width_01mm": 900, "height_01mm": 900}}"#;
        assert_eq!(audit_pattern_spec_json(clean), 0);

        let bad = r#"{"seam_allowance_01mm": 30, "punch_hole_diameter_01mm": 15, "embroidery": {"width_01mm": 990, "height_01mm": 500}}"#;
        assert_eq!(
            audit_pattern_spec_json(bad),
            VIOLATION_SEAM_UNDER_5MM | VIOLATION_PUNCH_NOT_25 | VIOLATION_M370_HOOP_OVERFLOW
        );
    }
}
