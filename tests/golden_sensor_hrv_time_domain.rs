// 本檔由 cloud_farmer 依 specs/sensor_hrv_time_domain.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::sensor_hrv_time_domain::*;

#[test]
fn golden_vectors_from_spec() {
    let hrv = hrv_time_domain_metrics(&[800.0, 810.0, 890.0, 820.0, 900.0]).unwrap();
    assert!((hrv.0 - 844.0).abs() < 1e-4, "spec golden vector #1");
    assert!((hrv.1 - 42.2374).abs() < 1e-4, "spec golden vector #2");
    assert!((hrv.2 - 66.7083).abs() < 1e-4, "spec golden vector #3");
    assert!((hrv.3 - 75.0).abs() < 1e-4, "spec golden vector #4");
    assert!(hrv_time_domain_metrics(&[800.0]).is_none(), "spec golden vector #5");
    assert!(hrv_time_domain_metrics(&[800.0, 199.0, 820.0]).is_none(), "spec golden vector #6");
    assert!(hrv_time_domain_metrics(&[800.0, 3001.0, 820.0]).is_none(), "spec golden vector #7");
}
