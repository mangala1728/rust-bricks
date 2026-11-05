// ## Map
// Role: integration test for ECG-derived respiration rate estimator
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::sensor_ecg_derived_resp::*;

#[test]
fn test_sensor_ecg_derived_resp_golden() {
    assert!((sensor_edr_r_peak_amplitude(&[0.1, 1.5, 0.2], 1, 0.2).unwrap() - 1.3).abs() < 1e-10);
    assert!((sensor_respiration_rate_from_intervals(&[4.0, 4.0, 4.0]).unwrap() - 15.0).abs() < 1e-10);
    assert!((sensor_respiration_rate_from_intervals(&[3.0, 3.0]).unwrap() - 20.0).abs() < 1e-10);
}

#[test]
fn test_sensor_ecg_derived_resp_guards() {
    // Peak index out of bounds
    assert!(sensor_edr_r_peak_amplitude(&[0.1, 1.5], 2, 0.2).is_none());
    // Insufficient intervals (< 2)
    assert!(sensor_respiration_rate_from_intervals(&[4.0]).is_none());
    // Interval out of normal physiological bounds [1.0, 10.0]
    assert!(sensor_respiration_rate_from_intervals(&[0.5, 4.0]).is_none());
    assert!(sensor_respiration_rate_from_intervals(&[15.0, 4.0]).is_none());
    // NaN
    assert!(sensor_edr_r_peak_amplitude(&[0.1, 1.5], 1, f64::NAN).is_none());
}
