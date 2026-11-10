use rust_bricks::fin_macd_oscillator::*;

#[test]
fn test_fin_macd_step_golden() {
    // @safety-proof: Golden vector values are hardcoded within valid domain bounds.
    let res = fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).unwrap();
    assert!((res.0 - 11.0).abs() < 1e-12);
    assert!((res.1 - 8.0).abs() < 1e-12);
    assert!((res.2 - 3.0).abs() < 1e-12);
    assert!((res.3 - 95.0).abs() < 1e-12);
    assert!((res.4 - 84.0).abs() < 1e-12);

    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.2, 0.5, 0.5).is_none());
    assert!(fin_macd_step(-10.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).is_none());
}

#[test]
fn test_fin_macd_step_boundary_guards() {
    assert!(fin_macd_step(f64::NAN, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).is_none());
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.5, 0.5).is_none());
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 1.1, 0.2, 0.5).is_none());
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.0, 0.5).is_none());
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.0).is_none());
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.2, 1.5).is_none());
}
