// 本檔由 cloud_farmer 依 specs/dsp_stft_ola_denoiser.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_stft_ola_denoiser::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.05) == 0.8, "spec golden vector #1");
    assert!(spectral_subtraction_gain_single(10.0, 10.0, 2.0, 0.05) == 0.05, "spec golden vector #2");
    assert!(spectral_subtraction_gain_single(0.0, 10.0, 2.0, 0.05) == 0.05, "spec golden vector #3");
    assert!(spectral_subtraction_gain_single(100.0, 0.0, 2.0, 0.05) == 1.0, "spec golden vector #4");
    assert!(spectral_subtraction_gain_single(100.0, 10.0, 0.5, 0.05) == 1.0, "spec golden vector #5");
    assert!(spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.6) == 1.0, "spec golden vector #6");
}
