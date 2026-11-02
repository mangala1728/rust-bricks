// ## Map
// Role: RiskMetrics EWMA dynamic volatility single-step and annualization calculator.
// Inputs: Previous variance, current return, decay factor (lambda), daily variance, trading days per year.
// Outputs: EWMA updated conditional variance or annualized volatility wrapped in Option<f64>.
// Invariants: Pure std, zero dependencies, no panic/unwrap/unsafe, max nesting depth <= 2, function length <= 60 lines.

/// EWMA 條件變異數單步更新
/// @outputs: 1
/// @domain: prev_variance [0.0..1.0e12]
/// @domain: current_return [-1.0..10.0]
/// @domain: lambda [0.5..0.999]
pub fn ewma_variance_step(
    prev_variance: f64,
    current_return: f64,
    lambda: f64,
) -> Option<f64> {
    if prev_variance < 0.0 || prev_variance > 1.0e12
        || current_return < -1.0 || current_return > 10.0
        || lambda < 0.5 || lambda > 0.999
    {
        return None;
    }

    let next_variance = lambda * prev_variance + (1.0 - lambda) * current_return * current_return;
    if next_variance.is_nan() || next_variance.is_infinite() {
        None
    } else {
        Some(next_variance)
    }
}

/// 計算年化波動率 (Annualized Volatility)
/// @outputs: 1
/// @domain: daily_variance [0.0..1.0e12]
/// @domain: trading_days_per_year [200.0..365.0]
pub fn ewma_annualized_volatility(
    daily_variance: f64,
    trading_days_per_year: f64,
) -> Option<f64> {
    if daily_variance < 0.0 || daily_variance > 1.0e12
        || trading_days_per_year < 200.0 || trading_days_per_year > 365.0
    {
        return None;
    }

    let ann_var = daily_variance * trading_days_per_year;
    if ann_var < 0.0 || ann_var.is_nan() || ann_var.is_infinite() {
        None
    } else {
        Some(ann_var.sqrt())
    }
}
