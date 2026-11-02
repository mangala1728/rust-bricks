use rust_bricks::fin_sortino_ratio::*;

#[test]
fn golden_sortino_ratio() {
    // Golden vector from the specification
    let returns = [0.10_f64, -0.05, 0.15, -0.10];
    let risk_free = 0.0_f64;

    // Expected values (computed in the spec)
    let expected_deviation = 0.055901699_f64; // ≈ sqrt(0.003125)
    let expected_ratio = 0.4472136_f64;       // ≈ 0.025 / expected_deviation
    let eps = 1e-4_f64;

    // Downside deviation
    let dd = downside_deviation(&returns, risk_free)
        .expect("downside deviation should be defined");
    assert!((dd - expected_deviation).abs() < eps,
        "downside deviation mismatch: got {dd}, expected {expected_deviation}");

    // Sortino ratio
    let sr = sortino_ratio(&returns, risk_free)
        .expect("sortino ratio should be defined");
    assert!((sr - expected_ratio).abs() < eps,
        "sortino ratio mismatch: got {sr}, expected {expected_ratio}");
}

#[test]
fn test_domain_boundary_guards() {
    let returns = [0.10_f64, -0.05, 0.15, -0.10];
    assert!(downside_deviation(&returns, -1.1).is_none());
    assert!(downside_deviation(&returns, 10.1).is_none());
    assert!(downside_deviation(&[], 0.0).is_none());

    assert!(sortino_ratio(&returns, -1.1).is_none());
    assert!(sortino_ratio(&returns, 10.1).is_none());
    assert!(sortino_ratio(&[], 0.0).is_none());

    // When returns are all strictly above target return, downside deviation is 0 -> sortino_ratio returns None
    let positive_returns = [0.05, 0.10, 0.15];
    assert!(sortino_ratio(&positive_returns, 0.0).is_none());
}
