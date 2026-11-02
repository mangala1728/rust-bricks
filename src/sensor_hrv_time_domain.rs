// ## Map
// Role: Sensor HRV Time-Domain Metrics Processor
// Inputs: rr_intervals_ms (&[f64]) - RR interval values in milliseconds.
// Outputs: Option<(f64, f64, f64, f64)> - Tuple containing (Mean RR, SDNN, RMSSD, pNN50).
// Invariants:
// - Input slice length must be >= 2.
// - All RR intervals must strictly lie within [200.0, 3000.0].
// - Arithmetic calculations must maintain f64 precision without panics or undefined behavior.

/// Calculates HRV time-domain metrics: Mean RR, SDNN, RMSSD, and pNN50.
///
/// @outputs: 4
/// @domain: rr_intervals_ms [200.0..3000.0]
pub fn hrv_time_domain_metrics(rr_intervals_ms: &[f64]) -> Option<(f64, f64, f64, f64)> {
    if rr_intervals_ms.len() < 2 {
        return None;
    }

    for &rr in rr_intervals_ms {
        if rr < 200.0 || rr > 3000.0 || rr.is_nan() {
            return None;
        }
    }

    let n = rr_intervals_ms.len() as f64;
    let sum_rr: f64 = rr_intervals_ms.iter().sum();
    let mean_rr = sum_rr / n;

    let variance_sum: f64 = rr_intervals_ms
        .iter()
        .map(|&rr| {
            let diff = rr - mean_rr;
            diff * diff
        })
        .sum();
    let sdnn = (variance_sum / n).sqrt();

    let mut diff_sq_sum = 0.0;
    let mut nn50_count = 0.0;
    let num_diffs = n - 1.0;

    for i in 0..(rr_intervals_ms.len() - 1) {
        let diff = rr_intervals_ms[i + 1] - rr_intervals_ms[i];
        diff_sq_sum += diff * diff;
        if diff.abs() > 50.0 {
            nn50_count += 1.0;
        }
    }

    let rmssd = (diff_sq_sum / num_diffs).sqrt();
    let pnn50 = (nn50_count / num_diffs) * 100.0;

    Some((mean_rr, sdnn, rmssd, pnn50))
}
