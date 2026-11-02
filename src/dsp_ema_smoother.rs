// ## Map
// Role: Hardware-grade zero-allocation Exponential Moving Average (EMA) and signal smoother for DSP/sensor pipelines.
// Inputs: Current signal sample (f64), previous filtered value (f64), smoothing factor alpha (f64).
// Outputs: Filtered signal value (Option<f64> or Option<(f64, f64, f64)>).
// Invariants: Deterministic execution, zero memory allocation, strict first-line domain validation.

/// Calculates a single step Exponential Moving Average (EMA).
///
/// Formula: `out = alpha * current + (1.0 - alpha) * prev`
///
/// @outputs: 2
/// @domain: current [-1000000.0..1000000.0]
/// @domain: prev [-1000000.0..1000000.0]
/// @domain: alpha [0.0001..1.0]
pub fn ema_step_f64(current: f64, prev: f64, alpha: f64) -> Option<f64> {
    if !alpha.is_finite()
        || alpha < 0.0001
        || alpha > 1.0
        || !current.is_finite()
        || current < -1000000.0
        || current > 1000000.0
        || !prev.is_finite()
        || prev < -1000000.0
        || prev > 1000000.0
    {
        return None;
    }

    let smoothed = alpha * current + (1.0 - alpha) * prev;
    Some(smoothed)
}

/// Calculates Double Exponential Moving Average (DEMA) to eliminate phase lag in sensor streams.
///
/// Returns `(dema, new_ema1, new_ema2)` state.
///
/// @outputs: 3
/// @domain: current [-1000000.0..1000000.0]
/// @domain: ema1_prev [-1000000.0..1000000.0]
/// @domain: ema2_prev [-1000000.0..1000000.0]
/// @domain: alpha [0.0001..1.0]
pub fn dema_step_f64(
    current: f64,
    ema1_prev: f64,
    ema2_prev: f64,
    alpha: f64,
) -> Option<(f64, f64, f64)> {
    if !alpha.is_finite()
        || alpha < 0.0001
        || alpha > 1.0
        || !current.is_finite()
        || current < -1000000.0
        || current > 1000000.0
        || !ema1_prev.is_finite()
        || ema1_prev < -1000000.0
        || ema1_prev > 1000000.0
        || !ema2_prev.is_finite()
        || ema2_prev < -1000000.0
        || ema2_prev > 1000000.0
    {
        return None;
    }

    let ema1 = alpha * current + (1.0 - alpha) * ema1_prev;
    let ema2 = alpha * ema1 + (1.0 - alpha) * ema2_prev;
    let dema = 2.0 * ema1 - ema2;

    Some((dema, ema1, ema2))
}
