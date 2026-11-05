// ## Map
// Role: ECG R-peak baseline amplitude extraction and respiration rate estimation
// Inputs: ecg_samples: &[f64], peak_idx: usize, baseline: f64, intervals: &[f64]
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: peak_idx < ecg_samples.len(), baseline is finite
pub fn sensor_edr_r_peak_amplitude(
    ecg_samples: &[f64],
    peak_idx: usize,
    baseline: f64,
) -> Option<f64> {
    if peak_idx >= ecg_samples.len() || !baseline.is_finite() || !ecg_samples[peak_idx].is_finite() {
        return None;
    }
    Some(ecg_samples[peak_idx] - baseline)
}

/// @outputs: 1
/// @domain: intervals.len() in [2..100000], all elements in [1.0..=10.0]
pub fn sensor_respiration_rate_from_intervals(intervals: &[f64]) -> Option<f64> {
    if intervals.len() < 2 || intervals.len() > 100_000 {
        return None;
    }
    let mut sum = 0.0;
    for &t in intervals {
        if !t.is_finite() || t < 1.0 || t > 10.0 {
            return None;
        }
        sum += t;
    }
    let mean_t = sum / (intervals.len() as f64);
    if mean_t <= 0.0 {
        return None;
    }
    Some(60.0 / mean_t)
}
