// ## Map
// Role: Bollinger Bands channel computation with %B and Bandwidth indicators
// Inputs: prices: &[f64], k: f64
// Outputs: Option<(f64, f64, f64, f64, f64)> (middle, upper, lower, percent_b, bandwidth)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 5
/// @domain: prices.len() in [2..=256], k > 0.0, prices > 0.0
pub fn fin_compute_bollinger_bands(
    prices: &[f64],
    k: f64,
) -> Option<(f64, f64, f64, f64, f64)> {
    if prices.len() < 2 || prices.len() > 256 || k <= 0.0 || !k.is_finite() {
        return None;
    }

    let mut sum = 0.0;
    for &p in prices {
        if p <= 0.0 || !p.is_finite() {
            return None;
        }
        sum += p;
    }

    let n = prices.len() as f64;
    let middle = sum / n;

    let mut var_sum = 0.0;
    for &p in prices {
        let diff = p - middle;
        var_sum += diff * diff;
    }

    let variance = var_sum / n;
    let std_dev = variance.sqrt();

    let upper = middle + k * std_dev;
    let lower = middle - k * std_dev;

    let latest = match prices.last() {
        Some(&p) => p,
        None => return None,
    };

    let band_diff = upper - lower;
    let bandwidth = band_diff / middle;

    let percent_b = if band_diff < 1e-12 {
        0.5
    } else {
        (latest - lower) / band_diff
    };

    Some((middle, upper, lower, percent_b, bandwidth))
}
