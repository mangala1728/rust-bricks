// 本檔由 cloud_farmer 依 specs/geo_polygon_area_centroid.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::geo_polygon_area_centroid::*;

#[test]
fn golden_vectors_from_spec() {
    let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    assert!((geo_polygon_signed_area(&sq).unwrap() - 1.0).abs() < 1e-10, "spec golden vector #1");
    let (cx, cy) = geo_polygon_centroid(&sq).unwrap();
    assert!((cx - 0.5).abs() < 1e-10 && (cy - 0.5).abs() < 1e-10, "spec golden vector #2");
    let tri = [(0.0, 0.0), (3.0, 0.0), (0.0, 3.0)];
    assert!((geo_polygon_signed_area(&tri).unwrap() - 4.5).abs() < 1e-10, "spec golden vector #3");
    let (tcx, tcy) = geo_polygon_centroid(&tri).unwrap();
    assert!((tcx - 1.0).abs() < 1e-10 && (tcy - 1.0).abs() < 1e-10, "spec golden vector #4");
    assert!(geo_polygon_signed_area(&[(0.0, 0.0), (1.0, 0.0)]).is_none(), "spec golden vector #5");
}
