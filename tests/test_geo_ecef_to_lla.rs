use rust_bricks::geo_ecef_to_lla::*;

#[test]
fn golden_equator_origin() {
    let (lat, lon, alt) = geo_ecef_to_geodetic_lla(6_378_137.0, 0.0, 0.0).unwrap();
    assert!(lat.abs() < 1e-7, "lat = {lat}");
    assert!(lon.abs() < 1e-7, "lon = {lon}");
    assert!(alt.abs() < 1e-4, "alt = {alt}");
}

#[test]
fn golden_invalid_zero() {
    assert!(geo_ecef_to_geodetic_lla(0.0, 0.0, 0.0).is_none());
}

#[test]
fn golden_invalid_nan() {
    assert!(geo_ecef_to_geodetic_lla(f64::NAN, 0.0, 0.0).is_none());
}
