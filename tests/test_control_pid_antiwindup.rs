use rust_bricks::control_pid_antiwindup::*;

/// Test: normal operation without saturation
#[test]
fn test_normal_unsaturated() {
    // @safety-proof: parameters are valid, function must return Some
    let (out, next_i) = control_pid_step(
        10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -100.0, 100.0,
    )
    .unwrap();
    assert!((out - 4.2).abs() < 1e-12);
    assert!((next_i - 0.2).abs() < 1e-12);
}

/// Test: output saturation with anti‑windup (integral freeze)
#[test]
fn test_saturation_antiwindup() {
    // @safety-proof: parameters are valid, function must return Some
    let (out, next_i) = control_pid_step(
        10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -10.0, 3.0,
    )
    .unwrap();
    assert!((out - 3.0).abs() < 1e-12);
    assert!((next_i - 0.0).abs() < 1e-12);
}

/// Test: invalid domain parameters return None
#[test]
fn test_invalid_parameters() {
    assert!(control_pid_step(
        10.0,
        8.0,
        0.0,
        2.0,
        0.0, // dt <= 0
        2.0,
        1.0,
        0.5,
        -10.0,
        10.0
    )
    .is_none());

    assert!(control_pid_step(
        10.0,
        8.0,
        0.0,
        2.0,
        0.1,
        2.0,
        1.0,
        0.5,
        10.0, // out_min >= out_max
        -10.0
    )
    .is_none());
}
