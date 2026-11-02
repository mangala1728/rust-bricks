// ## Map
// Role: 1D Adaptive Kalman Filter Step
// Inputs: (state.0 = estimate, state.1 = covariance), measurement, q_noise, r_noise
// Outputs: updated estimate (Option<f64>)
// Invariants: state.1 ≥ 0.0, q_noise ≥ 0.0, r_noise > 0.0, measurement is finite

/// One‑dimensional Kalman filter single‑step update.
///
/// # Parameters
/// * `state` – mutable reference to `(estimate, covariance)`.
/// * `measurement` – current sensor reading `z`.
/// * `q_noise` – process‑noise covariance `Q`.
/// * `r_noise` – measurement‑noise covariance `R`.
///
/// # Returns
/// `Some(updated_estimate)` on success, `None` if any input violates its domain.
///
/// @outputs: 1
/// @domain: q_noise [0.0..1000.0]
/// @domain: r_noise [1.0e-9..1000.0]
pub fn kalman_1d_step(
    state: &mut (f64, f64),
    measurement: f64,
    q_noise: f64,
    r_noise: f64,
) -> Option<f64> {
    // ---- Guard checks -------------------------------------------------------
    if q_noise < 0.0 || q_noise > 1000.0 {
        return None;
    }
    if r_noise < 1.0e-9 || r_noise > 1000.0 {
        return None;
    }
    if state.1 < 0.0 {
        return None;
    }
    if !measurement.is_finite() {
        return None;
    }

    // ---- Predict (time update) ---------------------------------------------
    let p_pred = state.1 + q_noise;               // P⁻ = P + Q

    // ---- Update (measurement update) ---------------------------------------
    let k = p_pred / (p_pred + r_noise);          // Kalman gain K
    let x_new = state.0 + k * (measurement - state.0); // \hat{x}
    let p_new = (1.0 - k) * p_pred;               // P = (1‑K)·P⁻

    // ---- Write back ---------------------------------------------------------
    state.0 = x_new;
    state.1 = p_new;

    Some(x_new)
}
