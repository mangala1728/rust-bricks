// ## Map
// Role: Integration test for `fin_var_historical` module.
// Inputs: None (uses hard‑coded golden vectors).
// Outputs: Test success/failure reported by `cargo test`.
// Invariants: All assertions use a strict absolute tolerance of 1e‑4.

use rust_bricks::fin_var_historical::*;

#[test]
fn test_invalid_inputs_return_none() {
    // Too few observations
    assert!(fin_var_historical(&[0.01], 0.95).is_none());
    assert!(fin_cvar_historical(&[0.01], 0.95).is_none());

    // Confidence out of range
    let data = [0.01, -0.02];
    assert!(fin_var_historical(&data, 0.0).is_none());
    assert!(fin_var_historical(&data, 1.0).is_none());
    assert!(fin_cvar_historical(&data, -0.1).is_none());
    assert!(fin_cvar_historical(&data, 1.5).is_none());

    // Non‑finite numbers
    let bad = [0.01, f64::NAN];
    assert!(fin_var_historical(&bad, 0.95).is_none());
    assert!(fin_cvar_historical(&bad, 0.95).is_none());
}

#[test]
fn test_var_and_cvar_golden_vectors() {
    // Sample returns (sorted already for readability)
    let returns = [
        -0.05, -0.04, -0.03, -0.02, -0.01,
         0.01,  0.02,  0.03,  0.04,  0.05,
    ];

    // ---- 90 % confidence ----------------------------------------------------
    let var_90 = fin_var_historical(&returns, 0.90).expect("valid VaR");
    let cvar_90 = fin_cvar_historical(&returns, 0.90).expect("valid CVaR");
    assert!((var_90 - 0.04).abs() < 1e-4, "VaR 90% mismatch: {}", var_90);
    assert!((cvar_90 - 0.05).abs() < 1e-4, "CVaR 90% mismatch: {}", cvar_90);

    // ---- 80 % confidence ----------------------------------------------------
    let var_80 = fin_var_historical(&returns, 0.80).expect("valid VaR");
    let cvar_80 = fin_cvar_historical(&returns, 0.80).expect("valid CVaR");
    assert!((var_80 - 0.03).abs() < 1e-4, "VaR 80% mismatch: {}", var_80);
    assert!((cvar_80 - 0.045).abs() < 1e-4, "CVaR 80% mismatch: {}", cvar_80);
}
