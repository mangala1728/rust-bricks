// ## Map Role: DSP Linear Fractional Resampler
// Inputs: input: &[f64], in_rate_hz: f64, out_rate_hz: f64
// Outputs: Vec<f64>
// Invariants: 1.0 ≤ in_rate_hz ≤ 1_000_000.0, 1.0 ≤ out_rate_hz ≤ 1_000_000.0,
//             input may be empty (handled), output length = floor(input.len() / R)

/// @outputs: 1
/// @domain: in_rate_hz [1.0..1000000.0]
/// @domain: out_rate_hz [1.0..1000000.0]
pub fn resample_linear_1d(
    input: &[f64],
    in_rate_hz: f64,
    out_rate_hz: f64,
) -> Option<Vec<f64>> {
    // Guard: validate domains and empty input
    if input.is_empty()
        || in_rate_hz < 1.0
        || in_rate_hz > 1_000_000.0
        || out_rate_hz < 1.0
        || out_rate_hz > 1_000_000.0
    {
        return None;
    }

    // Ratio R = in_rate / out_rate
    let r = in_rate_hz / out_rate_hz;

    // Output length M = floor(N / R)
    let m = ((input.len() as f64) / r).floor() as usize;

    // If no samples are produced, return empty vector
    if m == 0 {
        return Some(Vec::new());
    }

    // Allocate output buffer
    let mut out = Vec::with_capacity(m);

    // Linear interpolation loop (nesting depth ≤ 2)
    for idx_out in 0..m {
        let pos = (idx_out as f64) * r;
        let base = pos.floor() as usize;
        let alpha = pos - (base as f64);

        // Safe indexing – base is always < input.len()
        let sample = if base + 1 < input.len() {
            (1.0 - alpha) * input[base] + alpha * input[base + 1]
        } else {
            // Edge case: exactly at the last input sample
            input[base]
        };
        out.push(sample);
    }

    Some(out)
}
