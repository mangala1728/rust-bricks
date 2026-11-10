use rust_bricks::control_sliding_mode_1d::*;

#[test]
fn golden_values_inside_boundary() {
    // error = 1.0, error_dot = -0.5, c = 1.0 → s = 0.5
    // k = 10.0, epsilon = 1.0 → norm_s = 0.5 → sat = 0.5 → u = 5.0
    let (s, u) = control_sliding_mode_step(1.0, -0.5, 1.0, 10.0, 1.0).unwrap();
    assert!((s - 0.5).abs() < 1e-12, "s = {s}, expected 0.5");
    assert!((u - 5.0).abs() < 1e-12, "u = {u}, expected 5.0");
}

#[test]
fn golden_values_outside_boundary() {
    // error = 2.0, error_dot = 1.0, c = 1.0 → s = 3.0
    // epsilon = 1.0 → norm_s = 3.0 → sat saturated to 1.0 → u = 10.0
    let (s, u) = control_sliding_mode_step(2.0, 1.0, 1.0, 10.0, 1.0).unwrap();
    assert!((s - 3.0).abs() < 1e-12, "s = {s}, expected 3.0");
    assert!((u - 10.0).abs() < 1e-12, "u = {u}, expected 10.0");
}

#[test]
fn rejects_invalid_parameters() {
    // c <= 0.0
    assert!(control_sliding_mode_step(1.0, 0.0, 0.0, 10.0, 1.0).is_none());

    // epsilon <= 0.0
    assert!(control_sliding_mode_step(1.0, 0.0, 1.0, 10.0, 0.0).is_none());

    // non‑finite inputs
    assert!(control_sliding_mode_step(f64::NAN, 0.0, 1.0, 10.0, 1.0).is_none());
    assert!(control_sliding_mode_step(1.0, f64::INFINITY, 1.0, 10.0, 1.0).is_none());
}
