// ## Map
// Role: integration test for dsp_window_hamming
// Inputs: none (uses golden vectors)
// Outputs: test results (assertions)
// Invariants: golden values are fixed, tolerance 1e‑4

use rust_bricks::dsp_window_hamming::{hamming_window_coeff, apply_hamming_window};

/// Helper to compare floating‑point numbers with a tight tolerance.
fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-4
}

#[test]
fn coeff_golden_values() {
    // N = 5 → indices 0,2,4 are enough for the spec
    let n = 5usize;

    let w0 = hamming_window_coeff(0, n).expect("valid coeff");
    let w2 = hamming_window_coeff(2, n).expect("valid coeff");
    let w4 = hamming_window_coeff(4, n).expect("valid coeff");

    assert!(approx_eq(w0, 0.08));
    assert!(approx_eq(w2, 1.00));
    assert!(approx_eq(w4, 0.08));
}

#[test]
fn apply_window_correctness() {
    // Input signal: simple ramp 0..4
    let mut buf = [0.0_f64, 1.0, 2.0, 3.0, 4.0];
    let ok = apply_hamming_window(&mut buf);
    assert!(ok, "apply_hamming_window should succeed");

    // Expected values = original * coeff
    let n = buf.len();
    for i in 0..n {
        let coeff = hamming_window_coeff(i, n).expect("valid coeff");
        let expected = (i as f64) * coeff;
        assert!(approx_eq(buf[i], expected));
    }
}

#[test]
fn edge_cases() {
    // Too short buffer → false
    let mut tiny = [1.0_f64];
    assert!(!apply_hamming_window(&mut tiny));

    // Invalid coefficient request → None
    assert!(hamming_window_coeff(0, 1).is_none());
    assert!(hamming_window_coeff(5, 5).is_none());
}
