// ## Map
// Role: DSP Biquad Notch Filter (zero‑external‑dependency, std only)
// Inputs: sample_rate_hz, center_freq_hz, q_factor, input, state, coeffs
// Outputs: coefficients array ([f64;5]), filtered sample (f64)
// Invariants:
//   - All numeric parameters must be finite (no NaN/Inf)
//   - sample_rate_hz ∈ [1.0 .. 1_000_000.0]
//   - center_freq_hz ∈ [0.1 .. sample_rate_hz/2.0)
//   - q_factor ∈ [0.01 .. 1000.0]
//   - input ∈ [-1.0e9 .. 1.0e9]

/// @outputs: 5
/// @domain: sample_rate_hz [1.0..1000000.0]
/// @domain: center_freq_hz [0.1..sample_rate_hz/2.0]
/// @domain: q_factor [0.01..1000.0]
///
/// Compute the normalized coefficients of a second‑order IIR notch filter
/// (Audio EQ Cookbook formulation). Returns `None` if any parameter is out
/// of its domain, non‑finite, or if the centre frequency is not strictly
/// lower than Nyquist.
pub fn biquad_notch_coefficients(
    sample_rate_hz: f64,
    center_freq_hz: f64,
    q_factor: f64,
) -> Option<[f64; 5]> {
    // Guard against out‑of‑range or non‑finite values
    if !(sample_rate_hz >= 1.0 && sample_rate_hz <= 1_000_000.0 && sample_rate_hz.is_finite()) {
        return None;
    }
    if !(center_freq_hz >= 0.1 && center_freq_hz < sample_rate_hz / 2.0 && center_freq_hz.is_finite())
    {
        return None;
    }
    if !(q_factor >= 0.01 && q_factor <= 1000.0 && q_factor.is_finite()) {
        return None;
    }

    // ω₀ = 2π·f₀ / fₛ
    let omega = 2.0 * std::f64::consts::PI * center_freq_hz / sample_rate_hz;
    // α = sin(ω₀) / (2Q)
    let alpha = omega.sin() / (2.0 * q_factor);

    // Un‑normalized coefficients (Audio EQ Cookbook)
    let b0 = 1.0;
    let b1 = -2.0 * omega.cos();
    let b2 = 1.0;
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

/// @outputs: 1
/// @domain: input [-1.0e9..1.0e9]
///
/// Perform one step of a Transposed Direct Form‑II biquad notch filter.
/// `state` holds the two internal delay elements. Returns `None` if any
/// intermediate result is not finite.
pub fn biquad_notch_step(
    input: f64,
    state: &mut [f64; 2],
    coeffs: &[f64; 5],
) -> Option<f64> {
    // Guard input domain and finiteness
    if !(input >= -1.0e9 && input <= 1.0e9 && input.is_finite()) {
        return None;
    }

    // y = b0·x + s0
    let y = coeffs[0] * input + state[0];
    if !y.is_finite() {
        return None;
    }

    // s0 = b1·x - a1·y + s1
    let new_s0 = coeffs[1] * input - coeffs[3] * y + state[1];
    if !new_s0.is_finite() {
        return None;
    }

    // s1 = b2·x - a2·y
    let new_s1 = coeffs[2] * input - coeffs[4] * y;
    if !new_s1.is_finite() {
        return None;
    }

    // Commit state updates
    state[0] = new_s0;
    state[1] = new_s1;

    Some(y)
}
