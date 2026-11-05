// ## Map
// Role: Downside semi-deviation and upside potential ratio below MAR target
// Inputs: returns: &[f64], target_mar: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: returns.len() [1..1000000]
pub fn fin_downside_deviation(returns: &[f64], target_mar: f64) -> Option<f64> {
    if returns.is_empty() || returns.len() > 1_000_000 || !target_mar.is_finite() {
        return None;
    }
    let mut sum_sq = 0.0;
    for &r in returns {
        if !r.is_finite() {
            return None;
        }
        let diff = r - target_mar;
        if diff < 0.0 {
            sum_sq += diff * diff;
        }
    }
    let variance = sum_sq / (returns.len() as f64);
    Some(variance.sqrt())
}

/// @outputs: 1
/// @domain: returns.len() [1..1000000]
pub fn fin_upside_potential_ratio(returns: &[f64], target_mar: f64) -> Option<f64> {
    let down_dev = fin_downside_deviation(returns, target_mar)?;
    if down_dev <= 1e-12 {
        return None;
    }
    let mut upside_sum = 0.0;
    for &r in returns {
        let diff = r - target_mar;
        if diff > 0.0 {
            upside_sum += diff;
        }
    }
    let upside_mean = upside_sum / (returns.len() as f64);
    Some(upside_mean / down_dev)
}
