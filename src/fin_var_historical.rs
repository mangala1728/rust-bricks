// ## Map
// Role:   Financial risk analytics – historical simulation VaR & CVaR.
// Inputs: returns: &[f64] (historical portfolio returns)
//         confidence: f64 (confidence level, 0.0 < confidence < 1.0)
// Outputs: Option<f64> – Some(value) if inputs are valid, None otherwise.
// Invariants:
//   - returns.len() ≥ 2
//   - all returns are finite numbers
//   - confidence ∈ (0.0, 1.0) and finite
//   - Returned VaR/CVaR are non‑negative loss figures.

/// @outputs: 1
/// @domain: returns.len() >= 2, confidence in (0.0..1.0)
///
/// Historical Value‑at‑Risk (VaR) using the Historical Simulation method.
/// Returns the loss (as a non‑negative number) at the given confidence level,
/// or `None` if the input domain is violated.
pub fn fin_var_historical(returns: &[f64], confidence: f64) -> Option<f64> {
    // ---- domain guards -------------------------------------------------------
    if returns.len() < 2
        || !(confidence > 0.0 && confidence < 1.0 && confidence.is_finite())
    {
        return None;
    }
    if returns.iter().any(|&r| !r.is_finite()) {
        return None;
    }

    // ---- sort returns ascending ---------------------------------------------
    let mut sorted = returns.to_vec();
    // @safety-proof: `partial_cmp` only returns None for NaN, which we have filtered.
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // ---- compute VaR index ---------------------------------------------------
    let alpha = 1.0 - confidence;
    // tiny epsilon prevents floating‑point truncation on the boundary.
    let idx = (((alpha * (sorted.len() as f64)) + 1e-9).floor() as usize)
        .min(sorted.len() - 1);

    // ---- loss convention (non‑negative) --------------------------------------
    Some(-sorted[idx])
}

/// @outputs: 1
/// @domain: returns.len() >= 2, confidence in (0.0..1.0)
///
/// Historical Conditional Value‑at‑Risk (CVaR, Expected Shortfall) using the
/// Historical Simulation method. Returns the average loss of the worst
/// `(1‑confidence)`‑fraction of observations, or `None` if the input domain is
/// violated.
pub fn fin_cvar_historical(returns: &[f64], confidence: f64) -> Option<f64> {
    // ---- domain guards -------------------------------------------------------
    if returns.len() < 2
        || !(confidence > 0.0 && confidence < 1.0 && confidence.is_finite())
    {
        return None;
    }
    if returns.iter().any(|&r| !r.is_finite()) {
        return None;
    }

    // ---- sort returns ascending ---------------------------------------------
    let mut sorted = returns.to_vec();
    // @safety-proof: same reasoning as in `fin_var_historical`.
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // ---- number of tail observations ----------------------------------------
    let alpha = 1.0 - confidence;
    let n = sorted.len() as f64;
    // `ceil` gives the smallest integer ≥ α·N; no epsilon is added (see spec).
    let count = ((alpha * n).ceil() as usize).max(1).min(sorted.len());

    // ---- average loss of the worst `count` returns ---------------------------
    let sum_loss: f64 = sorted[..count].iter().map(|&r| -r).sum();
    Some(sum_loss / (count as f64))
}
