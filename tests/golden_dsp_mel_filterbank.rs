// 本檔由 cloud_farmer 依 specs/dsp_mel_filterbank.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_mel_filterbank::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((hz_to_mel(1000.0).unwrap() - 999.985).abs() < 1e-2, "spec golden vector #1");
    assert!((mel_to_hz(999.985).unwrap() - 1000.0).abs() < 1e-2, "spec golden vector #2");
    assert!(hz_to_mel(-1.0).is_none(), "spec golden vector #3");
    assert!(mel_to_hz(-1.0).is_none(), "spec golden vector #4");
    assert!(hz_to_mel(f64::NAN).is_none(), "spec golden vector #5");
    assert!(mel_filterbank_centers_hz(1, 0.0, 8000.0).is_none(), "spec golden vector #6");
    assert!(mel_filterbank_centers_hz(10, 8000.0, 8000.0).is_none(), "spec golden vector #7");
    let centers = mel_filterbank_centers_hz(10, 0.0, 8000.0).unwrap();
    assert!(centers.len() == 12, "spec golden vector #8");
    assert!((centers[0] - 0.0).abs() < 1e-2, "spec golden vector #9");
    assert!((centers[11] - 8000.0).abs() < 1e-2, "spec golden vector #10");
}
