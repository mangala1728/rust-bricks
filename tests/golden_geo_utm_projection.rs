// 本檔由 cloud_farmer 依 specs/geo_utm_projection.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_utm_projection::*;

#[test]
fn golden_vectors_from_spec() {
    let (z1, h1, cm1) = geo_determine_utm_zone(25.0, 121.5).unwrap();
    assert!(z1 == 51, "spec golden vector #1");
    assert!(h1 == 'N', "spec golden vector #2");
    assert!((cm1 - 123.0).abs() < 1e-12, "spec golden vector #3");
    let (z2, h2, cm2) = geo_determine_utm_zone(51.5, 0.0).unwrap();
    assert!(z2 == 31, "spec golden vector #4");
    assert!(h2 == 'N', "spec golden vector #5");
    assert!((cm2 - 3.0).abs() < 1e-12, "spec golden vector #6");
    assert!(geo_determine_utm_zone(85.0, 0.0).is_none(), "spec golden vector #7");
    assert!(geo_determine_utm_zone(-81.0, 0.0).is_none(), "spec golden vector #8");
    assert!(geo_determine_utm_zone(0.0, 185.0).is_none(), "spec golden vector #9");
}
