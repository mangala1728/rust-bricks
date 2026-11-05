// ## Map
// Role: Analytical Cramer-Lundberg ruin probability for exponential claims
// Inputs: initial_capital: f64, safety_loading: f64, mean_claim: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: safety_loading > 0.0, mean_claim > 0.0
pub fn fin_lundberg_adjustment_coefficient(safety_loading: f64, mean_claim: f64) -> Option<f64> {
    if safety_loading <= 0.0
        || mean_claim <= 0.0
        || !safety_loading.is_finite()
        || !mean_claim.is_finite()
    {
        return None;
    }
    let denom = mean_claim * (1.0 + safety_loading);
    if denom <= 0.0 {
        return None;
    }
    Some(safety_loading / denom)
}

/// @outputs: 1
/// @domain: initial_capital >= 0.0, safety_loading > 0.0, mean_claim > 0.0
pub fn fin_ruin_probability_exponential(
    initial_capital: f64,
    safety_loading: f64,
    mean_claim: f64,
) -> Option<f64> {
    if initial_capital < 0.0 || !initial_capital.is_finite() {
        return None;
    }
    let r = fin_lundberg_adjustment_coefficient(safety_loading, mean_claim)?;
    let base_prob = 1.0 / (1.0 + safety_loading);
    let exp_term = (-r * initial_capital).exp();
    Some(base_prob * exp_term)
}
