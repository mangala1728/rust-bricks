// 本檔由 cloud_farmer 依 specs/fin_hurst_exponent_rs.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_hurst_exponent_rs::*;

#[test]
fn golden_vectors_from_spec() {
    let s = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
    let (rs, h) = fin_compute_hurst_rs(&s).unwrap();
    assert!(rs > 1.0, "spec golden vector #1");
    assert!(h > 0.0 && h <= 1.0, "spec golden vector #2");
    assert!(fin_compute_hurst_rs(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]).is_none(), "spec golden vector #3");
    assert!(fin_compute_hurst_rs(&[1.0, 2.0, 3.0]).is_none(), "spec golden vector #4");
}
