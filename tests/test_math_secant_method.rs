// ## Map
// Role: integration test for secant method root finder
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::math_secant_method::*;

#[test]
fn test_math_secant_step_golden() {
    let s = math_secant_step(1.0, 3.0, -3.0, 5.0).unwrap();
    assert!((s - 1.75).abs() < 1e-10);

    // Zero slope -> None
    assert!(math_secant_step(1.0, 3.0, 5.0, 5.0).is_none());
}

#[test]
fn test_math_secant_solve_golden() {
    let (root, iters) = math_secant_solve(1.0, 3.0, 1e-6, 50, |x| x * x - 4.0).unwrap();
    assert!((root - 2.0).abs() < 1e-5);
    assert!(iters > 0 && iters <= 20);
}

#[test]
fn test_math_secant_guards() {
    // Coincident initial guesses
    assert!(math_secant_solve(1.0, 1.0, 1e-6, 50, |x| x * x - 4.0).is_none());
    // Negative tolerance
    assert!(math_secant_solve(1.0, 3.0, -1e-6, 50, |x| x * x - 4.0).is_none());
    // Zero max_iter
    assert!(math_secant_solve(1.0, 3.0, 1e-6, 0, |x| x * x - 4.0).is_none());
    // NaN
    assert!(math_secant_step(f64::NAN, 3.0, -3.0, 5.0).is_none());
    assert!(math_secant_solve(f64::NAN, 3.0, 1e-6, 50, |x| x * x - 4.0).is_none());
}
