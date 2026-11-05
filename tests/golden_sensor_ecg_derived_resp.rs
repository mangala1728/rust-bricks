// 本檔由 cloud_farmer 依 specs/sensor_ecg_derived_resp.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_ecg_derived_resp::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((sensor_edr_r_peak_amplitude(&[0.1, 1.5, 0.2], 1, 0.2).unwrap() - 1.3).abs() < 1e-10, "spec golden vector #1");
    assert!(sensor_edr_r_peak_amplitude(&[0.1, 1.5], 2, 0.2).is_none(), "spec golden vector #2");
    assert!((sensor_respiration_rate_from_intervals(&[4.0, 4.0, 4.0]).unwrap() - 15.0).abs() < 1e-10, "spec golden vector #3");
    assert!((sensor_respiration_rate_from_intervals(&[3.0, 3.0]).unwrap() - 20.0).abs() < 1e-10, "spec golden vector #4");
    assert!(sensor_respiration_rate_from_intervals(&[4.0]).is_none(), "spec golden vector #5");
    assert!(sensor_respiration_rate_from_intervals(&[0.5, 4.0]).is_none(), "spec golden vector #6");
    assert!(sensor_respiration_rate_from_intervals(&[15.0, 4.0]).is_none(), "spec golden vector #7");
}
