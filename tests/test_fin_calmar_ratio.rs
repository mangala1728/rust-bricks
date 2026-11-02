use rust_bricks::fin_calmar_ratio::*;

/// Verify the golden vector for Maximum Drawdown.
#[test]
fn test_max_drawdown_golden() {
    let prices = [100.0, 120.0, 90.0, 110.0, 80.0, 130.0];
    // @safety-proof
    let dd = fin_max_drawdown(&prices).unwrap();
    let expected = 1.0 / 3.0; // ≈0.333333
    assert!((dd - expected).abs() < 1e-4, "dd = {dd}, expected ≈ {expected}");
}

/// Verify the golden vector for Calmar Ratio.
#[test]
fn test_calmar_ratio_golden() {
    let annualized_return = 0.20_f64;
    let max_drawdown = 0.10_f64;
    // @safety-proof
    let calmar = fin_calmar_ratio(annualized_return, max_drawdown).unwrap();
    let expected = 2.0_f64;
    assert!((calmar - expected).abs() < 1e-4, "calmar = {calmar}, expected = {expected}");
}
