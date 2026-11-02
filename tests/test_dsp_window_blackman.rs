// ## Map
// Role: Integration test for Blackman window module
// Inputs: none (uses public API)
// Outputs: test results (assertions)
// Invariants: golden vectors must match within 1e‑4 (coeff) or 1e‑2 (windowed data)

use rust_bricks::dsp_window_blackman::*; // crate name as defined in Cargo.toml

#[test]
fn test_blackman_window_coeff_basic() {
    // Golden vector for N = 5
    let n = 5usize;

    let w0 = blackman_window_coeff(0, n).expect("valid coefficient");
    assert!((w0 - 0.0).abs() < 1e-4);

    let w2 = blackman_window_coeff(2, n).expect("valid coefficient");
    assert!((w2 - 1.0).abs() < 1e-4);

    let w4 = blackman_window_coeff(4, n).expect("valid coefficient");
    assert!((w4 - 0.0).abs() < 1e-4);
}

#[test]
fn test_blackman_window_coeff_invalid() {
    // window_len < 2
    assert!(blackman_window_coeff(0, 1).is_none());

    // index out of range
    assert!(blackman_window_coeff(2, 2).is_none());
}

#[test]
fn test_apply_blackman_window() {
    // Input buffer of ones, length 5 → expected windowed values:
    // [0.0, 0.34, 1.0, 0.34, 0.0]
    let mut data = vec![1.0_f64; 5];
    let ok = apply_blackman_window(&mut data);
    assert!(ok, "apply should succeed for length ≥ 2");

    let expected = [0.0_f64, 0.34_f64, 1.0_f64, 0.34_f64, 0.0_f64];
    for (got, &exp) in data.iter().zip(expected.iter()) {
        assert!((got - exp).abs() < 1e-2);
    }

    // Buffer too short → should return false and leave data untouched
    let mut short = vec![1.0_f64];
    assert!(!apply_blackman_window(&mut short));
}
