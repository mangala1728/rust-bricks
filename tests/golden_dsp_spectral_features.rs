// 本檔由 cloud_farmer 依 specs/dsp_spectral_features.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_spectral_features::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((spectral_centroid(&[1.0, 2.0, 4.0, 2.0, 1.0], 1000.0).unwrap() - 250.0).abs() < 1e-4, "spec golden vector #1");
    assert!((spectral_rolloff(&[1.0, 2.0, 4.0, 2.0, 1.0], 1000.0, 0.85).unwrap() - 375.0).abs() < 1e-4, "spec golden vector #2");
    assert!(spectral_centroid(&[1.0], 1000.0).is_none(), "spec golden vector #3");
    assert!(spectral_centroid(&[0.0, 0.0, 0.0], 1000.0).is_none(), "spec golden vector #4");
}
