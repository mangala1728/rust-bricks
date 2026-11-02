// ## Map: Role=IntegrationTest, Inputs=module_functions, Outputs=assertions, Invariants=deterministic_golden_values

use rust_bricks::math_newton_raphson_1d::*;

#[test]
fn test_sqrt_two() {
    // f(x) = x^2 - 2, f'(x) = 2x
    let f = |x: f64| x * x - 2.0;
    let df = |x: f64| 2.0 * x;

    let root = newton_raphson_1d(1.0, 1.0e-6, 20, f, df);
    // Golden value √2 ≈ 1.41421356
    assert!(match root {
        Some(v) => (v - 1.41421356_f64).abs() < 1.0e-4,
        None => false,
    });
}

#[test]
fn test_dottie_number() {
    // f(x) = cos(x) - x, f'(x) = -sin(x) - 1
    let f = |x: f64| x.cos() - x;
    let df = |x: f64| -x.sin() - 1.0;

    let root = newton_raphson_1d(0.5, 1.0e-6, 20, f, df);
    // Golden value ≈ 0.739085
    assert!(match root {
        Some(v) => (v - 0.739085_f64).abs() < 1.0e-4,
        None => false,
    });
}

#[test]
fn test_domain_boundary_guards() {
    let f = |x: f64| x * x - 2.0;
    let df = |x: f64| 2.0 * x;

    assert!(newton_raphson_1d(f64::NAN, 1.0e-6, 20, f, df).is_none());
    assert!(newton_raphson_1d(1.0, 1.0e-13, 20, f, df).is_none());
    assert!(newton_raphson_1d(1.0, 1.5, 20, f, df).is_none());
    assert!(newton_raphson_1d(1.0, 1.0e-6, 0, f, df).is_none());
    assert!(newton_raphson_1d(1.0, 1.0e-6, 1001, f, df).is_none());

    // Zero derivative
    let f_flat = |_: f64| 5.0;
    let df_flat = |_: f64| 0.0;
    assert!(newton_raphson_1d(1.0, 1.0e-6, 20, f_flat, df_flat).is_none());
}
