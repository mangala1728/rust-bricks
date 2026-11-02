// ## Map
// Role: One-dimensional Golden-Section Search for unimodal function minimization.
// Inputs: Interval bounds `a` and `b`, tolerance `tol`, max iterations `max_iter`, target function `f`.
// Outputs: Option containing tuple `(x_min, f(x_min))` for estimated minimum point and value.
// Invariants: `a < b`, `tol` in [1.0e-12, 1.0], `max_iter` in [1, 1000], zero external dependencies, nesting depth <= 2.

/// @outputs: 2
/// @domain: tol [1.0e-12..1.0]
/// @domain: max_iter [1..1000]
pub fn golden_section_search_min<F>(
    mut a: f64,
    mut b: f64,
    tol: f64,
    max_iter: usize,
    f: F,
) -> Option<(f64, f64)>
where
    F: Fn(f64) -> f64,
{
    if a >= b || tol < 1.0e-12 || tol > 1.0 || max_iter < 1 || max_iter > 1000 {
        return None;
    }

    let r: f64 = (5.0_f64.sqrt() - 1.0) / 2.0;
    let mut c = b - r * (b - a);
    let mut d = a + r * (b - a);
    let mut fc = f(c);
    let mut fd = f(d);

    for _ in 0..max_iter {
        if (b - a) < tol {
            break;
        }

        if fc < fd {
            b = d;
            d = c;
            fd = fc;
            c = b - r * (b - a);
            fc = f(c);
        } else {
            a = c;
            c = d;
            fc = fd;
            d = a + r * (b - a);
            fd = f(d);
        }
    }

    let x_mid = (a + b) / 2.0;
    Some((x_mid, f(x_mid)))
}
