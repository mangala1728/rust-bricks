// ## Map
// Role: integration test for bisection implied volatility solver
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::fin_implied_volatility::*;

#[test]
fn test_fin_implied_volatility_golden() {
    let p = fin_bs_call_price(100.0, 100.0, 0.05, 1.0, 0.20).unwrap();
    assert!((p - 10.45058).abs() < 1e-4);

    let iv = fin_implied_volatility_bisection(10.45058, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).unwrap();
    assert!((iv - 0.20).abs() < 1e-3);
}

#[test]
fn test_fin_implied_volatility_guards() {
    // Price exceeds stock price (arbitrage upper bound violation)
    assert!(fin_implied_volatility_bisection(150.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none());

    // Negative market price
    assert!(fin_implied_volatility_bisection(-1.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none());

    // Zero / invalid params
    assert!(fin_implied_volatility_bisection(10.0, 0.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none());
    assert!(fin_implied_volatility_bisection(10.0, 100.0, 0.0, 0.05, 1.0, 1e-5, 50).is_none());
    assert!(fin_implied_volatility_bisection(10.0, 100.0, 100.0, 0.05, 0.0, 1e-5, 50).is_none());
    assert!(fin_implied_volatility_bisection(10.0, 100.0, 100.0, 0.05, 1.0, 0.0, 50).is_none());
    assert!(fin_implied_volatility_bisection(10.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 0).is_none());
}
