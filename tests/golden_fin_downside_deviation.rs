// 本檔由 cloud_farmer 依 specs/fin_downside_deviation.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_downside_deviation::*;

#[test]
fn golden_vectors_from_spec() {
    let r = [0.10, -0.05, 0.05, -0.10];
    assert!((fin_downside_deviation(&r, 0.0).unwrap() - 0.0559017).abs() < 1e-5, "spec golden vector #1");
    assert!((fin_upside_potential_ratio(&r, 0.0).unwrap() - 0.670820).abs() < 1e-5, "spec golden vector #2");
    assert!(fin_downside_deviation(&[], 0.0).is_none(), "spec golden vector #3");
    assert!(fin_downside_deviation(&[0.05, f64::NAN], 0.0).is_none(), "spec golden vector #4");
    assert!(fin_upside_potential_ratio(&[0.10, 0.20], 0.0).is_none(), "spec golden vector #5");
}
