use rust_bricks::dsp_envelope_follower::*;

#[test]
fn test_golden_vectors() {
    let res_attack = dsp_envelope_follower_step(-2.0, 0.0, 0.8, 0.2);
    // @safety-proof: Valid inputs guarantee Some variant
    assert!((res_attack.unwrap() - 0.4).abs() < 1e-12);

    let res_release = dsp_envelope_follower_step(0.0, 1.0, 0.8, 0.9);
    // @safety-proof: Valid inputs guarantee Some variant
    assert!((res_release.unwrap() - 0.9).abs() < 1e-12);

    assert!(dsp_envelope_follower_step(1.0, -0.1, 0.8, 0.2).is_none());
    assert!(dsp_envelope_follower_step(1.0, 0.5, 1.2, 0.2).is_none());
    assert!(dsp_envelope_follower_step(f64::NAN, 0.5, 0.8, 0.2).is_none());
}

#[test]
fn test_domain_boundary_guards() {
    // Out of range input sample
    assert!(dsp_envelope_follower_step(1e6 + 1.0, 0.0, 0.5, 0.5).is_none());
    assert!(dsp_envelope_follower_step(-1e6 - 1.0, 0.0, 0.5, 0.5).is_none());

    // Infinities
    assert!(dsp_envelope_follower_step(f64::INFINITY, 0.0, 0.5, 0.5).is_none());
    assert!(dsp_envelope_follower_step(0.0, f64::INFINITY, 0.5, 0.5).is_none());
    assert!(dsp_envelope_follower_step(0.0, 0.0, f64::NAN, 0.5).is_none());
    assert!(dsp_envelope_follower_step(0.0, 0.0, 0.5, f64::NEG_INFINITY).is_none());

    // Negative release coefficient
    assert!(dsp_envelope_follower_step(0.5, 0.0, 0.5, -0.01).is_none());
}

#[test]
fn test_edge_coefficients() {
    // Immediate attack (attack_coeff = 0.0)
    let res_fast_attack = dsp_envelope_follower_step(0.75, 0.0, 0.0, 0.5);
    // @safety-proof: Boundary coefficient inside domain [0.0..=1.0]
    assert!((res_fast_attack.unwrap() - 0.75).abs() < 1e-12);

    // No release decay (release_coeff = 1.0)
    let res_hold_release = dsp_envelope_follower_step(0.0, 0.8, 0.5, 1.0);
    // @safety-proof: Boundary coefficient inside domain [0.0..=1.0]
    assert!((res_hold_release.unwrap() - 0.8).abs() < 1e-12);
}
