// 本檔由 cloud_farmer 依 specs/dsp_window_hamming.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_window_hamming::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((hamming_window_coeff(0, 5).unwrap() - 0.08).abs() < 1e-4, "spec golden vector #1");
    assert!((hamming_window_coeff(2, 5).unwrap() - 1.0).abs() < 1e-4, "spec golden vector #2");
    assert!((hamming_window_coeff(4, 5).unwrap() - 0.08).abs() < 1e-4, "spec golden vector #3");
}
