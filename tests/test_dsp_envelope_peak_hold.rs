// ## Map
// Role: Integration test for dsp_envelope_peak_hold module
// Inputs: see individual test cases
// Outputs: deterministic golden values
// Invariants: all returned Options must be Some(...)

use rust_bricks::dsp_envelope_peak_hold::*;

/// Attack phase – the envelope should rise towards the absolute input.
#[test]
fn envelope_attack_phase() {
    let result = dsp_envelope_follower(1.0, 0.0, 0.5, 0.1);
    assert_eq!(result, Some(0.5));
}

/// Release phase – the envelope should fall towards the absolute input.
#[test]
fn envelope_release_phase() {
    let result = dsp_envelope_follower(0.0, 0.5, 0.5, 0.2);
    assert_eq!(result, Some(0.4));
}

/// Peak‑hold freeze – while the hold timer is > 0 the peak stays unchanged.
#[test]
fn peak_hold_freeze() {
    let result = dsp_peak_hold_step(0.2, 0.8, 2, 5, 0.1);
    assert_eq!(result, Some((0.8, 1)));
}

/// Peak‑hold decay – after the timer reaches zero the peak decays deterministically.
#[test]
fn peak_hold_decay() {
    let result = dsp_peak_hold_step(0.2, 0.8, 0, 5, 0.1);
    // 0.8 * (1.0 - 0.1) = 0.72 exactly after rounding.
    assert_eq!(result, Some((0.72, 0)));
}
