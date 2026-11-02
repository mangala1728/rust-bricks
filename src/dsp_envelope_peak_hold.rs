// ## Map
// Role: DSP envelope follower & peak‑hold processor
// Inputs:
//   - dsp_envelope_follower: input (f64), prev_env (f64), attack_coeff (0.0..=1.0), release_coeff (0.0..=1.0)
//   - dsp_peak_hold_step: current_val (f64), peak (f64), hold_timer (u32), hold_duration (u32), decay_rate (0.0..=1.0)
// Outputs:
//   - dsp_envelope_follower → env (f64)
//   - dsp_peak_hold_step → (new_peak (f64), new_timer (u32))
// Invariants:
//   - All envelope / peak values are non‑negative and finite.
//   - Coefficients stay inside the closed interval [0.0, 1.0].
//   - hold_timer never exceeds hold_duration.

/// @outputs: 1
/// @domain: attack_coeff [0.0..=1.0]
/// @domain: release_coeff [0.0..=1.0]
pub fn dsp_envelope_follower(
    input: f64,
    prev_env: f64,
    attack_coeff: f64,
    release_coeff: f64,
) -> Option<f64> {
    // Guard: parameter sanity
    if !input.is_finite()
        || !prev_env.is_finite()
        || prev_env < 0.0
        || attack_coeff < 0.0
        || attack_coeff > 1.0
        || release_coeff < 0.0
        || release_coeff > 1.0
    {
        return None;
    }

    // Absolute value of the incoming sample
    let x = input.abs();

    // Choose the proper coefficient
    let coeff = if x > prev_env { attack_coeff } else { release_coeff };

    // Exponential attack / release step
    let env = prev_env + coeff * (x - prev_env);
    Some(env)
}

/// @outputs: 2
/// @domain: decay_rate [0.0..=1.0]
pub fn dsp_peak_hold_step(
    current_val: f64,
    peak: f64,
    hold_timer: u32,
    hold_duration: u32,
    decay_rate: f64,
) -> Option<(f64, u32)> {
    // Guard: parameter sanity
    if !current_val.is_finite()
        || !peak.is_finite()
        || peak < 0.0
        || current_val < 0.0
        || decay_rate < 0.0
        || decay_rate > 1.0
    {
        return None;
    }

    // New peak discovered → reset hold timer
    if current_val >= peak {
        return Some((current_val, hold_duration));
    }

    // Still holding the previous peak
    if hold_timer > 0 {
        return Some((peak, hold_timer - 1));
    }

    // Hold time elapsed → apply decay
    let new_peak = round_f64(peak * (1.0 - decay_rate), 12);
    Some((new_peak, 0))
}

// ---------------------------------------------------------------------------
// Helper – deterministic rounding to a fixed number of decimal places.
// This eliminates the tiny binary‑floating‑point artefacts that would
// otherwise break golden‑value tests.
// ---------------------------------------------------------------------------
fn round_f64(value: f64, places: u32) -> f64 {
    let factor = 10_f64.powi(places as i32);
    // The intermediate multiplication is safe because `factor` is a power of ten.
    // No unwrap / panic is used – `round` always returns a finite value for finite input.
    (value * factor).round() / factor
}
