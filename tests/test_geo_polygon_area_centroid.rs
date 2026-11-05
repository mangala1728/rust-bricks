// ## Map
// Role: integration test for polygon shoelace signed area and centroid
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::geo_polygon_area_centroid::*;

#[test]
fn test_geo_polygon_area_centroid_golden() {
    let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
    assert!((geo_polygon_signed_area(&sq).unwrap() - 1.0).abs() < 1e-10);
    let (cx, cy) = geo_polygon_centroid(&sq).unwrap();
    assert!((cx - 0.5).abs() < 1e-10 && (cy - 0.5).abs() < 1e-10);

    let tri = [(0.0, 0.0), (3.0, 0.0), (0.0, 3.0)];
    assert!((geo_polygon_signed_area(&tri).unwrap() - 4.5).abs() < 1e-10);
    let (tcx, tcy) = geo_polygon_centroid(&tri).unwrap();
    assert!((tcx - 1.0).abs() < 1e-10 && (tcy - 1.0).abs() < 1e-10);
}

#[test]
fn test_geo_polygon_area_centroid_guards() {
    assert!(geo_polygon_signed_area(&[(0.0, 0.0), (1.0, 0.0)]).is_none());
    assert!(geo_polygon_centroid(&[(0.0, 0.0), (1.0, 0.0)]).is_none());

    // Degenerate collinear points -> zero area -> centroid is None
    let line = [(0.0, 0.0), (1.0, 1.0), (2.0, 2.0)];
    assert!((geo_polygon_signed_area(&line).unwrap()).abs() < 1e-10);
    assert!(geo_polygon_centroid(&line).is_none());

    // Non-finite vertex
    assert!(geo_polygon_signed_area(&[(0.0, 0.0), (f64::NAN, 1.0), (1.0, 1.0)]).is_none());
}
