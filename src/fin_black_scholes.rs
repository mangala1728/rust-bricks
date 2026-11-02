// ## Map
// Role:  Black‑Scholes analytical pricing & Greeks (pure std)
// Inputs:  s0, strike, rate, sigma, t_years, x
// Outputs:  (call_price, put_price, call_delta, put_delta) or CDF value
// Invariants: all numeric inputs must lie inside the documented domains

/// @outputs: 1
/// @domain: x [-10.0..10.0]
pub fn standard_normal_cdf(x: f64) -> Option<f64> {
    // guard domain
    if x < -10.0 || x > 10.0 {
        return None;
    }

    let z = x.abs();
    let t = 1.0 / (1.0 + 0.2316419 * z);

    // coefficients from Abramowitz & Stegun 7.1.26
    const B1: f64 = 0.319381530;
    const B2: f64 = -0.356563782;
    const B3: f64 = 1.781477937;
    const B4: f64 = -1.821255978;
    const B5: f64 = 1.330274429;

    // pdf = (1/√(2π))·e^{‑z²/2}
    let pdf = (-(z * z) / 2.0).exp() / (2.0_f64 * std::f64::consts::PI).sqrt();

    // polynomial part
    let poly = B1 * t
        + B2 * t * t
        + B3 * t * t * t
        + B4 * t * t * t * t
        + B5 * t * t * t * t * t;

    let p = pdf * poly; // = 1 - Φ(z) for z ≥ 0

    // @safety-proof: unwrap is safe because we already validated the domain
    if x >= 0.0 {
        Some(1.0 - p)
    } else {
        Some(p)
    }
}

/// @outputs: 4
/// @domain: s0 [0.001..1.0e9]
/// @domain: strike [0.001..1.0e9]
/// @domain: rate [-0.5..1.0]
/// @domain: sigma [0.001..10.0]
/// @domain: t_years [0.001..100.0]
pub fn black_scholes_european(
    s0: f64,
    strike: f64,
    rate: f64,
    sigma: f64,
    t_years: f64,
) -> Option<(f64, f64, f64, f64)> {
    // guard domains
    if s0 < 0.001 || s0 > 1.0e9 { return None; }
    if strike < 0.001 || strike > 1.0e9 { return None; }
    if rate < -0.5 || rate > 1.0 { return None; }
    if sigma < 0.001 || sigma > 10.0 { return None; }
    if t_years < 0.001 || t_years > 100.0 { return None; }

    // helper: safe CDF that falls back to 0/1 outside [-10,10]
    fn cdf(x: f64) -> f64 {
        match standard_normal_cdf(x) {
            Some(v) => v,
            None => if x > 0.0 { 1.0 } else { 0.0 },
        }
    }

    let sqrt_t = t_years.sqrt();
    let d1 = ( (s0 / strike).ln() + (rate + 0.5 * sigma * sigma) * t_years )
        / (sigma * sqrt_t);
    let d2 = d1 - sigma * sqrt_t;

    let nd1 = cdf(d1);
    let nd2 = cdf(d2);
    let n_minus_d1 = cdf(-d1);
    let n_minus_d2 = cdf(-d2);

    let discount = (-rate * t_years).exp();

    // price formulas
    let call_price = s0 * nd1 - strike * discount * nd2;
    let put_price  = strike * discount * n_minus_d2 - s0 * n_minus_d1;

    // Greeks
    let call_delta = nd1;
    let put_delta  = nd1 - 1.0;

    Some((call_price, put_price, call_delta, put_delta))
}
