// ## Map
// Role: integration test for skin temperature EMA smoothing and fever triage
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::sensor_skin_temp_ema::*;

#[test]
fn test_sensor_skin_temp_ema_golden() {
    let smooth = sensor_skin_temp_smooth(37.0, 36.0, 0.2).unwrap();
    assert!((smooth - 36.2).abs() < 1e-10);

    assert_eq!(sensor_fever_triage(36.8, 37.3, 38.5), Some(0));
    assert_eq!(sensor_fever_triage(37.5, 37.3, 38.5), Some(1));
    assert_eq!(sensor_fever_triage(39.2, 37.3, 38.5), Some(2));
}

#[test]
fn test_sensor_skin_temp_ema_guards() {
    assert!(sensor_skin_temp_smooth(9.9, 36.0, 0.2).is_none());
    assert!(sensor_skin_temp_smooth(50.1, 36.0, 0.2).is_none());
    assert!(sensor_skin_temp_smooth(37.0, 9.9, 0.2).is_none());
    assert!(sensor_skin_temp_smooth(37.0, 50.1, 0.2).is_none());
    assert!(sensor_skin_temp_smooth(37.0, 36.0, -0.1).is_none());
    assert!(sensor_skin_temp_smooth(37.0, 36.0, 1.1).is_none());
    assert!(sensor_skin_temp_smooth(f64::NAN, 36.0, 0.2).is_none());

    assert!(sensor_fever_triage(24.9, 37.3, 38.5).is_none());
    assert!(sensor_fever_triage(45.1, 37.3, 38.5).is_none());
    assert!(sensor_fever_triage(37.0, 35.9, 38.5).is_none());
    assert!(sensor_fever_triage(37.0, 38.1, 38.5).is_none());
    assert!(sensor_fever_triage(37.0, 37.3, 37.9).is_none());
    assert!(sensor_fever_triage(37.0, 37.3, 42.1).is_none());
    assert!(sensor_fever_triage(37.0, 39.0, 38.0).is_none());
    assert!(sensor_fever_triage(f64::NAN, 37.3, 38.5).is_none());
}
