use rust_bricks::math_simpson_integral::*;

#[test]
fn test_simpson_sin_integral() {
    // ∫₀^π sin(x) dx = 2
    let result = simpson_integral_1d(0.0, std::f64::consts::PI, 100, |x| x.sin());
    match result {
        Some(val) => assert!((val - 2.0).abs() < 1e-4, "Expected ≈2.0, got {}", val),
        None => panic!("Integration returned None"),
    }
}

#[test]
fn test_simpson_quadratic_integral() {
    // ∫₀³ x² dx = 9
    let result = simpson_integral_1d(0.0, 3.0, 50, |x| x * x);
    match result {
        Some(val) => assert!((val - 9.0).abs() < 1e-6, "Expected 9.0, got {}", val),
        None => panic!("Integration returned None"),
    }
}

#[test]
fn test_domain_boundary_guards() {
    assert!(simpson_integral_1d(3.0, 0.0, 50, |x| x).is_none());
    assert!(simpson_integral_1d(0.0, 3.0, 1, |x| x).is_none());
    assert!(simpson_integral_1d(0.0, 3.0, 100_001, |x| x).is_none());
}
