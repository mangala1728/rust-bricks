// ## Map
// Role:   Compute financial risk metrics (Maximum Drawdown & Calmar Ratio)
// Inputs: prices: &[f64] (net‑asset values), annualized_return: f64, max_drawdown: f64
// Outputs: Option<f64> (metric value)
// Invariants: prices.len() >= 2, each price > 0.0, max_drawdown ∈ (0.0, 1.0], all f64 values are finite

/// @outputs: 1
/// @domain: prices.len() >= 2, each price > 0.0
///
/// Compute the Maximum Drawdown (MDD) of a net‑value series.
/// Returns `None` if the input does not satisfy the domain constraints.
pub fn fin_max_drawdown(prices: &[f64]) -> Option<f64> {
    // Guard: length & positivity check
    if prices.len() < 2 || prices.iter().any(|&p| !p.is_finite() || p <= 0.0) {
        return None;
    }

    // Single‑pass algorithm: track running peak and maximal relative drop
    let mut peak = prices[0];
    let mut max_dd = 0.0_f64;

    for &price in prices.iter().skip(1) {
        if price > peak {
            peak = price;
        } else {
            let dd = (peak - price) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    Some(max_dd)
}

/// @outputs: 1
/// @domain: max_drawdown [0.0..=1.0]
///
/// Compute the Calmar Ratio given an annualized return and a maximum drawdown.
/// Returns `None` if the inputs violate the domain constraints.
pub fn fin_calmar_ratio(annualized_return: f64, max_drawdown: f64) -> Option<f64> {
    // Guard: finiteness and drawdown bounds
    if !annualized_return.is_finite()
        || !max_drawdown.is_finite()
        || max_drawdown <= 1e-9
        || max_drawdown > 1.0
    {
        return None;
    }

    // Calmar = annualized_return / max_drawdown
    Some(annualized_return / max_drawdown)
}
