// ## Map
// Role: Portfolio maximum drawdown (MDD) and underwater duration analysis
// Inputs: equity_curve: &[f64]
// Outputs: Option<(f64, usize, f64)> (max_drawdown_pct, max_underwater_len, current_drawdown_pct)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: equity_curve.len() in [1..=2048]
pub fn fin_compute_drawdown_metrics(
    equity_curve: &[f64],
) -> Option<(f64, usize, f64)> {
    // Domain guard: length must be within 1..=2048
    if equity_curve.is_empty() || equity_curve.len() > 2048 {
        return None;
    }

    // Validate all values are positive finite numbers
    for &v in equity_curve {
        if !(v > 0.0) || !v.is_finite() {
            return None;
        }
    }

    // Initialise tracking variables
    let mut peak = equity_curve[0];
    let mut max_dd = 0.0_f64;
    let mut current_underwater = 0usize;
    let mut max_underwater = 0usize;

    // Single explicit loop over the curve
    for &v in equity_curve.iter() {
        if v >= peak {
            peak = v;
            current_underwater = 0;
        } else {
            current_underwater += 1;
            if current_underwater > max_underwater {
                max_underwater = current_underwater;
            }
            let dd = (peak - v) / peak * 100.0;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }

    // Current drawdown based on the last value
    let last = *equity_curve.last().unwrap(); // @safety-proof
    let curr_dd = (peak - last) / peak * 100.0;

    Some((max_dd, max_underwater, curr_dd))
}
