// ## Map
// Role: Integration test for fin_hurst_exponent_rs module
// Inputs: series vectors
// Outputs: assertions on rs_ratio and hurst_exponent
// Invariants: 0 external dependencies, deterministic golden values

use rust_bricks::fin_hurst_exponent_rs::*;

#[test]
fn golden_sequence_behaviour() {
    let s = [1.0_f64, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let (rs, h) = fin_compute_hurst_rs(&s).expect("valid series should return Some");
    assert!(rs > 1.0, "rs_ratio must be > 1.0 for increasing series");
    assert!(h > 0.0 && h <= 1.0, "hurst must lie in (0,1] for valid series");
}

#[test]
fn constant_series_is_rejected() {
    let const_series = [1.0_f64; 8];
    assert!(fin_compute_hurst_rs(&const_series).is_none(),
            "constant series (zero variance) must return None");
}

#[test]
fn short_series_is_rejected() {
    let short = [1.0_f64, 2.0, 3.0];
    assert!(fin_compute_hurst_rs(&short).is_none(),
            "series shorter than 8 must return None");
}

#[test]
fn nan_series_is_rejected() {
    let mut bad = [1.0_f64; 8];
    bad[3] = f64::NAN;
    assert!(fin_compute_hurst_rs(&bad).is_none(),
            "series containing NaN must return None");
}
