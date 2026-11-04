// 本檔由 cloud_farmer 依 specs/fin_kelly_criterion.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_kelly_criterion::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((kelly_optimal_fraction(0.60, 1.0, 1.0).unwrap() - 0.20).abs() < 1e-4, "spec golden vector #1");
    assert!((kelly_optimal_fraction(0.60, 1.0, 0.5).unwrap() - 0.10).abs() < 1e-4, "spec golden vector #2");
    assert!(kelly_optimal_fraction(0.40, 1.0, 1.0) == Some(0.0), "spec golden vector #3");
}
