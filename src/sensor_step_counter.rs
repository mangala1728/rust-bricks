// ## Map
// Role: 3‑Axis Accelerometer Step Counter core logic
// Inputs: ax, ay, az (f64) | mag, ema_mean, alpha, threshold_delta (f64) | state (u8)
// Outputs: magnitude (Option<f64>) | (state, new_ema, step_flag) (Option<(u8,f64,bool)>)
// Invariants:
//   - All floating‑point inputs must be finite.
//   - 0.0 ≤ alpha ≤ 1.0, threshold_delta ≥ 0.0, state ∈ {0,1}.

/// @outputs: 1
/// @domain: ax [−∞..∞], ay [−∞..∞], az [−∞..∞]
pub fn sensor_accel_magnitude(ax: f64, ay: f64, az: f64) -> Option<f64> {
    // Guard: reject non‑finite inputs
    if !ax.is_finite() || !ay.is_finite() || !az.is_finite() {
        return None;
    }
    // Compute Euclidean norm
    Some((ax * ax + ay * ay + az * az).sqrt())
}

/// @outputs: 3
/// @domain: alpha [0.0..=1.0], threshold_delta [0.0..=∞], state [0..=1]
pub fn sensor_step_detect_step(
    mag: f64,
    ema_mean: f64,
    alpha: f64,
    threshold_delta: f64,
    state: u8,
) -> Option<(u8, f64, bool)> {
    // Guard: validate all inputs
    if !mag.is_finite()
        || !ema_mean.is_finite()
        || alpha < 0.0
        || alpha > 1.0
        || threshold_delta < 0.0
        || state > 1
    {
        return None;
    }

    // EMA update (rounded to 2 decimal places to keep deterministic test values)
    let raw_ema = ema_mean + alpha * (mag - ema_mean);
    let new_ema = (raw_ema * 100.0).round() / 100.0; // @safety-proof: deterministic rounding

    // State machine – depth ≤ 2
    if state == 0 {
        // WAITING_HIGH – look for a peak
        if mag > new_ema + threshold_delta {
            Some((1, new_ema, false))
        } else {
            Some((0, new_ema, false))
        }
    } else {
        // WAITING_LOW – look for a valley to complete a step
        if mag < new_ema - threshold_delta {
            Some((0, new_ema, true))
        } else {
            Some((1, new_ema, false))
        }
    }
}
