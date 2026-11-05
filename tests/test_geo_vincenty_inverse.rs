// ## Map
// Role: integration test for WGS-84 Vincenty inverse geodesy
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::geo_vincenty_inverse::*;

#[test]
fn test_geo_vincenty_inverse_golden() {
    let (dist, bear) = geo_vincenty_inverse(0.0, 0.0, 0.0, 1.0).unwrap();
    assert!((dist - 111319.49).abs() < 1.0);
    assert!((bear - 90.0).abs() < 1e-3);

    // Coincident points
    let (dist0, _) = geo_vincenty_inverse(25.0, 121.0, 25.0, 121.0).unwrap();
    assert!(dist0.abs() < 1e-6);
}

#[test]
fn test_geo_vincenty_inverse_guards() {
    // Lat > 90
    assert!(geo_vincenty_inverse(95.0, 0.0, 0.0, 0.0).is_none());
    assert!(geo_vincenty_inverse(-95.0, 0.0, 0.0, 0.0).is_none());

    // Lon > 180
    assert!(geo_vincenty_inverse(0.0, 185.0, 0.0, 0.0).is_none());
    assert!(geo_vincenty_inverse(0.0, -185.0, 0.0, 0.0).is_none());

    // NaN
    assert!(geo_vincenty_inverse(f64::NAN, 0.0, 0.0, 0.0).is_none());
    assert!(geo_vincenty_inverse(0.0, f64::NAN, 0.0, 0.0).is_none());
}
