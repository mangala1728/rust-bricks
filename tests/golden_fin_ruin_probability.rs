// 本檔由 cloud_farmer 依 specs/fin_ruin_probability.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_ruin_probability::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((fin_ruin_probability_exponential(0.0, 0.25, 100.0).unwrap() - 0.80).abs() < 1e-10, "spec golden vector #1");
    assert!((fin_ruin_probability_exponential(500.0, 0.25, 100.0).unwrap() - 0.2943036).abs() < 1e-6, "spec golden vector #2");
    assert!(fin_ruin_probability_exponential(-10.0, 0.25, 100.0).is_none(), "spec golden vector #3");
    assert!(fin_ruin_probability_exponential(100.0, 0.0, 100.0).is_none(), "spec golden vector #4");
    assert!(fin_ruin_probability_exponential(100.0, 0.25, 0.0).is_none(), "spec golden vector #5");
}
