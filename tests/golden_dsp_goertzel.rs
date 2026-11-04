// 本檔由 cloud_farmer 依 specs/dsp_goertzel.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_goertzel::*;

#[test]
fn golden_vectors_from_spec() {
    let coeff = goertzel_coeff(1000.0, 8000.0).unwrap();
    assert!((coeff - 1.41421356).abs() < 1e-4, "spec golden vector #1");
    let x: Vec<f64> = (0..8).map(|n| (2.0 * std::f64::consts::PI * 1000.0 * n as f64 / 8000.0).sin()).collect();
    let (p, mag) = goertzel_detect(&x, coeff).unwrap();
    assert!((p - 16.0).abs() < 1e-4, "spec golden vector #2");
    assert!((mag - 4.0).abs() < 1e-4, "spec golden vector #3");
    assert!(goertzel_coeff(4000.0, 8000.0).is_none(), "spec golden vector #4");
    assert!(goertzel_detect(&[], 1.0).is_none(), "spec golden vector #5");
    assert!(goertzel_detect(&[1.0, 2.0], 2.5).is_none(), "spec golden vector #6");
}
