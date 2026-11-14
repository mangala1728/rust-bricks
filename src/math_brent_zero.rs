// ## Map
// Role: Brent-Dekker numerical root-finder candidate step computation
// Inputs: a, b, c, fa, fb, fc: f64
// Outputs: Option<f64> (next_candidate_x)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: a [-1e308..1e308]
/// @domain: b [-1e308..1e308]
/// @domain: c [-1e308..1e308]
/// @domain: fa [-1e308..1e308]
/// @domain: fb [-1e308..1e308]
/// @domain: fc [-1e308..1e308]
pub fn math_brent_candidate_step(
    a: f64,
    b: f64,
    c: f64,
    fa: f64,
    fb: f64,
    fc: f64,
) -> Option<f64> {
    // ---- domain guards -------------------------------------------------------
    if !a.is_finite()
        || !b.is_finite()
        || !c.is_finite()
        || !fa.is_finite()
        || !fb.is_finite()
        || !fc.is_finite()
    {
        return None;
    }
    // root must be bracketed (fa * fb <= 0)
    if fa * fb > 0.0 {
        return None;
    }
    // avoid division by (fa - fb) ≈ 0
    if (fa - fb).abs() < 1e-15 {
        return None;
    }

    // ---- candidate computation ------------------------------------------------
    let s = if (fa - fc).abs() > 1e-14 && (fb - fc).abs() > 1e-14 {
        // Inverse Quadratic Interpolation
        let term1 = a * fb * fc / ((fa - fb) * (fa - fc));
        let term2 = b * fa * fc / ((fb - fa) * (fb - fc));
        let term3 = c * fa * fb / ((fc - fa) * (fc - fb));
        term1 + term2 + term3
    } else {
        // Secant step
        b - fb * (b - a) / (fb - fa)
    };

    // ---- safety check (bisection fallback) ----------------------------------
    let min_bound = (3.0 * a + b) / 4.0;
    let (low, high) = if min_bound < b {
        (min_bound, b)
    } else {
        (b, min_bound)
    };
    let s = if s < low || s > high {
        (a + b) / 2.0
    } else {
        s
    };

    Some(s)
}
