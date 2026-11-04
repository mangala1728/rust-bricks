// 本檔由 cloud_farmer 依 specs/dsp_biquad_bandpass.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_biquad_bandpass::*;

#[test]
fn golden_vectors_from_spec() {
    let c = biquad_bandpass_coefficients(1000.0, 250.0, 1.0).unwrap();
    assert!((c[0] - 0.333333).abs() < 1e-4, "spec golden vector #1");
    assert!((c[1] - 0.0).abs() < 1e-4, "spec golden vector #2");
    assert!((c[2] - (-0.333333)).abs() < 1e-4, "spec golden vector #3");
    assert!((c[3] - 0.0).abs() < 1e-4, "spec golden vector #4");
    assert!((c[4] - 0.333333).abs() < 1e-4, "spec golden vector #5");
    assert!(biquad_bandpass_coefficients(1000.0, 500.0, 1.0).is_none(), "spec golden vector #6");
    assert!(biquad_bandpass_coefficients(1000.0, 250.0, 0.0).is_none(), "spec golden vector #7");
    assert!(biquad_bandpass_coefficients(f64::NAN, 250.0, 1.0).is_none(), "spec golden vector #8");
}
