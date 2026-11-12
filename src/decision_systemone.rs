// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: System One Lightweight Decision Model & Risk Scoring Adapter (safety/core)
//! - Inputs: Command string, working dir, preliminary risk score, raw probabilities, invoice fields, modal billing raw
//! - Outputs: `DecisionVerdict` (Allow/Block/Escalate/Bypass), confidence score, calibrated invoice reconciliation, billing snapshot
//! - Invariants: Fail-safe fallback; bounded loops; zero mutable globals; zero recursion; nest depth <= 2.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionVerdict {
    Allow,
    Block,
    Escalate,
    Bypass,
}

impl DecisionVerdict {
    /// Returns the canonical uppercase string representation of the verdict.
    /// @outputs: 4
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionVerdict::Allow => "ALLOW",
            DecisionVerdict::Block => "BLOCK",
            DecisionVerdict::Escalate => "ESCALATE",
            DecisionVerdict::Bypass => "BYPASS",
        }
    }
}

/// Trims and bounds state text for lightweight System One consumption.
/// @outputs: 1
/// @domain: max_len [100..10000]
pub fn sanitize_state_text(text: &str, max_len: usize) -> String {
    if max_len < 100 || max_len > 10000 {
        return text.trim().to_string();
    }

    let trimmed = text.trim();
    if trimmed.len() <= max_len {
        return trimmed.to_string();
    }

    let mut out = String::with_capacity(max_len + 64);
    out.push_str(&trimmed[..max_len]);
    out.push_str("\n...[truncated for lightweight decision state]");
    out
}

/// Evaluates calibrated choice probabilities against configured safety thresholds.
/// @outputs: 3
/// @domain: p_allow [0.0..1.0]
/// @domain: p_block [0.0..1.0]
/// @domain: p_escalate [0.0..1.0]
/// @domain: block_thresh [0.5..1.0]
/// @domain: escalate_thresh [0.4..1.0]
pub fn evaluate_probabilities(
    p_allow: f64,
    p_block: f64,
    p_escalate: f64,
    block_thresh: f64,
    escalate_thresh: f64,
) -> (DecisionVerdict, f64, &'static str) {
    if p_allow < 0.0 || p_allow > 1.0
        || p_block < 0.0 || p_block > 1.0
        || p_escalate < 0.0 || p_escalate > 1.0
        || block_thresh < 0.5 || block_thresh > 1.0
        || escalate_thresh < 0.4 || escalate_thresh > 1.0
    {
        return (DecisionVerdict::Bypass, 0.0, "Invalid domain parameters");
    }

    if p_block >= block_thresh {
        return (DecisionVerdict::Block, p_block, "P(BLOCK) >= threshold");
    }
    if p_escalate >= escalate_thresh {
        return (DecisionVerdict::Escalate, p_escalate, "P(ESCALATE) >= threshold");
    }
    if p_allow >= 0.50 {
        return (DecisionVerdict::Allow, p_allow, "P(ALLOW) >= 0.50");
    }
    if p_block > p_allow && p_block > p_escalate {
        return (DecisionVerdict::Block, p_block, "P(BLOCK) is plurality");
    }

    (DecisionVerdict::Escalate, p_escalate, "Ambiguous probability distribution")
}

/// Fast bounded scan to extract verdict choice and confidence from JSON string.
/// @outputs: 2
pub fn extract_choice_from_json(json_str: &str) -> (DecisionVerdict, f64) {
    let limit = json_str.len().min(65536);
    let slice = &json_str[..limit];

    let choice = if slice.contains("\"choice\": \"BLOCK\"")
        || slice.contains("\"choice\":\"BLOCK\"")
        || slice.contains("choice: BLOCK")
        || slice.contains("choice:BLOCK")
    {
        DecisionVerdict::Block
    } else if slice.contains("\"choice\": \"ALLOW\"")
        || slice.contains("\"choice\":\"ALLOW\"")
        || slice.contains("choice: ALLOW")
        || slice.contains("choice:ALLOW")
    {
        DecisionVerdict::Allow
    } else if slice.contains("\"choice\": \"ESCALATE\"")
        || slice.contains("\"choice\":\"ESCALATE\"")
        || slice.contains("choice: ESCALATE")
        || slice.contains("choice:ESCALATE")
    {
        DecisionVerdict::Escalate
    } else {
        DecisionVerdict::Bypass
    };

    let pos = if let Some(p) = slice.find("\"confidence\":") {
        Some(p + 13)
    } else if let Some(p) = slice.find("confidence:") {
        Some(p + 11)
    } else {
        None
    };

    let conf = if let Some(p) = pos {
        let after = &slice[p..];
        let trimmed = after.trim_start();
        let num_end = trimmed.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(trimmed.len());
        trimmed[..num_end].parse::<f64>().unwrap_or(0.80)
    } else {
        0.80
    };

    (choice, conf)
}

