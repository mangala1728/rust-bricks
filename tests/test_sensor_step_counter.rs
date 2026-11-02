use rust_bricks::sensor_step_counter::*;

#[test]
fn test_sensor_accel_magnitude_basic() {
    // Golden vector 1 – static gravity
    let mag = sensor_accel_magnitude(0.0, 0.0, 9.8);
    assert_eq!(mag, Some(9.8));
}

#[test]
fn test_sensor_accel_magnitude_invalid() {
    // Any NaN should be rejected
    let mag = sensor_accel_magnitude(f64::NAN, 0.0, 0.0);
    assert_eq!(mag, None);
}

#[test]
fn test_step_detect_invalid_inputs() {
    // Alpha out of range
    let res = sensor_step_detect_step(10.0, 9.8, -0.1, 1.0, 0);
    assert_eq!(res, None);
    // State out of range
    let res = sensor_step_detect_step(10.0, 9.8, 0.5, 1.0, 2);
    assert_eq!(res, None);
}

#[test]
fn test_step_detect_peak_transition() {
    // Golden vector 2 – peak crossing
    let res = sensor_step_detect_step(12.0, 9.8, 0.1, 1.0, 0);
    // Expected: state 1, EMA 10.02, no step yet
    assert_eq!(res, Some((1, 10.02, false)));
}

#[test]
fn test_step_detect_valley_completion() {
    // Golden vector 3 – valley crossing, step completed
    let res = sensor_step_detect_step(7.0, 10.0, 0.1, 1.0, 1);
    // Expected: state 0, EMA 9.7, step flag true
    assert_eq!(res, Some((0, 9.7, true)));
}
