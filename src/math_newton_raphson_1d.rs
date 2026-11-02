// ## Map: Role=Solver, Inputs=initial_x, tol, max_iter, f, df, Outputs=root, Invariants=convergence, no_external_deps

/// @outputs: 1
/// @domain: tol [1.0e-12..1.0]
/// @domain: max_iter [1..1000]
///
/// One‑dimensional Newton‑Raphson root finder.
///
/// Returns `Some(root)` when the algorithm converges within `max_iter`
/// iterations and the supplied tolerances, otherwise `None`.
pub fn newton_raphson_1d<F, DF>(
    initial_x: f64,
    tol: f64,
    max_iter: usize,
    f: F,
    df: DF,
) -> Option<f64>
where
    F: Fn(f64) -> f64,
    DF: Fn(f64) -> f64,
{
    // Guard: input validation
    if !initial_x.is_finite()
        || tol < 1.0e-12
        || tol > 1.0
        || max_iter < 1
        || max_iter > 1000
    {
        return None;
    }

    let mut x = initial_x;

    for _ in 0..max_iter {
        let y = f(x);
        if y.abs() < tol {
            return Some(x);
        }

        let dy = df(x);
        if dy.abs() < 1.0e-12 || dy.is_nan() {
            return None;
        }

        let step = y / dy;
        let x_next = x - step;
        if (x_next - x).abs() < tol {
            return Some(x_next);
        }

        x = x_next;
    }

    None
}
