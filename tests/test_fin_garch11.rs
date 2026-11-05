// ## Map
// Role: integration test for GARCH(1,1) model
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::fin_garch11::*;

#[test]
fn test_fin_garch11_golden() {
    let vl = fin_garch11_unconditional_variance(0.000002, 0.08, 0.90).unwrap();
    assert!((vl - 0.0001).abs() < 1e-8);

    let v_step = fin_garch11_step(0.000002, 0.08, 0.90, 0.0001, 0.02).unwrap();
    assert!((v_step - 0.000124).abs() < 1e-8);
}

#[test]
fn test_fin_garch11_guards() {
    // Non-stationary
    assert!(fin_garch11_step(0.000002, 0.50, 0.60, 0.0001, 0.02).is_none());
    assert!(fin_garch11_unconditional_variance(0.000002, 0.50, 0.50).is_none());

    // Non-positive omega
    assert!(fin_garch11_step(0.0, 0.08, 0.90, 0.0001, 0.02).is_none());
    assert!(fin_garch11_unconditional_variance(-0.001, 0.08, 0.90).is_none());

    // Negative prev_var
    assert!(fin_garch11_step(0.000002, 0.08, 0.90, -0.0001, 0.02).is_none());

    // NaN
    assert!(fin_garch11_step(f64::NAN, 0.08, 0.90, 0.0001, 0.02).is_none());
}

#[test]
fn test_fin_garch11_series() {
    let returns = [0.02, -0.01, 0.03];
    let mut vars = [0.0; 3];
    let n = fin_garch11_series(0.000002, 0.08, 0.90, 0.0001, &returns, &mut vars);
    assert_eq!(n, 3);
    assert!((vars[0] - 0.000124).abs() < 1e-8);
}
