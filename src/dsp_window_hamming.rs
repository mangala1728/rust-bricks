// ## Map
// Role: DSP windowing primitive – Hamming window
// Inputs: index: usize, window_len: usize, buffer: &mut [f64]
// Outputs: Option<f64> (coeff), bool (apply result)
// Invariants: 2 ≤ window_len ≤ 1_048_576, 0 ≤ index < window_len, buffer.len() ≥ 2

/// Compute a single Hamming‑window coefficient.
///
/// @outputs: 1
/// @domain: index [0..1_048_575]
/// @domain: window_len [2..1_048_576]
pub fn hamming_window_coeff(index: usize, window_len: usize) -> Option<f64> {
    // guard: illegal length or out‑of‑range index
    if window_len < 2 || index >= window_len {
        return None;
    }

    // w(n) = α – β·cos(2πn/(N‑1)), α = 0.54, β = 0.46
    const ALPHA: f64 = 0.54;
    const BETA: f64 = 0.46;
    let theta = 2.0 * std::f64::consts::PI * (index as f64) / ((window_len - 1) as f64);
    let coeff = ALPHA - BETA * theta.cos();
    Some(coeff)
}

/// Apply an in‑place Hamming window to a mutable slice.
///
/// @outputs: 1
pub fn apply_hamming_window(buffer: &mut [f64]) -> bool {
    // guard: need at least two samples to form a window
    if buffer.len() < 2 {
        return false;
    }

    let n = buffer.len();
    for (i, sample) in buffer.iter_mut().enumerate() {
        // safety‑proof: we have already validated `i < n` and `n ≥ 2`
        // → `hamming_window_coeff` must return `Some`.
        if let Some(coeff) = hamming_window_coeff(i, n) {
            *sample *= coeff;
        } else {
            // Should never happen because of the guard above.
            return false;
        }
    }
    true
}
