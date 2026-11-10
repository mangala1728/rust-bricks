// 本檔由 cloud_farmer 依 specs/fin_drawdown_recovery.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_drawdown_recovery::*;

#[test]
fn golden_vectors_from_spec() {
    let s = [100.0, 120.0, 90.0, 110.0, 130.0, 104.0];
    let (mdd, uw, curr) = fin_compute_drawdown_metrics(&s).unwrap();
    assert!((mdd - 25.0).abs() < 1e-12, "spec golden vector #1");
    assert!(uw == 2, "spec golden vector #2");
    assert!((curr - 20.0).abs() < 1e-12, "spec golden vector #3");
    assert!(fin_compute_drawdown_metrics(&[]).is_none(), "spec golden vector #4");
    assert!(fin_compute_drawdown_metrics(&[100.0, -50.0]).is_none(), "spec golden vector #5");
}
