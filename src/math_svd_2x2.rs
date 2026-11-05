// ## Map
// Role: Analytical 2x2 SVD singular values and condition number
// Inputs: a, b, c, d: f64
// Outputs: Option<(f64, f64)> or Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: a, b, c, d are finite
pub fn math_svd_2x2_singular_values(a: f64, b: f64, c: f64, d: f64) -> Option<(f64, f64)> {
    if !a.is_finite() || !b.is_finite() || !c.is_finite() || !d.is_finite() {
        return None;
    }
    let s1 = a * a + b * b + c * c + d * d;
    let det = a * d - b * c;
    let discr = (s1 * s1 - 4.0 * det * det).max(0.0);
    let s2 = discr.sqrt();

    let sigma1 = (0.5 * (s1 + s2)).max(0.0).sqrt();
    let sigma2 = (0.5 * (s1 - s2).max(0.0)).sqrt();

    Some((sigma1, sigma2))
}

/// @outputs: 1
/// @domain: a, b, c, d are finite
pub fn math_matrix_2x2_condition_number(a: f64, b: f64, c: f64, d: f64) -> Option<f64> {
    let (sigma1, sigma2) = math_svd_2x2_singular_values(a, b, c, d)?;
    if sigma2 <= 1e-12 {
        return None;
    }
    Some(sigma1 / sigma2)
}
