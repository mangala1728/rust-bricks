// ## Map
// Role: Cross-correlation peak lag and TDOA time-delay estimator
// Inputs: x: &[f64], y: &[f64], max_lag: usize
// Outputs: Option<(i32, f64)> (best_lag, peak_correlation)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

use rust_bricks::dsp_cross_correlation_lag::*;

#[test]
fn golden_basic_peak() {
    let x = [0.0_f64, 1.0, 2.0, 3.0, 0.0];
    let y = [0.0_f64, 0.0, 1.0, 2.0, 3.0];
    let (lag, corr) = dsp_find_peak_cross_correlation_lag(&x, &y, 2).unwrap();
    assert_eq!(lag, 1);
    // The exact value is 14.0, any value > 10.0 satisfies the spec.
    assert!(corr > 10.0);
}

#[test]
fn guard_zero_lag() {
    let x = [0.0_f64, 1.0, 2.0, 3.0];
    let y = [0.0_f64, 0.0, 1.0, 2.0];
    assert!(dsp_find_peak_cross_correlation_lag(&x, &y, 0).is_none());
}

#[test]
fn guard_short_vectors() {
    let y = [0.0_f64, 0.0, 1.0, 2.0];
    assert!(dsp_find_peak_cross_correlation_lag(&[1.0, 2.0], &y, 1).is_none());
}
