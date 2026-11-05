// 本檔由 cloud_farmer 依 specs/fin_garch11.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_garch11::*;

#[test]
fn golden_vectors_from_spec() {
    let vl = fin_garch11_unconditional_variance(0.000002, 0.08, 0.90).unwrap();
    assert!((vl - 0.0001).abs() < 1e-8, "spec golden vector #1");
    let v_step = fin_garch11_step(0.000002, 0.08, 0.90, 0.0001, 0.02).unwrap();
    assert!((v_step - 0.000124).abs() < 1e-8, "spec golden vector #2");
    assert!(fin_garch11_step(0.000002, 0.50, 0.60, 0.0001, 0.02).is_none(), "spec golden vector #3");
    assert!(fin_garch11_step(0.0, 0.08, 0.90, 0.0001, 0.02).is_none(), "spec golden vector #4");
    assert!(fin_garch11_step(0.000002, 0.08, 0.90, -0.0001, 0.02).is_none(), "spec golden vector #5");
}
