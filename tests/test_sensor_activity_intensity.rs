use rust_bricks::sensor_activity_intensity::*;

#[test]
fn test_sensor_sma_basic() {
    // Golden vector 1: static gravity aligned with Z axis
    let ax = [0.0_f64, 0.0];
    let ay = [0.0_f64, 0.0];
    let az = [9.8_f64, 9.8];
    let expected = 9.8_f64;

    assert_eq!(sensor_sma(&ax, &ay, &az), Some(expected));
}

#[test]
fn test_sensor_sma_invalid_inputs() {
    // Empty slice
    assert_eq!(sensor_sma(&[], &[], &[]), None);

    // Mismatched lengths
    assert_eq!(sensor_sma(&[1.0], &[1.0, 2.0], &[1.0]), None);

    // Non‑finite value
    let ax = [1.0_f64, f64::NAN];
    let ay = [1.0_f64, 1.0];
    let az = [1.0_f64, 1.0];
    assert_eq!(sensor_sma(&ax, &ay, &az), None);
}

#[test]
fn test_sensor_classify_activity_golden_vectors() {
    let sedentary = 10.0_f64;
    let active = 15.0_f64;

    // Below sedentary threshold → 0
    assert_eq!(sensor_classify_activity(9.5, sedentary, active), Some(0));

    // Between thresholds → 1
    assert_eq!(sensor_classify_activity(12.0, sedentary, active), Some(1));

    // Above active threshold → 2
    assert_eq!(sensor_classify_activity(18.0, sedentary, active), Some(2));
}

#[test]
fn test_sensor_classify_activity_invalid_params() {
    // Negative SMA
    assert_eq!(sensor_classify_activity(-1.0, 10.0, 15.0), None);

    // NaN SMA
    assert_eq!(sensor_classify_activity(f64::NAN, 10.0, 15.0), None);

    // Invalid thresholds (negative)
    assert_eq!(sensor_classify_activity(12.0, -5.0, 15.0), None);

    // active_threshold < sedentary_threshold
    assert_eq!(sensor_classify_activity(12.0, 10.0, 5.0), None);
}
