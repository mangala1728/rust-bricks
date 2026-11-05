// ## Map
// Role: GARCH(1,1) conditional variance recursion and long-term equilibrium variance
// Inputs: omega, alpha, beta, prev_var, prev_return: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: omega > 0.0, alpha >= 0.0, beta >= 0.0, alpha + beta < 1.0, prev_var > 0.0
pub fn fin_garch11_step(
    omega: f64,
    alpha: f64,
    beta: f64,
    prev_var: f64,
    prev_return: f64,
) -> Option<f64> {
    if omega <= 0.0
        || alpha < 0.0
        || beta < 0.0
        || (alpha + beta) >= 1.0
        || prev_var <= 0.0
        || !omega.is_finite()
        || !alpha.is_finite()
        || !beta.is_finite()
        || !prev_var.is_finite()
        || !prev_return.is_finite()
    {
        return None;
    }
    let next_var = omega + alpha * prev_return * prev_return + beta * prev_var;
    Some(next_var)
}

/// @outputs: 1
/// @domain: omega > 0.0, alpha >= 0.0, beta >= 0.0, alpha + beta < 1.0
pub fn fin_garch11_unconditional_variance(omega: f64, alpha: f64, beta: f64) -> Option<f64> {
    if omega <= 0.0
        || alpha < 0.0
        || beta < 0.0
        || (alpha + beta) >= 1.0
        || !omega.is_finite()
        || !alpha.is_finite()
        || !beta.is_finite()
    {
        return None;
    }
    let denom = 1.0 - alpha - beta;
    if denom <= 1e-12 {
        return None;
    }
    Some(omega / denom)
}

/// @outputs: 1
/// @domain: returns.len() [1..1000000]
pub fn fin_garch11_series(
    omega: f64,
    alpha: f64,
    beta: f64,
    initial_var: f64,
    returns: &[f64],
    variances_out: &mut [f64],
) -> usize {
    if returns.is_empty() || variances_out.len() < returns.len() {
        return 0;
    }
    let mut current_var = initial_var;
    for (i, &r) in returns.iter().enumerate() {
        match fin_garch11_step(omega, alpha, beta, current_var, r) {
            Some(v) => {
                variances_out[i] = v;
                current_var = v;
            }
            None => return i,
        }
    }
    returns.len()
}
