// ## Map
// Role: integration test for accelerometer fall detection state machine
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::sensor_fall_detection::*;

#[test]
fn test_sensor_fall_detection_golden() {
    assert!((sensor_svm_magnitude(0.0, 0.0, 9.8).unwrap() - 9.8).abs() < 1e-6);
    assert_eq!(sensor_fall_detect_impact(3.0, 5.0, 25.0, false), Some((true, false)));
    assert_eq!(sensor_fall_detect_impact(28.0, 5.0, 25.0, true), Some((false, true)));
    assert_eq!(sensor_fall_detect_impact(9.8, 5.0, 25.0, false), Some((false, false)));
}

#[test]
fn test_sensor_fall_detection_guards() {
    assert!(sensor_svm_magnitude(f64::NAN, 0.0, 9.8).is_none());
    assert!(sensor_fall_detect_impact(-1.0, 5.0, 25.0, false).is_none());
    assert!(sensor_fall_detect_impact(10.0, 25.0, 5.0, false).is_none());
    assert!(sensor_fall_detect_impact(10.0, -5.0, 25.0, false).is_none());
    assert!(sensor_fall_detect_impact(10.0, 5.0, f64::NAN, false).is_none());
}
