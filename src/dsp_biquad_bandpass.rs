// ## Map
// Role: DSP Biquad Band‑Pass Filter (pure Rust, no external crates)
// Inputs: sample_rate_hz [1.0..1_000_000.0], center_freq_hz [0.1..sample_rate_hz/2.0],
//         q_factor [0.01..1000.0], input_signal [-1.0e9..1.0e9]
// Outputs: coeffs [5] (normalized biquad coefficients), filtered_sample [1]
// Invariants: all parameters must be finite (no NaN/Inf), centre frequency < Nyquist,
/// Computes the normalized biquad coefficients for a constant‑0 dB peak‑gain band‑pass filter.
///
/// @outputs: 5
/// @domain: sample_rate_hz [1.0..1_000_000.0]
/// @domain: center_freq_hz [0.1..sample_rate_hz/2.0)（不含 fs/2：ω₀=π 時 α≈0，濾波器退化）
/// @domain: q_factor [0.01..1000.0]
pub fn biquad_bandpass_coefficients(
    sample_rate_hz: f64,
    center_freq_hz: f64,
    q_factor: f64,
) -> Option<[f64; 5]> {
    // Guard against out‑of‑range or non‑finite values
    if !(sample_rate_hz.is_finite()
        && center_freq_hz.is_finite()
        && q_factor.is_finite())
    {
        return None;
    }
    if sample_rate_hz < 1.0 || sample_rate_hz > 1_000_000.0 {
        return None;
    }
    if center_freq_hz < 0.1 || center_freq_hz >= sample_rate_hz / 2.0 {
        return None;
    }
    if q_factor < 0.01 || q_factor > 1000.0 {
        return None;
    }

    // ω₀ = 2π·f₀ / fₛ
    let omega = 2.0 * std::f64::consts::PI * center_freq_hz / sample_rate_hz;
    // α = sin(ω₀) / (2Q)
    let alpha = omega.sin() / (2.0 * q_factor);

    // Un‑normalized coefficients
    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * omega.cos();
    let a2 = 1.0 - alpha;

    // Normalisation (divide by a0)
    let inv_a0 = 1.0 / a0;
    Some([
        b0 * inv_a0,
        b1 * inv_a0,
        b2 * inv_a0,
        a1 * inv_a0,
        a2 * inv_a0,
    ])
}

/// Performs one Direct‑Form‑II‑Transposed step of the biquad band‑pass filter.
///
/// @outputs: 1
/// @domain: input [-1.0e9..1.0e9]
pub fn biquad_bandpass_step(
    input: f64,
    state: &mut [f64; 2],
    coeffs: &[f64; 5],
) -> Option<f64> {
    // Guard against out‑of‑range or non‑finite input
    if !input.is_finite() {
        return None;
    }
    if input < -1.0e9 || input > 1.0e9 {
        return None;
    }

    // Direct‑Form‑II‑Transposed equations
    let y = coeffs[0] * input + state[0];
    let new_state0 = coeffs[1] * input - coeffs[3] * y + state[1];
    let new_state1 = coeffs[2] * input - coeffs[4] * y;

    state[0] = new_state0;
    state[1] = new_state1;

    Some(y)
}
