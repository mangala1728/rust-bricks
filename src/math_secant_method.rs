// ## Map
// Role: 1D numerical root finding using secant line approximation
// Inputs: x_prev, x_curr, f_prev, f_curr: f64, tol: f64, max_iter: u32, f: Fn(f64)->f64
// Outputs: Option<f64> or Option<(f64, u32)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x_prev, x_curr, f_prev, f_curr are finite, |f_curr - f_prev| > 1e-15
pub fn math_secant_step(
    x_prev: f64,
    x_curr: f64,
    f_prev: f64,
    f_curr: f64,
) -> Option<f64> {
    if !x_prev.is_finite()
        || !x_curr.is_finite()
        || !f_prev.is_finite()
        || !f_curr.is_finite()
        || (f_curr - f_prev).abs() <= 1e-15
    {
        return None;
    }
    let df = f_curr - f_prev;
    let dx = x_curr - x_prev;
    let next = x_curr - f_curr * (dx / df);
    Some(next)
}

/// @outputs: 2
/// @domain: tol > 0.0, max_iter >= 1, x0 != x1, x0 and x1 are finite
pub fn math_secant_solve<F>(
    x0: f64,
    x1: f64,
    tol: f64,
    max_iter: u32,
    f: F,
) -> Option<(f64, u32)>
where
    F: Fn(f64) -> f64,
{
    if tol <= 0.0
        || max_iter == 0
        || (x1 - x0).abs() <= 1e-15
        || !x0.is_finite()
        || !x1.is_finite()
        || !tol.is_finite()
    {
        return None;
    }

    let mut curr_x0 = x0;
    let mut curr_x1 = x1;
    let mut f0 = f(curr_x0);
    let mut f1 = f(curr_x1);

    if !f0.is_finite() || !f1.is_finite() {
        return None;
    }

    for iter in 1..=max_iter {
        if f1.abs() <= tol {
            return Some((curr_x1, iter - 1));
        }

        let x_next = math_secant_step(curr_x0, curr_x1, f0, f1)?;
        let f_next = f(x_next);
        if !f_next.is_finite() {
            return None;
        }

        if (x_next - curr_x1).abs() <= tol || f_next.abs() <= tol {
            return Some((x_next, iter));
        }

        curr_x0 = curr_x1;
        f0 = f1;
        curr_x1 = x_next;
        f1 = f_next;
    }
    None
}
