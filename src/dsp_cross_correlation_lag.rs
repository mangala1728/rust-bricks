// ## Map
// Role: Cross-correlation peak lag and TDOA time-delay estimator
// Inputs: x: &[f64], y: &[f64], max_lag: usize
// Outputs: Option<(i32, f64)> (best_lag, peak_correlation)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: x.len() [4..usize::MAX]
/// @domain: y.len() [4..usize::MAX]
/// @domain: max_lag [1..usize::MAX]
///
/// Compute the lag (in samples) that maximises the discrete cross‑correlation
/// between `x` and `y` inside the search window `[-max_lag, max_lag]`.
/// Returns `None` if any domain guard fails.
pub fn dsp_find_peak_cross_correlation_lag(
    x: &[f64],
    y: &[f64],
    max_lag: usize,
) -> Option<(i32, f64)> {
    // ----- domain guards ----------------------------------------------------
    if x.len() < 4 || y.len() < 4 {
        return None;
    }
    if max_lag == 0 || max_lag >= x.len() || max_lag >= y.len() {
        return None;
    }
    // all values must be finite (no NaN / Inf)
    if x.iter().any(|v| !v.is_finite()) || y.iter().any(|v| !v.is_finite()) {
        return None;
    }

    // ----- search -----------------------------------------------------------
    let mut best_lag: i32 = 0;
    let mut best_val: f64 = f64::NEG_INFINITY;

    let max_lag_i32 = max_lag as i32;
    let x_len = x.len() as i32;
    let y_len = y.len() as i32;

    let mut lag = -max_lag_i32;
    while lag <= max_lag_i32 {
        let mut acc = 0.0_f64;
        let mut n = 0_i32;
        while n < x_len {
            let idx_y = n + lag;
            if idx_y >= 0 && idx_y < y_len {
                // SAFETY: indices are within bounds after the checks above
                acc += x[n as usize] * y[idx_y as usize];
            }
            n += 1;
        }
        if acc > best_val {
            best_val = acc;
            best_lag = lag;
        }
        lag += 1;
    }

    Some((best_lag, best_val))
}
