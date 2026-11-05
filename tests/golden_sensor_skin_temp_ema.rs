// 本檔由 cloud_farmer 依 specs/sensor_skin_temp_ema.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_skin_temp_ema::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((sensor_skin_temp_smooth(37.0, 36.0, 0.2).unwrap() - 36.2).abs() < 1e-10, "spec golden vector #1");
    assert!(sensor_skin_temp_smooth(55.0, 36.0, 0.2).is_none(), "spec golden vector #2");
    assert!(sensor_skin_temp_smooth(37.0, 36.0, 1.2).is_none(), "spec golden vector #3");
    assert!(sensor_fever_triage(36.8, 37.3, 38.5) == Some(0), "spec golden vector #4");
    assert!(sensor_fever_triage(37.5, 37.3, 38.5) == Some(1), "spec golden vector #5");
    assert!(sensor_fever_triage(39.2, 37.3, 38.5) == Some(2), "spec golden vector #6");
    assert!(sensor_fever_triage(20.0, 37.3, 38.5).is_none(), "spec golden vector #7");
    assert!(sensor_fever_triage(37.0, 39.0, 38.0).is_none(), "spec golden vector #8");
}
