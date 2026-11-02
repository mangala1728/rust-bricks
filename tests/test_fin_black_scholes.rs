// ## Map
// Role: Integration test for Black-Scholes analytical option pricing
// Inputs: none (uses public API)
// Outputs: test assertions
// Invariants: golden values verified against John Hull's benchmark textbook

use rust_bricks::fin_black_scholes::*;

#[test]
fn test_black_scholes_golden() {
    // Hull's classic example
    let s0 = 100.0;
    let k = 100.0;
    let r = 0.05;
    let sigma = 0.20;
    let t = 1.0;

    let (call, put, call_delta, put_delta) = black_scholes_european(s0, k, r, sigma, t)
        .expect("inputs are within domain");

    let tol = 1e-3;
    assert!((call - 10.4506).abs() < tol, "call price mismatch");
    assert!((put - 5.5735).abs() < tol, "put price mismatch");
    assert!((call_delta - 0.6368).abs() < tol, "call delta mismatch");
    assert!((put_delta + 0.3632).abs() < tol, "put delta mismatch");
}
