// ## Map
// Role: integration test for analytical 2x2 matrix SVD
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::math_svd_2x2::*;

#[test]
fn test_math_svd_2x2_golden() {
    let (s1, s2) = math_svd_2x2_singular_values(1.0, 0.0, 0.0, 1.0).unwrap();
    assert!((s1 - 1.0).abs() < 1e-10 && (s2 - 1.0).abs() < 1e-10);
    assert!((math_matrix_2x2_condition_number(1.0, 0.0, 0.0, 1.0).unwrap() - 1.0).abs() < 1e-10);

    let (s1, s2) = math_svd_2x2_singular_values(3.0, 0.0, 0.0, 2.0).unwrap();
    assert!((s1 - 3.0).abs() < 1e-10 && (s2 - 2.0).abs() < 1e-10);
    assert!((math_matrix_2x2_condition_number(3.0, 0.0, 0.0, 2.0).unwrap() - 1.5).abs() < 1e-10);

    let (s1, s2) = math_svd_2x2_singular_values(1.0, 2.0, 2.0, 4.0).unwrap();
    assert!((s1 - 5.0).abs() < 1e-10 && s2.abs() < 1e-10);
    assert!(math_matrix_2x2_condition_number(1.0, 2.0, 2.0, 4.0).is_none());
}

#[test]
fn test_math_svd_2x2_guards() {
    assert!(math_svd_2x2_singular_values(f64::NAN, 0.0, 0.0, 1.0).is_none());
    assert!(math_matrix_2x2_condition_number(f64::NAN, 0.0, 0.0, 1.0).is_none());
    assert!(math_svd_2x2_singular_values(1.0, f64::INFINITY, 0.0, 1.0).is_none());
}
