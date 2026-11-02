// ## Map
// Role: integration test for point_in_polygon_ray
// Inputs: various points & polygons
// Outputs: deterministic assertions (golden values)
// Invariants: no use of super::*, only public API

use rust_bricks::geo_point_in_polygon_ray::*;

/// Helper to build the L‑shaped polygon used in several golden tests.
fn l_shape() -> Vec<(f64, f64)> {
    vec![
        (0.0, 0.0),
        (6.0, 0.0),
        (6.0, 3.0),
        (3.0, 3.0),
        (3.0, 6.0),
        (0.0, 6.0),
    ]
}

#[test]
fn inside_l_shape_bottom() {
    let poly = l_shape();
    let pt = (1.0, 1.0);
    assert_eq!(point_in_polygon_ray(pt, &poly), Some(true));
}

#[test]
fn outside_l_shape_hole() {
    let poly = l_shape();
    let pt = (4.0, 4.0);
    assert_eq!(point_in_polygon_ray(pt, &poly), Some(false));
}

#[test]
fn far_away_point() {
    let poly = l_shape();
    let pt = (10.0, 10.0);
    assert_eq!(point_in_polygon_ray(pt, &poly), Some(false));
}

#[test]
fn degenerate_polygon_returns_none() {
    let poly = vec![(0.0, 0.0), (1.0, 1.0)]; // only 2 vertices
    let pt = (0.5, 0.5);
    assert_eq!(point_in_polygon_ray(pt, &poly), None);
}
