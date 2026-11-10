// 本檔由 cloud_farmer 依 specs/geo_ecef_to_lla.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_ecef_to_lla::*;

#[test]
fn golden_vectors_from_spec() {
    let (lat0, lon0, alt0) = geo_ecef_to_geodetic_lla(6378137.0, 0.0, 0.0).unwrap();
    assert!(lat0.abs() < 1e-7, "spec golden vector #1");
    assert!(lon0.abs() < 1e-7, "spec golden vector #2");
    assert!(alt0.abs() < 1e-4, "spec golden vector #3");
    assert!(geo_ecef_to_geodetic_lla(0.0, 0.0, 0.0).is_none(), "spec golden vector #4");
    assert!(geo_ecef_to_geodetic_lla(f64::NAN, 0.0, 0.0).is_none(), "spec golden vector #5");
}
