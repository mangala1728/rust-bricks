// ## Map
// Role: Verify the correctness and domain guards of the solar zenith/elevation calculator
// Inputs: day_of_year, utc_hour, lat_deg, lon_deg
// Outputs: test assertions
// Invariants: zero external dependencies, pure Rust, deterministic golden values

use rust_bricks::astro_solar_zenith_angle::*;

#[test]
fn golden_equinox_at_equator_noon() {
    // Spring equinox ≈ day 81, UTC noon, lat=0°, lon=0°
    let (zenith, elevation) = astro_solar_zenith_and_elevation(81, 12.0, 0.0, 0.0)
        .expect("valid input should produce a result");

    // zenith + elevation must be exactly 90°
    assert!((zenith + elevation - 90.0).abs() < 1e-10, "zenith+elevation ≠ 90°");

    // Sun should be almost overhead
    assert!(zenith < 5.0, "zenith not close to 0°");
    assert!(elevation > 85.0, "elevation not close to 90°");
}

#[test]
fn rejects_invalid_day() {
    assert!(astro_solar_zenith_and_elevation(0, 12.0, 0.0, 0.0).is_none());
    assert!(astro_solar_zenith_and_elevation(367, 12.0, 0.0, 0.0).is_none());
}

#[test]
fn rejects_invalid_utc_hour() {
    assert!(astro_solar_zenith_and_elevation(81, -1.0, 0.0, 0.0).is_none());
    assert!(astro_solar_zenith_and_elevation(81, 25.0, 0.0, 0.0).is_none());
    assert!(astro_solar_zenith_and_elevation(81, f64::NAN, 0.0, 0.0).is_none());
}

#[test]
fn rejects_invalid_lat_lon() {
    assert!(astro_solar_zenith_and_elevation(81, 12.0, 95.0, 0.0).is_none());
    assert!(astro_solar_zenith_and_elevation(81, 12.0, -95.0, 0.0).is_none());
    assert!(astro_solar_zenith_and_elevation(81, 12.0, 0.0, 200.0).is_none());
    assert!(astro_solar_zenith_and_elevation(81, 12.0, 0.0, -200.0).is_none());
}
