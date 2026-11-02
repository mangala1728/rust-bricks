// ## Map
// Role:   DSP Goertzel single‑tone detector (pure Rust, no external crates)
// Inputs: target_freq_hz: f64, sample_rate_hz: f64, samples: &[f64], coeff: f64
// Outputs: Option<f64> (coeff), Option<(f64, f64)> (power, magnitude)
// Invariants:
//   - 0.0 < target_freq_hz < sample_rate_hz / 2.0
//   - |coeff| ≤ 2.0  (derived from cosine bound)
//   - samples length may be any non‑zero size
//   - power ≥ 0.0, magnitude = sqrt(power)

/// @outputs: 1
/// @domain: target_freq_hz [1.0..500000.0]
/// @domain: sample_rate_hz [2.0..1000000.0]
///
/// Compute the Goertzel coefficient `2·cos(ω)`.
/// Returns `None` if the frequency is out of the Nyquist range or inputs are invalid.
pub fn goertzel_coeff(target_freq_hz: f64, sample_rate_hz: f64) -> Option<f64> {
    // @safety-proof: simple numeric guards, no unsafe code
    if target_freq_hz < 1.0
        || target_freq_hz > 500_000.0
        || sample_rate_hz < 2.0
        || sample_rate_hz > 1_000_000.0
        || target_freq_hz >= sample_rate_hz / 2.0
    {
        return None;
    }

    // ω = 2π·f₀ / fₛ
    let omega = std::f64::consts::TAU * target_freq_hz / sample_rate_hz;
    // coeff = 2·cos(ω)
    Some(2.0 * omega.cos())
}

/// @outputs: 2
/// @domain: coeff [-2.0..2.0]
///
/// Run the Goertzel algorithm on a block of samples.
/// Returns `(power, magnitude)` or `None` on invalid input.
pub fn goertzel_detect(samples: &[f64], coeff: f64) -> Option<(f64, f64)> {
    // @safety-proof: guard against empty input and out‑of‑range coefficient
    if samples.is_empty() || coeff.abs() > 2.0 {
        return None;
    }

    let mut s1 = 0.0_f64;
    let mut s2 = 0.0_f64;

    for &x in samples.iter() {
        // s0 = x + coeff·s1 - s2
        let s0 = x + coeff * s1 - s2;
        s2 = s1;
        s1 = s0;
    }

    // Power = s1² + s2² - coeff·s1·s2
    let mut power = s1 * s1 + s2 * s2 - coeff * s1 * s2;
    if power < 0.0 {
        power = 0.0;
    }
    let magnitude = power.sqrt();

    Some((power, magnitude))
}
