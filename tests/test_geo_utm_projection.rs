use rust_bricks::geo_utm_projection::*;

#[test]
fn test_golden_utm_zone_determination() {
    // @safety-proof: Golden value test inputs are guaranteed within valid domain range [-80.0..=84.0] and [-180.0..=180.0].
    let (z1, h1, cm1) = geo_determine_utm_zone(25.0, 121.5).unwrap();
    assert_eq!(z1, 51);
    assert_eq!(h1, 'N');
    assert!((cm1 - 123.0).abs() < 1e-12);

    // @safety-proof: Golden value test inputs are guaranteed within valid domain range.
    let (z2, h2, cm2) = geo_determine_utm_zone(51.5, 0.0).unwrap();
    assert_eq!(z2, 31);
    assert_eq!(h2, 'N');
    assert!((cm2 - 3.0).abs() < 1e-12);

    assert!(geo_determine_utm_zone(85.0, 0.0).is_none());
    assert!(geo_determine_utm_zone(-81.0, 0.0).is_none());
    assert!(geo_determine_utm_zone(0.0, 185.0).is_none());
}

#[test]
fn test_boundary_and_edge_cases() {
    // Maximum upper latitude bound
    // @safety-proof: Inputs strictly at boundary limit lat 84.0, lon 180.0.
    let (z_max_lat, h_max_lat, _cm_max) = geo_determine_utm_zone(84.0, 180.0).unwrap();
    assert_eq!(z_max_lat, 60);
    assert_eq!(h_max_lat, 'N');

    // Minimum lower latitude bound
    // @safety-proof: Inputs strictly at boundary limit lat -80.0, lon -180.0.
    let (z_min, h_min, cm_min) = geo_determine_utm_zone(-80.0, -180.0).unwrap();
    assert_eq!(z_min, 1);
    assert_eq!(h_min, 'S');
    assert!((cm_min - (-177.0)).abs() < 1e-12);

    // Non-finite value guards
    assert!(geo_determine_utm_zone(f64::NAN, 121.5).is_none());
    assert!(geo_determine_utm_zone(25.0, f64::INFINITY).is_none());
}
