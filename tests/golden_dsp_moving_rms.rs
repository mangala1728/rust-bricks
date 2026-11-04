// 本檔由 cloud_farmer 依 specs/dsp_moving_rms.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_moving_rms::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((dsp_rms_slice(&[3.0, 3.0, 3.0, 3.0]).unwrap() - 3.0).abs() < 1e-6, "spec golden vector #1");
    let sine: Vec<f64> = (0..8).map(|n| (2.0 * std::f64::consts::PI * n as f64 / 8.0).sin()).collect();
    assert!((dsp_rms_slice(&sine).unwrap() - 0.70710678).abs() < 1e-6, "spec golden vector #2");
    let (s, r) = dsp_moving_rms_step(4.0, 3.0, 9.0, 4).unwrap();
    assert!((s - 16.0).abs() < 1e-6, "spec golden vector #3");
    assert!((r - 2.0).abs() < 1e-6, "spec golden vector #4");
    assert!(dsp_moving_rms_step(4.0, 3.0, 9.0, 0).is_none(), "spec golden vector #5");
    assert!(dsp_moving_rms_step(4.0, 3.0, -1.0, 4).is_none(), "spec golden vector #6");
    assert!(dsp_moving_rms_step(f64::NAN, 3.0, 9.0, 4).is_none(), "spec golden vector #7");
    assert!(dsp_rms_slice(&[]).is_none(), "spec golden vector #8");
    assert!(dsp_rms_slice(&[1.0, f64::INFINITY]).is_none(), "spec golden vector #9");
}
