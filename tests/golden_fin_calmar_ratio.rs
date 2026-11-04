// 本檔由 cloud_farmer 依 specs/fin_calmar_ratio.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_calmar_ratio::*;

#[test]
fn golden_vectors_from_spec() {
    let prices = [100.0, 120.0, 90.0, 110.0, 80.0, 130.0];
    assert!((fin_max_drawdown(&prices).unwrap() - 1.0 / 3.0).abs() < 1e-4, "spec golden vector #1");
    assert!((fin_calmar_ratio(0.20, 0.10).unwrap() - 2.0).abs() < 1e-4, "spec golden vector #2");
}
