// ## Map
// Role: 1D Sliding mode control with boundary layer chattering alleviation
// Inputs: error, error_dot, c, k, epsilon: f64
// Outputs: Option<(f64, f64)> (sliding_surface_s, control_u)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: c [0.0..]
/// @domain: k [0.0..]
/// @domain: epsilon [0.0..]
///
/// Computes one step of a 1‑D sliding‑mode controller with a boundary layer.
///
/// * `error` – tracking error `e(t)`
/// * `error_dot` – derivative of the tracking error `ė(t)`
/// * `c` – positive surface coefficient (must be > 0)
/// * `k` – positive control gain (must be > 0)
/// * `epsilon` – positive boundary‑layer thickness (must be > 0)
///
/// Returns `None` if any argument is non‑finite or violates the domain guard,
/// otherwise returns the sliding surface `s` and the control effort `u`.
pub fn control_sliding_mode_step(
    error: f64,
    error_dot: f64,
    c: f64,
    k: f64,
    epsilon: f64,
) -> Option<(f64, f64)> {
    // domain guard – non‑finite values
    if !error.is_finite()
        || !error_dot.is_finite()
        || !c.is_finite()
        || !k.is_finite()
        || !epsilon.is_finite()
    {
        return None;
    }

    // domain guard – positivity constraints
    if c <= 0.0 || k <= 0.0 || epsilon <= 0.0 {
        return None;
    }

    // sliding surface
    let s = c * error + error_dot;

    // normalized surface
    let norm_s = s / epsilon;

    // saturation (boundary layer)
    let sat = if norm_s > 1.0 {
        1.0
    } else if norm_s < -1.0 {
        -1.0
    } else {
        norm_s
    };

    // control law
    let u = k * sat;

    Some((s, u))
}
