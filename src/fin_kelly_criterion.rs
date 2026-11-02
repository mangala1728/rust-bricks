// ## Map
// Role: Compute Kelly optimal fraction (full and fractional)
// Inputs: win_prob (f64) in [0.01..0.99], odds_b (f64) in [0.01..1000.0], fraction (f64) in [0.01..1.0]
// Outputs: Option<f64> where Some(value) is the position size in [0.0..1.0]
// Invariants: result ∈ [0.0, 1.0]; None is returned for any out‑of‑domain argument

/// @outputs: 1
/// @domain: win_prob [0.01..0.99]
/// @domain: odds_b [0.01..1000.0]
/// @domain: fraction [0.01..1.0]
pub fn kelly_optimal_fraction(
    win_prob: f64,
    odds_b: f64,
    fraction: f64,
) -> Option<f64> {
    // domain guards
    if win_prob < 0.01 || win_prob > 0.99 {
        return None;
    }
    if odds_b < 0.01 || odds_b > 1000.0 {
        return None;
    }
    if fraction < 0.01 || fraction > 1.0 {
        return None;
    }

    // q = probability of loss
    let q = 1.0 - win_prob;
    // full Kelly fraction f* = win_prob - q / odds_b
    let f_star = win_prob - q / odds_b;

    // if expectation ≤ 0 → no position
    let base = if f_star <= 0.0 { 0.0 } else { f_star };

    // apply fractional Kelly
    let scaled = fraction * base;

    // clamp to [0.0, 1.0]
    let clamped = if scaled < 0.0 {
        0.0
    } else if scaled > 1.0 {
        1.0
    } else {
        scaled
    };

    Some(clamped)
}
