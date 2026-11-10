// ## Map
// Role: ECEF cartesian to WGS-84 Geodetic Latitude, Longitude, Altitude (LLA)
// Inputs: x, y, z: f64 (meters)
// Outputs: Option<(f64, f64, f64)> (lat_deg, lon_deg, alt_m)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: x [-1e8..1e8]
/// @domain: y [-1e8..1e8]
/// @domain: z [-1e8..1e8]
pub fn geo_ecef_to_geodetic_lla(
    x: f64,
    y: f64,
    z: f64,
) -> Option<(f64, f64, f64)> {
    // guard: x finite and within magnitude bound
    if !x.is_finite() || x.abs() >= 1e8_f64 {
        return None;
    }
    // guard: y finite and within magnitude bound
    if !y.is_finite() || y.abs() >= 1e8_f64 {
        return None;
    }
    // guard: z finite and within magnitude bound
    if !z.is_finite() || z.abs() >= 1e8_f64 {
        return None;
    }

    // guard: distance from polar axis must be > 1 m²
    let p_sq = x * x + y * y;
    if p_sq < 1.0 {
        return None;
    }

    // WGS‑84 constants
    const A: f64 = 6_378_137.0;                     // semi‑major axis (m)
    const F: f64 = 1.0 / 298.257_223_563;           // flattening
    const B: f64 = A * (1.0 - F);                   // semi‑minor axis (m)
    const E_SQ: f64 = 2.0 * F - F * F;              // first eccentricity squared
    const EP_SQ: f64 = (A * A - B * B) / (B * B);   // second eccentricity squared

    // Bowring closed‑form solution
    let p = p_sq.sqrt();
    let theta = (z * A).atan2(p * B);
    let sin_th = theta.sin();
    let cos_th = theta.cos();
    let phi = (z + EP_SQ * B * sin_th.powi(3))
        .atan2(p - E_SQ * A * cos_th.powi(3));
    let lambda = y.atan2(x);
    let sin_phi = phi.sin();
    let n_rad = A / (1.0 - E_SQ * sin_phi * sin_phi).sqrt();
    let h = p / phi.cos() - n_rad;

    Some((phi.to_degrees(), lambda.to_degrees(), h))
}
