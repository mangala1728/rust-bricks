// 本檔由 cloud_farmer 依 specs/sensor_fall_detection.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_fall_detection::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((sensor_svm_magnitude(0.0, 0.0, 9.8).unwrap() - 9.8).abs() < 1e-6, "spec golden vector #1");
    assert!(sensor_svm_magnitude(f64::NAN, 0.0, 9.8).is_none(), "spec golden vector #2");
    assert!(sensor_fall_detect_impact(3.0, 5.0, 25.0, false) == Some((true, false)), "spec golden vector #3");
    assert!(sensor_fall_detect_impact(28.0, 5.0, 25.0, true) == Some((false, true)), "spec golden vector #4");
    assert!(sensor_fall_detect_impact(9.8, 5.0, 25.0, false) == Some((false, false)), "spec golden vector #5");
    assert!(sensor_fall_detect_impact(-1.0, 5.0, 25.0, false).is_none(), "spec golden vector #6");
    assert!(sensor_fall_detect_impact(10.0, 25.0, 5.0, false).is_none(), "spec golden vector #7");
}