/// Deterministic invoice arithmetic and tax ID reconciliation.
/// @outputs: 2
/// @domain: claimed_amt [0.0..100000000.0]
/// @domain: inv_amt [0.0..100000000.0]
/// @domain: max_diff [0.0..10.0]
pub fn verify_invoice_match(
    claimed_amt: f64,
    inv_amt: f64,
    claimed_tax_id: &str,
    inv_tax_id: &str,
    max_diff: f64,
) -> (bool, f64) {
    if claimed_amt < 0.0 || claimed_amt > 100_000_000.0
        || inv_amt < 0.0 || inv_amt > 100_000_000.0
        || max_diff < 0.0 || max_diff > 10.0
    {
        return (false, 0.0);
    }

    let amt_ok = (claimed_amt - inv_amt).abs() <= max_diff;
    let t_claim = claimed_tax_id.trim();
    let t_inv = inv_tax_id.trim();
    let tax_ok = !t_claim.is_empty() && t_claim == t_inv;

    let is_approved = amt_ok && tax_ok;
    let confidence = if is_approved { 1.0 } else { 0.0 };
    (is_approved, confidence)
}

/// Helper to parse floating point number inside JSON key value without external dependencies.
/// @outputs: 1
fn extract_f64_key(json_str: &str, key: &str) -> f64 {
    let needle_q = format!("\"{}\":", key);
    let needle_u = format!("{}:", key);
    let pos = if let Some(p) = json_str.find(&needle_q) {
        p + needle_q.len()
    } else if let Some(p) = json_str.find(&needle_u) {
        p + needle_u.len()
    } else {
        return 0.0;
    };

    let after = json_str[pos..].trim_start();
    let unquoted = if after.starts_with('"') {
        &after[1..]
    } else {
        after
    };

    let end_pos = unquoted
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-' && c != 'e' && c != 'E')
        .unwrap_or(unquoted.len());

    unquoted[..end_pos].parse::<f64>().unwrap_or(0.0)
}

/// Parses Modal billing summary JSON and returns (metered_cost, credits, billed_cost, is_alert).
/// @outputs: 4
pub fn parse_modal_billing_summary(json_str: &str) -> (f64, f64, f64, bool) {
    let metered = extract_f64_key(json_str, "metered_cost");
    let credits = extract_f64_key(json_str, "credits");
    let billed = extract_f64_key(json_str, "billed_cost");

    let is_alert = metered >= 28.0;
    (metered, credits, billed, is_alert)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_decision_verdict_str() {
        assert_eq!(DecisionVerdict::Allow.as_str(), "ALLOW");
        assert_eq!(DecisionVerdict::Block.as_str(), "BLOCK");
        assert_eq!(DecisionVerdict::Escalate.as_str(), "ESCALATE");
        assert_eq!(DecisionVerdict::Bypass.as_str(), "BYPASS");
    }

    /// @outputs: 1
    #[test]
    fn test_sanitize_state_text() {
        let text = "   echo 'hello world'   ";
        let res = sanitize_state_text(text, 500);
        assert_eq!(res, "echo 'hello world'");

        let long_text = "a".repeat(200);
        let truncated = sanitize_state_text(&long_text, 100);
        assert!(truncated.contains("...[truncated for lightweight decision state]"));
    }

    /// @outputs: 1
    #[test]
    fn test_evaluate_probabilities() {
        let (v1, conf1, reason1) = evaluate_probabilities(0.01, 0.90, 0.09, 0.85, 0.60);
        assert_eq!(v1, DecisionVerdict::Block);
        assert_eq!(conf1, 0.90);
        assert_eq!(reason1, "P(BLOCK) >= threshold");

        let (v2, conf2, _) = evaluate_probabilities(0.10, 0.20, 0.70, 0.85, 0.60);
        assert_eq!(v2, DecisionVerdict::Escalate);
        assert_eq!(conf2, 0.70);

        let (v3, conf3, _) = evaluate_probabilities(0.95, 0.02, 0.03, 0.85, 0.60);
        assert_eq!(v3, DecisionVerdict::Allow);
        assert_eq!(conf3, 0.95);
    }

    /// @outputs: 1
    #[test]
    fn test_extract_choice_from_json() {
        let raw = r#"{"model":"nimble","answers":{"verdict":{"choice":"BLOCK","confidence":0.8595}}}"#;
        let (choice, conf) = extract_choice_from_json(raw);
        assert_eq!(choice, DecisionVerdict::Block);
        assert_eq!(conf, 0.8595);
    }

    /// @outputs: 1
    #[test]
    fn test_verify_invoice_match() {
        let (ok1, conf1) = verify_invoice_match(1500.0, 1500.0, "12345678", "12345678", 0.01);
        assert_eq!(ok1, true);
        assert_eq!(conf1, 1.0);

        let (ok2, conf2) = verify_invoice_match(1500.0, 1600.0, "12345678", "12345678", 0.01);
        assert_eq!(ok2, false);
        assert_eq!(conf2, 0.0);
    }

    /// @outputs: 1
    #[test]
    fn test_parse_modal_billing_summary() {
        let json_data = r#"{
            "metered_cost": "0.16112186",
            "billed_cost": "0E-8",
            "adjustments": {
                "credits": "-0.16000000"
            }
        }"#;
        let (metered, credits, billed, is_alert) = parse_modal_billing_summary(json_data);
        assert_eq!(metered, 0.16112186);
        assert_eq!(credits, -0.16);
        assert_eq!(billed, 0.0);
        assert_eq!(is_alert, false);
    }
}
