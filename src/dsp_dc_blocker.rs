// ## Map
// Role: First‑order IIR DC blocker (DSP brick)
// Inputs: x, x_prev, y_prev, r_coeff  (step) | samples slice, r_coeff (slice)
// Outputs: y, updated states (step) | filtered slice (slice)
// Invariants: r_coeff ∈ [0.0, 1.0) , all numeric arguments are finite, output is finite

/// @outputs: 3
/// @domain: r_coeff [0.0..1.0)
pub fn dsp_dc_blocker_step(
    x: f64,
    x_prev: f64,
    y_prev: f64,
    r_coeff: f64,
) -> Option<(f64, f64, f64)> {
    // Guard: all arguments must be finite and r_coeff in the allowed range
    if !x.is_finite()
        || !x_prev.is_finite()
        || !y_prev.is_finite()
        || !r_coeff.is_finite()
        || r_coeff < 0.0
        || r_coeff >= 1.0
    {
        return None;
    }

    // Core IIR equation: y[n] = x[n] - x[n‑1] + r·y[n‑1]
    let y = x - x_prev + r_coeff * y_prev;

    // Return the new output together with the state needed for the next step
    Some((y, x, y))
}

/// @outputs: 1
/// @domain: r_coeff [0.0..1.0)
pub fn dsp_dc_blocker_slice(samples: &mut [f64], r_coeff: f64) -> bool {
    // Guard: non‑empty slice and valid coefficient
    if samples.is_empty()
        || !r_coeff.is_finite()
        || r_coeff < 0.0
        || r_coeff >= 1.0
    {
        return false;
    }

    // Guard: every sample must be a finite number
    if samples.iter().any(|&s| !s.is_finite()) {
        return false;
    }

    // Initialise history
    let mut x_prev = 0.0_f64;
    let mut y_prev = 0.0_f64;

    // In‑place filtering
    for sample in samples.iter_mut() {
        let x = *sample;
        let y = x - x_prev + r_coeff * y_prev;
        x_prev = x;
        y_prev = y;
        *sample = y;
    }

    true
}
