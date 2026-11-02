// ## Map
// Role: DSP Hann window coefficient generator
// Inputs: n [0..65535], total_len [1..65536]
// Outputs: Option<f64> (Some coefficient or None on guard failure)
// Invariants: total_len > 0, n < total_len, total_len ≤ 65536, n ≤ 65535

/// Compute the Hann (Hanning) window coefficient for sample `n` in a window of length `total_len`.
///
/// @outputs: 1
/// @domain: n [0..65535]
/// @domain: total_len [1..65536]
pub fn hann_window_coeff(n: usize, total_len: usize) -> Option<f64> {
    // @safety-proof: No unsafe code, no unwrap/expect/panic.
    // Guard: total_len must be within [1, 65536] and n must be within [0, total_len-1].
    if total_len == 0 || total_len > 65536 || n > 65535 || n >= total_len {
        return None;
    }

    // Special case: a single‑point window is defined as 1.0.
    if total_len == 1 {
        return Some(1.0);
    }

    // Hann window formula: w[n] = 0.5 * (1 - cos(2π n / (total_len - 1))).
    let denom = (total_len - 1) as f64;
    let angle = 2.0 * std::f64::consts::PI * (n as f64) / denom;
    Some(0.5 * (1.0 - angle.cos()))
}
