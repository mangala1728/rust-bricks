// ## Map
// Role: Integration test for Haversine distance & bearing implementation
// Inputs: N/A
// Outputs: N/A
// Invariants: zero external deps, deterministic golden values

use rust_bricks::geo_haversine_waypoint::geo_haversine_distance_bearing;

/// Golden‑value test – identical points (zero distance, zero bearing)
#[test]
fn test_same_point() {
    let (dist, bear) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 0.0).unwrap(); // @safety-proof
    assert!(dist.abs() < 1e-6);
    assert!(bear.abs() < 1e-6);
}

/// Golden‑value test – 1° longitude difference on the equator
#[test]
fn test_one_degree_east() {
    let (dist, bear) = geo_haversine_distance_bearing(0.0, 0.0, 0.0, 1.0).unwrap(); // @safety-proof
    // Expected distance ≈ 2πR / 360 ≈ 111 194.9266 m
    assert!((dist - 111_194.9266).abs() < 1.0);
    // Expected bearing = 90°
    assert!((bear - 90.0).abs() < 1e-6);
}

/// Domain‑guard tests – out‑of‑range inputs must yield `None`
#[test]
fn test_out_of_range() {
    assert!(geo_haversine_distance_bearing(95.0, 0.0, 0.0, 0.0).is_none());
    assert!(geo_haversine_distance_bearing(0.0, -190.0, 0.0, 0.0).is_none());
}
