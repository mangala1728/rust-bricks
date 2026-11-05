// ## Map
// Role: Natural cubic spline single segment evaluation
// Inputs: x, x0, x1, y0, y1, m0, m1: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x0 < x1, x in [x0..=x1], all inputs finite
pub fn math_cubic_spline_eval(
    x: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    m0: f64,
    m1: f64,
) -> Option<f64> {
    if !x.is_finite()
        || !x0.is_finite()
        || !x1.is_finite()
        || !y0.is_finite()
        || !y1.is_finite()
        || !m0.is_finite()
        || !m1.is_finite()
        || x1 <= x0
        || x < x0
        || x > x1
    {
        return None;
    }

    let h = x1 - x0;
    let dx0 = x - x0;
    let dx1 = x1 - x;

    let term1 = (m0 * dx1 * dx1 * dx1) / (6.0 * h);
    let term2 = (m1 * dx0 * dx0 * dx0) / (6.0 * h);
    let term3 = (y0 - (m0 * h * h) / 6.0) * (dx1 / h);
    let term4 = (y1 - (m1 * h * h) / 6.0) * (dx0 / h);

    Some(term1 + term2 + term3 + term4)
}

/// @outputs: 1
/// @domain: x0 < x1, x in [x0..=x1], all inputs finite
pub fn math_cubic_spline_derivative(
    x: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    m0: f64,
    m1: f64,
) -> Option<f64> {
    if !x.is_finite()
        || !x0.is_finite()
        || !x1.is_finite()
        || !y0.is_finite()
        || !y1.is_finite()
        || !m0.is_finite()
        || !m1.is_finite()
        || x1 <= x0
        || x < x0
        || x > x1
    {
        return None;
    }

    let h = x1 - x0;
    let dx0 = x - x0;
    let dx1 = x1 - x;

    let d_term1 = -(m0 * dx1 * dx1) / (2.0 * h);
    let d_term2 = (m1 * dx0 * dx0) / (2.0 * h);
    let d_term3 = -(y0 - (m0 * h * h) / 6.0) / h;
    let d_term4 = (y1 - (m1 * h * h) / 6.0) / h;

    Some(d_term1 + d_term2 + d_term3 + d_term4)
}
