// ## Map
// Role: Moving RMS detector (pure‑std, O(1) sliding‑window update)
// Inputs: x_new: f64, x_old: f64, sum_sq: f64, window_len: usize, signal: &[f64]
// Outputs: (new_sum_sq, rms) for step, rms for slice
// Invariants: sum_sq ≥ 0.0, window_len ≥ 1, rms ≥ 0.0, all samples are finite

/// @outputs: 2
/// @domain: x_new any finite, x_old any finite, sum_sq [0.0..∞], window_len [1..usize::MAX]
///
/// O(1) recursive sliding‑window RMS update.
///
/// Returns `None` if any pre‑condition is violated.
pub fn dsp_moving_rms_step(
    x_new: f64,
    x_old: f64,
    sum_sq: f64,
    window_len: usize,
) -> Option<(f64, f64)> {
    // ---- domain guards -------------------------------------------------------
    if window_len == 0 {
        return None;
    }
    if !x_new.is_finite() || !x_old.is_finite() || !sum_sq.is_finite() {
        return None;
    }
    if sum_sq < 0.0 {
        return None;
    }

    // ---- core algorithm -------------------------------------------------------
    // update the squared‑sum, clamp to non‑negative to avoid tiny negatives
    let new_sum_sq = (sum_sq + x_new * x_new - x_old * x_old).max(0.0);
    // RMS = sqrt( mean of squares )
    let rms = (new_sum_sq / window_len as f64).sqrt();

    Some((new_sum_sq, rms))
}

/// @outputs: 1
/// @domain: signal.len() [1..∞]
///
/// Batch RMS for a whole slice.
///
/// Returns `None` if the slice is empty or contains a non‑finite value.
pub fn dsp_rms_slice(signal: &[f64]) -> Option<f64> {
    // ---- domain guards -------------------------------------------------------
    if signal.is_empty() {
        return None;
    }
    if signal.iter().any(|&x| !x.is_finite()) {
        return None;
    }

    // ---- core algorithm -------------------------------------------------------
    let sum_sq: f64 = signal.iter().map(|&x| x * x).sum();
    let mean_sq = sum_sq / signal.len() as f64;
    Some(mean_sq.sqrt())
}
