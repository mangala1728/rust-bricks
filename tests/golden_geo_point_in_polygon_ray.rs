// 本檔由 cloud_farmer 依 specs/geo_point_in_polygon_ray.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_point_in_polygon_ray::*;

#[test]
fn golden_vectors_from_spec() {
    let l = [(0.0, 0.0), (6.0, 0.0), (6.0, 3.0), (3.0, 3.0), (3.0, 6.0), (0.0, 6.0)];
    assert!(point_in_polygon_ray((1.0, 1.0), &l) == Some(true), "spec golden vector #1");
    assert!(point_in_polygon_ray((4.0, 4.0), &l) == Some(false), "spec golden vector #2");
    assert!(point_in_polygon_ray((10.0, 10.0), &l) == Some(false), "spec golden vector #3");
    assert!(point_in_polygon_ray((1.0, 1.0), &[(0.0, 0.0), (6.0, 0.0)]).is_none(), "spec golden vector #4");
    assert!(point_in_polygon_ray((f64::NAN, 1.0), &l).is_none(), "spec golden vector #5");
    assert!(point_in_polygon_ray((1.0, 1.0), &[(0.0, 0.0), (6.0, f64::INFINITY), (0.0, 6.0)]).is_none(), "spec golden vector #6");
}
