// 本檔由 cloud_farmer 依 specs/sensor_ppg_spo2.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_ppg_spo2::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((ppg_estimate_spo2(0.5).unwrap() - 98.757).abs() < 1e-3, "spec golden vector #1");
    assert!((ppg_estimate_spo2(1.0).unwrap() - 80.139).abs() < 1e-3, "spec golden vector #2");
    assert!((ppg_estimate_spo2(3.0).unwrap() - 0.0).abs() < 1e-3, "spec golden vector #3");
    assert!(ppg_ratio_of_ratios(0.02, 0.0, 0.04, 1.0).is_none(), "spec golden vector #4");
    assert!(ppg_ratio_of_ratios(0.02, 1.0, 0.04, 0.0).is_none(), "spec golden vector #5");
    assert!(ppg_ratio_of_ratios(0.02, 1.0, 0.0, 1.0).is_none(), "spec golden vector #6");
}
