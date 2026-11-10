// ## Map
// Role: Merton Jump Diffusion jump amplitude and Poisson weight calculator
// Inputs: lambda, t, mu_j, delta: f64, n: u32
// Outputs: Option<f64> or Option<(f64, f64)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: lambda >= 0.0, t > 0.0, n <= 30
pub fn fin_merton_poisson_weight(lambda: f64, t: f64, n: u32) -> Option<f64> {
    // @safety-proof: guard against invalid domain
    if lambda < 0.0
        || t <= 0.0
        || n > 30
        || lambda.is_nan()
        || t.is_nan()
    {
        return None;
    }

    let lt = lambda * t;

    // tiny intensity → deterministic outcome
    if lt < 1e-15 {
        return if n == 0 { Some(1.0) } else { Some(0.0) };
    }

    // log‑factorial via summation (stable for n ≤ 30)
    let mut ln_fact = 0.0_f64;
    let mut i = 2_u32;
    while i <= n {
        ln_fact += (i as f64).ln();
        i += 1;
    }

    // ln P(N_T=n) = -λt + n·ln(λt) – ln(n!)
    let ln_weight = -lt + (n as f64) * lt.ln() - ln_fact;
    Some(ln_weight.exp())
}

/// @outputs: 2
/// @domain: lambda >= 0.0, mu_j in [-2.0..=2.0], delta >= 0.0
pub fn fin_merton_jump_parameters(lambda: f64, mu_j: f64, delta: f64) -> Option<(f64, f64)> {
    // @safety-proof: guard against invalid domain
    if lambda < 0.0
        || delta < 0.0
        || !mu_j.is_finite()
        || lambda.is_nan()
        || delta.is_nan()
        || mu_j.is_nan()
    {
        return None;
    }

    // κ = exp(μ_j + ½δ²) – 1
    let kappa = (mu_j + delta * delta / 2.0).exp() - 1.0;
    // λ' = λ·(1+κ)
    let lambda_prime = lambda * (1.0 + kappa);
    Some((kappa, lambda_prime))
}
