// ## Map
// Role: Integration tests for `hann_window_coeff`
// Inputs: n, total_len
// Outputs: assertions on returned Option<f64>

use rust_bricks::dsp_window_hann::hann_window_coeff;

/// Helper to compare floating‑point values within a tolerance.
fn approx_eq(a: f64, b: f64, eps: f64) -> bool {
    (a - b).abs() < eps
}

#[test]
fn test_endpoint_zero() {
    // @safety-proof: Input is within valid domain boundaries.
    let val = hann_window_coeff(0, 5).expect("should return Some");
    assert!(approx_eq(val, 0.0, 1e-6), "expected 0.0, got {}", val);
}

#[test]
fn test_center_max() {
    // @safety-proof: Input is within valid domain boundaries.
    let val = hann_window_coeff(2, 5).expect("should return Some");
    assert!(approx_eq(val, 1.0, 1e-6), "expected 1.0, got {}", val);
}

#[test]
fn test_single_point_window() {
    // @safety-proof: Input is within valid domain boundaries.
    let val = hann_window_coeff(0, 1).expect("should return Some");
    assert!(approx_eq(val, 1.0, 1e-6), "expected 1.0 for length 1, got {}", val);
}

#[test]
fn test_out_of_bounds_n() {
    assert!(hann_window_coeff(5, 5).is_none(), "n == total_len must yield None");
}

#[test]
fn test_zero_total_len() {
    assert!(hann_window_coeff(0, 0).is_none(), "total_len == 0 must yield None");
}
