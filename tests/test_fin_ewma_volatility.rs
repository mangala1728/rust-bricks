use rust_bricks::fin_ewma_volatility::*;

#[test]
fn test_ewma_variance_step_golden_vector() {
    let prev_variance = 0.0004;
    let current_return = 0.03;
    let lambda = 0.94;

    let res = ewma_variance_step(prev_variance, current_return, lambda);
    assert!(res.is_some());
    if let Some(var) = res {
        let golden_var = 0.000430;
        assert!((var - golden_var).abs() < 1e-4);
    }
}

#[test]
fn test_ewma_annualized_volatility_golden_vector() {
    let daily_variance = 0.000430;
    let trading_days = 252.0;

    let res = ewma_annualized_volatility(daily_variance, trading_days);
    assert!(res.is_some());
    if let Some(vol) = res {
        let golden_vol = 0.3291808;
        assert!((vol - golden_vol).abs() < 1e-4);
    }
}

#[test]
fn test_domain_boundary_guards() {
    // 測試 Out-of-domain 數值觸發 Guard
    assert!(ewma_variance_step(-0.0001, 0.03, 0.94).is_none());
    assert!(ewma_variance_step(1.0e13, 0.03, 0.94).is_none());
    assert!(ewma_variance_step(0.0004, -1.5, 0.94).is_none());
    assert!(ewma_variance_step(0.0004, 11.0, 0.94).is_none());
    assert!(ewma_variance_step(0.0004, 0.03, 0.499).is_none());
    assert!(ewma_variance_step(0.0004, 0.03, 1.0).is_none());

    assert!(ewma_annualized_volatility(-0.0001, 252.0).is_none());
    assert!(ewma_annualized_volatility(1.0e13, 252.0).is_none());
    assert!(ewma_annualized_volatility(0.0004, 199.9).is_none());
    assert!(ewma_annualized_volatility(0.0004, 365.1).is_none());
}
