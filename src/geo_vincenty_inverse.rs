// ## Map
// Role: Vincenty inverse geodesic calculation on WGS-84 ellipsoid
// Inputs: lat1, lon1, lat2, lon2: f64 in degrees
// Outputs: Option<(f64, f64)> (distance_m, initial_bearing_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

const WGS84_A: f64 = 6378137.0;
const WGS84_F: f64 = 1.0 / 298.257223563;
const WGS84_B: f64 = (1.0 - WGS84_F) * WGS84_A;

/// @outputs: 6
fn vincenty_iteration(
    sin_u1: f64,
    cos_u1: f64,
    sin_u2: f64,
    cos_u2: f64,
    l_rad: f64,
) -> Option<(f64, f64, f64, f64, f64, f64)> {
    let mut lambda = l_rad;

    for _ in 0..100 {
        let sin_lambda = lambda.sin();
        let cos_lambda = lambda.cos();

        let t1 = cos_u2 * sin_lambda;
        let t2 = cos_u1 * sin_u2 - sin_u1 * cos_u2 * cos_lambda;
        let sin_sigma = (t1 * t1 + t2 * t2).sqrt();
        if sin_sigma <= 1e-15 {
            return Some((0.0, 0.0, 0.0, 0.0, 0.0, lambda));
        }
        let cos_sigma = sin_u1 * sin_u2 + cos_u1 * cos_u2 * cos_lambda;
        let sigma = sin_sigma.atan2(cos_sigma);

        let sin_alpha = (cos_u1 * cos_u2 * sin_lambda) / sin_sigma;
        let cos2_alpha = 1.0 - sin_alpha * sin_alpha;

        let cos2_sig_m = if cos2_alpha.abs() > 1e-12 {
            cos_sigma - 2.0 * sin_u1 * sin_u2 / cos2_alpha
        } else {
            0.0
        };

        let c = (WGS84_F / 16.0) * cos2_alpha * (4.0 + WGS84_F * (4.0 - 3.0 * cos2_alpha));
        let lambda_prev = lambda;
        let term = sigma + c * sin_sigma * (cos2_sig_m + c * cos_sigma * (-1.0 + 2.0 * cos2_sig_m * cos2_sig_m));
        lambda = l_rad + (1.0 - c) * WGS84_F * sin_alpha * term;

        if (lambda - lambda_prev).abs() < 1e-12 {
            return Some((sigma, sin_sigma, cos_sigma, cos2_alpha, cos2_sig_m, lambda));
        }
    }
    None
}

/// @outputs: 2
/// @domain: lat1, lat2 in [-90.0..=90.0], lon1, lon2 in [-180.0..=180.0]
pub fn geo_vincenty_inverse(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
) -> Option<(f64, f64)> {
    if !lat1.is_finite() || !lon1.is_finite() || !lat2.is_finite() || !lon2.is_finite()
        || lat1 < -90.0 || lat1 > 90.0 || lat2 < -90.0 || lat2 > 90.0
        || lon1 < -180.0 || lon1 > 180.0 || lon2 < -180.0 || lon2 > 180.0
    {
        return None;
    }
    if (lat1 - lat2).abs() < 1e-12 && (lon1 - lon2).abs() < 1e-12 {
        return Some((0.0, 0.0));
    }

    let phi1 = lat1.to_radians();
    let phi2 = lat2.to_radians();
    let u1 = ((1.0 - WGS84_F) * phi1.tan()).atan();
    let u2 = ((1.0 - WGS84_F) * phi2.tan()).atan();
    let (sin_u1, cos_u1) = (u1.sin(), u1.cos());
    let (sin_u2, cos_u2) = (u2.sin(), u2.cos());
    let l_rad = (lon2 - lon1).to_radians();

    let (sigma, sin_sigma, cos_sigma, cos2_alpha, cos2_sig_m, lambda_conv) =
        vincenty_iteration(sin_u1, cos_u1, sin_u2, cos_u2, l_rad)?;
    if sigma == 0.0 {
        return Some((0.0, 0.0));
    }

    let u_sq = cos2_alpha * (WGS84_A * WGS84_A - WGS84_B * WGS84_B) / (WGS84_B * WGS84_B);
    let a_val = 1.0 + (u_sq / 16384.0) * (4096.0 + u_sq * (-768.0 + u_sq * (320.0 - 175.0 * u_sq)));
    let b_val = (u_sq / 1024.0) * (256.0 + u_sq * (-128.0 + u_sq * (74.0 - 47.0 * u_sq)));

    let delta_sigma = b_val * sin_sigma * (cos2_sig_m + 0.25 * b_val * (cos_sigma * (-1.0 + 2.0 * cos2_sig_m * cos2_sig_m)
        - (b_val / 6.0) * cos2_sig_m * (-3.0 + 4.0 * sin_sigma * sin_sigma) * (-3.0 + 4.0 * cos2_sig_m * cos2_sig_m)));

    let dist = WGS84_B * a_val * (sigma - delta_sigma);

    let y = cos_u2 * lambda_conv.sin();
    let x = cos_u1 * sin_u2 - sin_u1 * cos_u2 * lambda_conv.cos();
    let bearing = (y.atan2(x).to_degrees() + 360.0) % 360.0;

    Some((dist, bearing))
}
