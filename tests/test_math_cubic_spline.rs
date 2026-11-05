// ## Map
// Role: integration test for natural cubic spline segment evaluation
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::math_cubic_spline::*;

#[test]
fn test_math_cubic_spline_golden() {
    // Linear
    let v_lin = math_cubic_spline_eval(1.0, 0.0, 2.0, 10.0, 20.0, 0.0, 0.0).unwrap();
    assert!((v_lin - 15.0).abs() < 1e-10);

    // Parabola y = x^2, m0 = m1 = 2
    let v_mid = math_cubic_spline_eval(1.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((v_mid - 1.0).abs() < 1e-10);

    let v0 = math_cubic_spline_eval(0.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((v0 - 0.0).abs() < 1e-10);

    let v1 = math_cubic_spline_eval(2.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((v1 - 4.0).abs() < 1e-10);
}

#[test]
fn test_math_cubic_spline_derivative() {
    // Parabola y = x^2, y' = 2x
    let d0 = math_cubic_spline_derivative(0.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((d0 - 0.0).abs() < 1e-10);

    let d1 = math_cubic_spline_derivative(1.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((d1 - 2.0).abs() < 1e-10);

    let d2 = math_cubic_spline_derivative(2.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap();
    assert!((d2 - 4.0).abs() < 1e-10);
}

#[test]
fn test_math_cubic_spline_guards() {
    // Out of bounds x > x1
    assert!(math_cubic_spline_eval(2.5, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).is_none());
    // Out of bounds x < x0
    assert!(math_cubic_spline_eval(-0.5, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).is_none());
    // Inverted bounds x1 <= x0
    assert!(math_cubic_spline_eval(1.0, 2.0, 0.0, 0.0, 4.0, 2.0, 2.0).is_none());
    // NaN
    assert!(math_cubic_spline_eval(f64::NAN, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).is_none());
}
