// ## Map
// Role: Haversine spherical distance and initial forward azimuth bearing
// Inputs: lat1_deg, lon1_deg, lat2_deg, lon2_deg: f64
// Outputs: Option<(f64, f64)> (distance_meters, initial_bearing_degrees)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: lat1_deg [-90.0..=90.0]
/// @domain: lon1_deg [-180.0..=180.0]
/// @domain: lat2_deg [-90.0..=90.0]
/// @domain: lon2_deg [-180.0..=180.0]
pub fn geo_haversine_distance_bearing(
    lat1_deg: f64,
    lon1_deg: f64,
    lat2_deg: f64,
    lon2_deg: f64,
) -> Option<(f64, f64)> {
    // domain guard – finite & range
    if !lat1_deg.is_finite()
        || !lon1_deg.is_finite()
        || !lat2_deg.is_finite()
        || !lon2_deg.is_finite()
        || lat1_deg < -90.0
        || lat1_deg > 90.0
        || lat2_deg < -90.0
        || lat2_deg > 90.0
        || lon1_deg < -180.0
        || lon1_deg > 180.0
        || lon2_deg < -180.0
        || lon2_deg > 180.0
    {
        return None;
    }

    // constants & radian conversion
    let r_earth = 6_371_000.0_f64;
    let phi1 = lat1_deg.to_radians();
    let phi2 = lat2_deg.to_radians();
    let d_phi = (lat2_deg - lat1_deg).to_radians();
    let d_lambda = (lon2_deg - lon1_deg).to_radians();

    // Haversine distance
    let a = (d_phi / 2.0).sin().powi(2)
        + phi1.cos() * phi2.cos() * (d_lambda / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).max(0.0).sqrt());
    let distance = r_earth * c;

    // Initial bearing
    let y = d_lambda.sin() * phi2.cos();
    let x = phi1.cos() * phi2.sin() - phi1.sin() * phi2.cos() * d_lambda.cos();
    let mut bearing = y.atan2(x).to_degrees();
    bearing = (bearing % 360.0 + 360.0) % 360.0;

    Some((distance, bearing))
}
