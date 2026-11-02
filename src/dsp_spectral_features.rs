// ## Map
// Role: Spectral feature extraction (centroid & rolloff)
// Inputs: magnitudes &[f64], sample_rate_hz f64, rolloff_ratio f64 (optional)
// Outputs: Option<f64> (spectral centroid or rolloff frequency)
// Invariants:
//   - sample_rate_hz ∈ [1.0 .. 1_000_000.0]
//   - rolloff_ratio ∈ [0.5 .. 0.99] (when used)
//   - magnitudes length N ≥ 2 for meaningful result
//   - total magnitude > 1.0e-12 for centroid calculation

/// @outputs: 1
/// @domain: sample_rate_hz [1.0..1000000.0]
///
/// Compute the spectral centroid of a magnitude spectrum.
///
/// Returns `None` if:
///   * `sample_rate_hz` is out of domain,
///   * the number of bins `N` is less than 2,
///   * the sum of magnitudes is ≤ 1.0e-12 (practically silent).
pub fn spectral_centroid(magnitudes: &[f64], sample_rate_hz: f64) -> Option<f64> {
    // domain guard
    if sample_rate_hz < 1.0 || sample_rate_hz > 1_000_000.0 {
        return None;
    }
    let n = magnitudes.len();
    if n < 2 {
        return None;
    }

    // compute total energy
    let mut sum_mag = 0.0_f64;
    for &m in magnitudes {
        sum_mag += m;
    }
    if sum_mag <= 1.0e-12 {
        return None;
    }

    // frequency resolution Δf = (fs/2) / (N-1)
    let delta_f = (sample_rate_hz / 2.0) / ((n - 1) as f64);

    // numerator Σ f_k * M[k]
    let mut weighted_sum = 0.0_f64;
    for (k, &m) in magnitudes.iter().enumerate() {
        weighted_sum += (k as f64) * delta_f * m;
    }

    Some(weighted_sum / sum_mag)
}

/// @outputs: 1
/// @domain: rolloff_ratio [0.5..0.99]
/// @domain: sample_rate_hz [1.0..1000000.0]
///
/// Compute the spectral roll‑off frequency.
///
/// Returns `None` if:
///   * any domain guard fails,
///   * the number of bins `N` is less than 2,
///   * the total magnitude is zero (cannot define a roll‑off).
pub fn spectral_rolloff(
    magnitudes: &[f64],
    sample_rate_hz: f64,
    rolloff_ratio: f64,
) -> Option<f64> {
    // domain guards
    if sample_rate_hz < 1.0 || sample_rate_hz > 1_000_000.0 {
        return None;
    }
    if rolloff_ratio < 0.5 || rolloff_ratio > 0.99 {
        return None;
    }
    let n = magnitudes.len();
    if n < 2 {
        return None;
    }

    // total energy E
    let mut total = 0.0_f64;
    for &m in magnitudes {
        total += m;
    }
    if total == 0.0 {
        return None;
    }

    // threshold T = E * rolloff_ratio
    let threshold = total * rolloff_ratio;

    // frequency resolution Δf
    let delta_f = (sample_rate_hz / 2.0) / ((n - 1) as f64);

    // cumulative sum until we cross the threshold
    let mut cumulative = 0.0_f64;
    for (k, &m) in magnitudes.iter().enumerate() {
        cumulative += m;
        if cumulative >= threshold {
            return Some((k as f64) * delta_f);
        }
    }

    // In theory we should have returned inside the loop,
    // but fall back to the Nyquist frequency.
    Some(sample_rate_hz / 2.0)
}
