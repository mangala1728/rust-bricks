// tests/test_fin_drawdown_recovery.rs
use rust_bricks::fin_drawdown_recovery::*;

#[test]
fn test_golden_values() {
    let s = [100.0, 120.0, 90.0, 110.0, 130.0, 104.0];
    let (mdd, uw, curr) = fin_compute_drawdown_metrics(&s).unwrap();
    assert!((mdd - 25.0).abs() < 1e-12);
    assert_eq!(uw, 2);
    assert!((curr - 20.0).abs() < 1e-12);
}

#[test]
fn test_invalid_inputs() {
    assert!(fin_compute_drawdown_metrics(&[]).is_none());
    assert!(fin_compute_drawdown_metrics(&[100.0, -50.0]).is_none());
}
