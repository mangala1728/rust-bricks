// ## Map
// Role: Compute Signal Magnitude Area (SMA) and classify activity intensity.
// Inputs: ax, ay, az slices of f64; sma f64, sedentary_threshold f64, active_threshold f64.
// Outputs: SMA value (Option<f64>); activity class (Option<u8>).
// Invariants:
//   - ax.len() == ay.len() == az.len() >= 1
//   - every element of ax, ay, az is finite
//   - sma >= 0.0, thresholds are finite, 0.0 <= sedentary_threshold <= active_threshold

/// @outputs: 1
/// @domain: ax length >= 1, ay length == ax length, az length == ax length, all values finite
pub fn sensor_sma(ax: &[f64], ay: &[f64], az: &[f64]) -> Option<f64> {
    // Guard: validate lengths and finiteness
    if ax.is_empty()
        || ax.len() != ay.len()
        || ax.len() != az.len()
        || ax.iter().chain(ay).chain(az).any(|v| !v.is_finite())
    {
        return None;
    }

    // Normalised Manhattan magnitude sum
    let n = ax.len() as f64;
    let sum: f64 = ax
        .iter()
        .zip(ay)
        .zip(az)
        .map(|((&x, &y), &z)| x.abs() + y.abs() + z.abs())
        .sum();

    Some(sum / n)
}

/// @outputs: 1
/// @domain: sma [0.0..], sedentary_threshold [0.0..], active_threshold [sedentary_threshold..]
pub fn sensor_classify_activity(
    sma: f64,
    sedentary_threshold: f64,
    active_threshold: f64,
) -> Option<u8> {
    // Guard: validate numeric pre‑conditions
    if !sma.is_finite()
        || sma < 0.0
        || !sedentary_threshold.is_finite()
        || !active_threshold.is_finite()
        || sedentary_threshold < 0.0
        || active_threshold < sedentary_threshold
    {
        return None;
    }

    // Three‑level classification
    if sma < sedentary_threshold {
        Some(0) // Sedentary
    } else if sma <= active_threshold {
        Some(1) // Light / Moderate
    } else {
        Some(2) // Vigorous
    }
}
