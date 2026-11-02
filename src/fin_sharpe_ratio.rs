// ## Map
// Role: Compute annualized Sharpe and Information ratios.
// Inputs: returns slice, annual risk‑free rate, periods per year; portfolio slice, benchmark slice.
// Outputs: Option<f64> containing the requested ratio.
// Invariants: slices have length ≥ 2, periods_per_year > 0, all numbers are finite, denominator > 1e‑12.

/// Annualized Sharpe Ratio
///
/// @outputs: 1
/// @domain: returns.len() >= 2, periods_per_year > 0.0, annual_rf finite, returns finite
pub fn fin_sharpe_ratio(
    returns: &[f64],
    annual_rf: f64,
    periods_per_year: f64,
) -> Option<f64> {
    // domain guards (first statements)
    if returns.len() < 2 { return None; }
    if periods_per_year <= 0.0 || !periods_per_year.is_finite() { return None; }
    if !annual_rf.is_finite() { return None; }
    if returns.iter().any(|&r| !r.is_finite()) { return None; }

    // risk‑free rate per period
    let rf_period = annual_rf / periods_per_year;

    // mean return
    let mean_r = returns.iter().sum::<f64>() / (returns.len() as f64);

    // sample standard deviation (unbiased)
    let n = returns.len() as f64;
    let var = returns
        .iter()
        .map(|&r| {
            let d = r - mean_r;
            d * d
        })
        .sum::<f64>()
        / (n - 1.0);
    let stdev = var.sqrt();
    if stdev <= 1e-12 { return None; }

    // annualized Sharpe
    let sharpe = (mean_r - rf_period) / stdev * periods_per_year.sqrt();
    Some(sharpe)
}

/// Annualized Information Ratio
///
/// @outputs: 1
/// @domain: port.len() >= 2, port.len() == bench.len(), periods_per_year > 0.0, inputs finite
pub fn fin_information_ratio(
    port: &[f64],
    bench: &[f64],
    periods_per_year: f64,
) -> Option<f64> {
    // domain guards (first statements)
    if port.len() < 2 { return None; }
    if port.len() != bench.len() { return None; }
    if periods_per_year <= 0.0 || !periods_per_year.is_finite() { return None; }
    if port.iter().any(|&r| !r.is_finite()) { return None; }
    if bench.iter().any(|&r| !r.is_finite()) { return None; }

    // active return series
    let diffs: Vec<f64> = port.iter()
        .zip(bench.iter())
        .map(|(&p, &b)| p - b)
        .collect();

    // mean active return
    let mean_d = diffs.iter().sum::<f64>() / (diffs.len() as f64);

    // tracking error (sample std dev)
    let n = diffs.len() as f64;
    let var = diffs
        .iter()
        .map(|&d| {
            let d2 = d - mean_d;
            d2 * d2
        })
        .sum::<f64>()
        / (n - 1.0);
    let te = var.sqrt();
    if te <= 1e-12 { return None; }

    // annualized Information Ratio
    let ir = mean_d / te * periods_per_year.sqrt();
    Some(ir)
}
