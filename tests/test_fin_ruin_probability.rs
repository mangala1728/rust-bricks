// ## Map
// Role: integration test for Cramer-Lundberg ruin probability model
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::fin_ruin_probability::*;

#[test]
fn test_fin_ruin_probability_golden() {
    let p0 = fin_ruin_probability_exponential(0.0, 0.25, 100.0).unwrap();
    assert!((p0 - 0.80).abs() < 1e-10);

    let p500 = fin_ruin_probability_exponential(500.0, 0.25, 100.0).unwrap();
    assert!((p500 - 0.2943036).abs() < 1e-6);
}

#[test]
fn test_fin_ruin_probability_guards() {
    assert!(fin_ruin_probability_exponential(-10.0, 0.25, 100.0).is_none());
    assert!(fin_ruin_probability_exponential(100.0, 0.0, 100.0).is_none());
    assert!(fin_ruin_probability_exponential(100.0, -0.1, 100.0).is_none());
    assert!(fin_ruin_probability_exponential(100.0, 0.25, 0.0).is_none());
    assert!(fin_ruin_probability_exponential(100.0, 0.25, -1.0).is_none());
    assert!(fin_ruin_probability_exponential(f64::NAN, 0.25, 100.0).is_none());
}
