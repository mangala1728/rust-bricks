// ## Map: Role=Integrator, Inputs=[a: f64, b: f64, intervals: usize, f: Fn(f64)->f64],
// Outputs=[Option<f64>], Invariants=[a<b, intervals∈[2..100000], intervals even after adjustment]

/// Numerical integration using the composite Simpson's 1/3 rule (1‑D).
///
/// @outputs: 1
/// @domain: intervals [2..100000]
pub fn simpson_integral_1d<F>(a: f64, b: f64, intervals: usize, f: F) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    // Guard for input domain
    if !(a < b && a.is_finite() && b.is_finite()) {
        return None;
    }
    if intervals < 2 || intervals > 100_000 {
        return None;
    }

    // Ensure an even number of sub‑intervals
    let n = if intervals % 2 == 0 { intervals } else { intervals + 1 };

    // Step size
    let h = (b - a) / n as f64;

    // Initial sum with the end points
    let mut s = f(a) + f(b);

    // Accumulate interior points
    for i in 1..n {
        let x = a + i as f64 * h;
        if i % 2 == 1 {
            s += 4.0 * f(x);
        } else {
            s += 2.0 * f(x);
        }
    }

    // Final integral value
    let i_val = s * h / 3.0;

    // Validate result
    if i_val.is_finite() {
        Some(i_val)
    } else {
        None
    }
}
