// 本檔由 cloud_farmer 依 specs/fin_ewma_volatility.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_ewma_volatility::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(((ewma_variance_step(0.0004, 0.03, 0.94).unwrap() - 0.00043) / 0.00043).abs() < 1e-9, "spec golden vector #1");
    assert!(((ewma_annualized_volatility(0.00043, 252.0).unwrap() - 0.3291808) / 0.3291808).abs() < 1e-7, "spec golden vector #2");
}
