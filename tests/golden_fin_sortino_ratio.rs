// 本檔由 cloud_farmer 依 specs/fin_sortino_ratio.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_sortino_ratio::*;

#[test]
fn golden_vectors_from_spec() {
    let returns = [0.10, -0.05, 0.15, -0.10];
    assert!((downside_deviation(&returns, 0.0).unwrap() - 0.055901699).abs() < 1e-4, "spec golden vector #1");
    assert!((sortino_ratio(&returns, 0.0).unwrap() - 0.4472136).abs() < 1e-4, "spec golden vector #2");
}
