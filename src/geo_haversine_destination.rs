// ## Map
// Role: Compute the destination point on a spherical Earth given start latitude, longitude,
// bearing and travel distance.
// Inputs: lat_deg (f64) ∈ [-90.0, 90.0], lon_deg (f64) ∈ [-180.0, 180.0],
//         bearing_deg (f64) ∈ [0.0, 360.0], distance_m (f64) ≥ 0.0
// Outputs: (lat2_deg, lon2_deg) – latitude and longitude of the destination point.
// Invariants: all angles are finite, distance is finite and non‑negative,
//             Earth radius is constant 6 371 000 m.

/// @outputs: 2
/// @domain: lat_deg [-90.0..=90.0]
/// @domain: lon_deg [-180.0..=180.0]
/// @domain: bearing_deg [0.0..=360.0]
/// @domain: distance_m [0.0..=∞]
pub fn geo_destination_point(
    lat_deg: f64,
    lon_deg: f64,
    bearing_deg: f64,
    distance_m: f64,
) -> Option<(f64, f64)> {
    // Guard: parameter validation (finite + range)
    if !lat_deg.is_finite()
        || !lon_deg.is_finite()
        || !bearing_deg.is_finite()
        || !distance_m.is_finite()
        || lat_deg < -90.0
        || lat_deg > 90.0
        || lon_deg < -180.0
        || lon_deg > 180.0
        || bearing_deg < 0.0
        || bearing_deg > 360.0
        || distance_m < 0.0
    {
        return None;
    }

    // Constants
    const R: f64 = 6_371_000.0; // mean Earth radius in metres

    // Convert to radians
    let phi1 = lat_deg.to_radians();
    let lambda1 = lon_deg.to_radians();
    let theta = bearing_deg.to_radians();
    let delta = distance_m / R;

    // Compute destination latitude (φ₂)
    let sin_phi1 = phi1.sin();
    let cos_phi1 = phi1.cos();
    let sin_delta = delta.sin();
    let cos_delta = delta.cos();
    let sin_theta = theta.sin();
    let cos_theta = theta.cos();

    let sin_phi2 = sin_phi1 * cos_delta + cos_phi1 * sin_delta * cos_theta;
    let phi2 = sin_phi2.clamp(-1.0, 1.0).asin(); // clamp for numerical safety

    // Compute destination longitude (λ₂)
    let y = sin_theta * sin_delta * cos_phi1;
    let x = cos_delta - sin_phi1 * sin_phi2;
    let lambda2 = lambda1 + y.atan2(x);

    // Normalise longitude to [-180°, 180°]
    let lon2_deg = ((lambda2.to_degrees() + 540.0) % 360.0) - 180.0;
    let lat2_deg = phi2.to_degrees();

    Some((lat2_deg, lon2_deg))
}
