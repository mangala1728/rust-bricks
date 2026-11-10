// ## Map
// Role: Chamberlin Digital State Variable Filter single-step computation
// Inputs: input: f64, s1: f64, s2: f64, f: f64, q: f64
// Outputs: Option<(f64, f64, f64, f64, f64, f64)> (lp, bp, hp, notch, next_s1, next_s2)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 6
/// @domain: input in [-1e6..=1e6], s1 in [-1e6..=1e6], s2 in [-1e6..=1e6], f in [0.0001..=0.8], q in [0.0001..=2.0]
pub fn dsp_svf_step(
    input: f64,
    s1: f64,
    s2: f64,
    f: f64,
    q: f64,
) -> Option<(f64, f64, f64, f64, f64, f64)> {
    if !input.is_finite()
        || !s1.is_finite()
        || !s2.is_finite()
        || !f.is_finite()
        || !q.is_finite()
        || !( -1e6..=1e6 ).contains(&input)
        || !( -1e6..=1e6 ).contains(&s1)
        || !( -1e6..=1e6 ).contains(&s2)
        || !( 0.0001..=0.8 ).contains(&f)
        || !( 0.0001..=2.0 ).contains(&q)
    {
        return None;
    }

    let hp = input - s1 - q * s2;
    let bp = s2 + f * hp;
    let lp = s1 + f * bp;
    let notch = hp + lp;
    let next_s1 = lp;
    let next_s2 = bp;

    if !hp.is_finite() || !bp.is_finite() || !lp.is_finite() || !notch.is_finite() {
        return None;
    }

    Some((lp, bp, hp, notch, next_s1, next_s2))
}
