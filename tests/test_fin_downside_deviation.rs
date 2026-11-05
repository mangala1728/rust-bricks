// ## Map
// Role: integration test for downside semi-deviation and upside potential ratio
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::fin_downside_deviation::*;

#[test]
fn test_fin_downside_deviation_golden() {
    let r = [0.10, -0.05, 0.05, -0.10];
    let down = fin_downside_deviation(&r, 0.0).unwrap();
    assert!((down - 0.0559017).abs() < 1e-5);

    let up = fin_upside_potential_ratio(&r, 0.0).unwrap();
    assert!((up - 0.670820).abs() < 1e-5);
}

#[test]
fn test_fin_downside_deviation_guards() {
    assert!(fin_downside_deviation(&[], 0.0).is_none());
    assert!(fin_downside_deviation(&[0.05, f64::NAN], 0.0).is_none());
    assert!(fin_downside_deviation(&[0.05], f64::NAN).is_none());

    // All returns above MAR -> zero downside deviation -> ratio is None
    assert!(fin_upside_potential_ratio(&[0.10, 0.20], 0.0).is_none());
}
