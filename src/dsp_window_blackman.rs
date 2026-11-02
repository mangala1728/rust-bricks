// ## Map
// Role: DSP windowing
// Inputs: index usize, window_len usize, buffer &mut [f64]
// Outputs: Option<f64>, bool
// Invariants: 0 ≤ index < window_len, window_len ≥ 2, buffer.len() ≥ 2

/// @outputs: 1
/// @domain: index [0..usize::MAX]
/// @domain: window_len [2..1048576]
pub fn blackman_window_coeff(index: usize, window_len: usize) -> Option<f64> {
    // ── guard ─────────────────────────────────────────────────────
    if window_len < 2 || window_len > 1_048_576 || index >= window_len {
        return None;
    }
    // ── compute ───────────────────────────────────────────────────
    let n = index as f64;
    let n_denom = (window_len - 1) as f64;
    let two_pi_n_over_n = 2.0 * std::f64::consts::PI * n / n_denom;
    let four_pi_n_over_n = 4.0 * std::f64::consts::PI * n / n_denom;
    let w = 0.42 - 0.5 * two_pi_n_over_n.cos() + 0.08 * four_pi_n_over_n.cos();
    Some(w)
}

/// @outputs: 1
pub fn apply_blackman_window(buffer: &mut [f64]) -> bool {
    // ── guard ─────────────────────────────────────────────────────
    if buffer.len() < 2 {
        return false;
    }
    // ── apply ─────────────────────────────────────────────────────
    let len = buffer.len();
    for (i, v) in buffer.iter_mut().enumerate() {
        // safety‑proof: `i < len` is guaranteed by the loop, therefore
        // `blackman_window_coeff(i, len)` always returns `Some`.
        let coeff = blackman_window_coeff(i, len).unwrap(); // @safety-proof
        *v *= coeff;
    }
    true
}
