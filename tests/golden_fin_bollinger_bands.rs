// 本檔由 cloud_farmer 依 specs/fin_bollinger_bands.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_bollinger_bands::*;

#[test]
fn golden_vectors_from_spec() {
    let (mid, up, low, pb, bw) = fin_compute_bollinger_bands(&[10.0, 20.0], 1.0).unwrap();
    assert!((mid - 15.0).abs() < 1e-12, "spec golden vector #1");
    assert!((up - 20.0).abs() < 1e-12, "spec golden vector #2");
    assert!((low - 10.0).abs() < 1e-12, "spec golden vector #3");
    assert!((pb - 1.0).abs() < 1e-12, "spec golden vector #4");
    assert!((bw - (10.0 / 15.0)).abs() < 1e-12, "spec golden vector #5");
    assert!(fin_compute_bollinger_bands(&[10.0], 1.0).is_none(), "spec golden vector #6");
    assert!(fin_compute_bollinger_bands(&[10.0, -5.0], 1.0).is_none(), "spec golden vector #7");
}
