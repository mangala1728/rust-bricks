// 本檔由 cloud_farmer 依 specs/geo_haversine_waypoint.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_haversine_waypoint::*;

#[test]
fn golden_vectors_from_spec() {
    let (d0, _) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 0.0).unwrap();
    assert!(d0.abs() < 1e-6, "spec golden vector #1");
    let (d1, b1) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 1.0).unwrap();
    assert!((d1 - 111194.9266).abs() < 1.0, "spec golden vector #2");
    assert!((b1 - 90.0).abs() < 1e-6, "spec golden vector #3");
    assert!(geo_haversine_distance_bearing(95.0, 0.0, 0.0, 0.0).is_none(), "spec golden vector #4");
    assert!(geo_haversine_distance_bearing(0.0, -190.0, 0.0, 0.0).is_none(), "spec golden vector #5");
}
