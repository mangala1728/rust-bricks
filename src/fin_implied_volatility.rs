// ## Map
// Role: Black-Scholes analytical call pricing and bisection implied volatility inversion
// Inputs: s, k, r, t, sigma, market_price, tol: f64, max_iter: u32
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

use crate::fin_black_scholes::standard_normal_cdf;

/// @outputs: 1
/// @domain: s > 0.0, k > 0.0, t > 0.0, sigma > 0.0, r is finite
pub fn fin_bs_call_price(s: f64, k: f64, r: f64, t: f64, sigma: f64) -> Option<f64> {
    if s <= 0.0
        || k <= 0.0
        || t <= 0.0
        || sigma <= 0.0
        || !s.is_finite()
        || !k.is_finite()
        || !r.is_finite()
        || !t.is_finite()
        || !sigma.is_finite()
    {
        return None;
    }
    let sqrt_t = t.sqrt();
    let d1 = ((s / k).ln() + (r + 0.5 * sigma * sigma) * t) / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let nd1 = standard_normal_cdf(d1.clamp(-10.0, 10.0))?;
    let nd2 = standard_normal_cdf(d2.clamp(-10.0, 10.0))?;

    let call = s * nd1 - k * (-r * t).exp() * nd2;
    Some(call.max(0.0))
}

/// @outputs: 1
/// @domain: s > 0.0, k > 0.0, t > 0.0, tol > 0.0, market_price > 0.0, max_iter >= 1
pub fn fin_implied_volatility_bisection(
    market_price: f64,
    s: f64,
    k: f64,
    r: f64,
    t: f64,
    tol: f64,
    max_iter: u32,
) -> Option<f64> {
    if !market_price.is_finite()
        || !s.is_finite()
        || !k.is_finite()
        || !r.is_finite()
        || !t.is_finite()
        || !tol.is_finite()
        || s <= 0.0
        || k <= 0.0
        || t <= 0.0
        || market_price <= 0.0
        || tol <= 0.0
        || max_iter == 0
    {
        return None;
    }

    let lower_bound = (s - k * (-r * t).exp()).max(0.0);
    if market_price < lower_bound || market_price >= s {
        return None;
    }

    let mut low = 0.0001;
    let mut high = 5.0;

    for _ in 0..max_iter {
        let mid = 0.5 * (low + high);
        let price = fin_bs_call_price(s, k, r, t, mid)?;
        let diff = price - market_price;

        if diff.abs() <= tol {
            return Some(mid);
        }
        if diff < 0.0 {
            low = mid;
        } else {
            high = mid;
        }
    }
    Some(0.5 * (low + high))
}
