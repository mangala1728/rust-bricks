// 本檔由 cloud_farmer 依 specs/dsp_window_hann.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_window_hann::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((hann_window_coeff(0, 5).unwrap() - 0.0).abs() < 1e-6, "spec golden vector #1");
    assert!((hann_window_coeff(2, 5).unwrap() - 1.0).abs() < 1e-6, "spec golden vector #2");
    assert!((hann_window_coeff(0, 1).unwrap() - 1.0).abs() < 1e-6, "spec golden vector #3");
    assert!(hann_window_coeff(5, 5).is_none(), "spec golden vector #4");
    assert!(hann_window_coeff(0, 0).is_none(), "spec golden vector #5");
}
