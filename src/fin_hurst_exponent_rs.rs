// ## Map
// Role: Rescaled Range (R/S) analysis and Hurst exponent estimation
// Inputs: series: &[f64]
// Outputs: Option<(f64, f64)> (rs_ratio, hurst_exponent)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: series.len() [8..=1024]
pub fn fin_compute_hurst_rs(series: &[f64]) -> Option<(f64, f64)> {
    // ---- domain guard -------------------------------------------------------
    if series.len() < 8 || series.len() > 1024 {
        return None;
    }

    // ---- finite‑value guard -------------------------------------------------
    if series.iter().any(|v| !v.is_finite()) {
        return None;
    }

    let n = series.len() as f64;

    // ---- mean ----------------------------------------------------------------
    let sum: f64 = series.iter().copied().sum();
    let mean = sum / n;

    // ---- variance ------------------------------------------------------------
    let var_sum: f64 = series.iter().map(|x| {
        let d = x - mean;
        d * d
    }).sum();
    let variance = var_sum / n;
    if variance <= 1e-14 {
        return None;
    }

    // ---- standard deviation --------------------------------------------------
    let s = variance.sqrt();

    // ---- cumulative deviation, range ----------------------------------------
    let mut cum = 0.0_f64;
    let mut max_z = 0.0_f64;
    let mut min_z = 0.0_f64;
    for &x in series {
        cum += x - mean;
        if cum > max_z { max_z = cum; }
        if cum < min_z { min_z = cum; }
    }
    let r = max_z - min_z;

    // ---- rescaled range & Hurst ---------------------------------------------
    let rs_ratio = r / s;
    let hurst = if rs_ratio <= 1.0 {
        0.5
    } else {
        (rs_ratio.ln() / n.ln()).clamp(0.0, 1.0)
    };

    Some((rs_ratio, hurst))
}
