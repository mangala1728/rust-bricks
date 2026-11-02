// ## Map
// Role: Compute downside deviation and Sortino ratio for a series of returns.
// Inputs: returns: &[f64], target_return: f64, risk_free_rate: f64
// Outputs: Option<f64> (Some(value) when calculation is defined, None otherwise)
// Invariants: target_return ∈ [-1.0, 10.0], risk_free_rate ∈ [-1.0, 10.0],
//             returns.len() > 0 for a defined result,
/// Compute the downside deviation (semi‑deviation) of a return series.
///
/// # Parameters
/// - `returns`: slice of periodic returns (e.g. monthly returns).
/// - `target_return`: the benchmark return used to separate “down‑side” from “up‑side”.
///
/// # Returns
/// - `Some(D)` where `D` is the downside deviation, if `returns` is non‑empty.
/// - `None` if `returns` is empty or `target_return` is out of the allowed domain.
///
/// @outputs: 1
/// @domain: target_return [-1.0..10.0]
pub fn downside_deviation(returns: &[f64], target_return: f64) -> Option<f64> {
    // Guard: domain check for target_return
    if target_return < -1.0 || target_return > 10.0 {
        return None;
    }
    // Guard: empty input
    if returns.is_empty() {
        return None;
    }

    // Accumulate squared downside deviations
    let mut sum_sq = 0.0_f64;
    for &r in returns {
        let diff = r - target_return;
        // Only negative deviations contribute
        let downside = if diff < 0.0 { diff } else { 0.0 };
        sum_sq += downside * downside;
    }

    // Semi‑variance = average of squared downside deviations
    let semi_variance = sum_sq / (returns.len() as f64);
    // Downside deviation = sqrt(semi‑variance)
    Some(semi_variance.sqrt())
}

/// Compute the Sortino ratio of a return series.
///
/// # Parameters
/// - `returns`: slice of periodic returns.
/// - `risk_free_rate`: the risk‑free benchmark (used as target return).
///
/// # Returns
/// - `Some(SR)` where `SR` is the Sortino ratio, if a valid downside deviation exists
///   and is larger than a tiny epsilon.
/// - `None` if the input slice is empty, the risk‑free rate is out of domain,
///   or the downside deviation is effectively zero.
///
/// @outputs: 1
/// @domain: risk_free_rate [-1.0..10.0]
pub fn sortino_ratio(returns: &[f64], risk_free_rate: f64) -> Option<f64> {
    // Guard: domain check for risk_free_rate
    if risk_free_rate < -1.0 || risk_free_rate > 10.0 {
        return None;
    }
    // Guard: empty input
    if returns.is_empty() {
        return None;
    }

    // Compute arithmetic mean of returns
    let mut sum = 0.0_f64;
    for &r in returns {
        sum += r;
    }
    let mean = sum / (returns.len() as f64);

    // Compute downside deviation using the risk‑free rate as target
    let dd_opt = downside_deviation(returns, risk_free_rate);
    // Safe unwrap via match (no unwrap/expect)
    let dd = match dd_opt {
        Some(v) => v,
        None => return None,
    };

    // Avoid division by (near) zero downside deviation
    if dd <= 1.0e-12 {
        return None;
    }

    // Sortino ratio = excess mean return / downside deviation
    Some((mean - risk_free_rate) / dd)
}
