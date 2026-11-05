// 本檔由 cloud_farmer 依 specs/astro_solar_zenith_angle.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_solar_zenith_angle::*;

#[test]
fn golden_vectors_from_spec() {
    let (z, el) = astro_solar_zenith_and_elevation(81, 12.0, 0.0, 0.0).unwrap();
    assert!((z + el - 90.0).abs() < 1e-10, "spec golden vector #1");
    assert!(z < 5.0, "spec golden vector #2");
    assert!(el > 85.0, "spec golden vector #3");
    assert!(astro_solar_zenith_and_elevation(0, 12.0, 0.0, 0.0).is_none(), "spec golden vector #4");
    assert!(astro_solar_zenith_and_elevation(81, 25.0, 0.0, 0.0).is_none(), "spec golden vector #5");
    assert!(astro_solar_zenith_and_elevation(81, 12.0, 95.0, 0.0).is_none(), "spec golden vector #6");
}
