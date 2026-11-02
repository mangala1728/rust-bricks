// Integration tests for dsp_ema_smoother module
use rust_bricks::dsp_ema_smoother::{dema_step_f64, ema_step_f64};

#[test]
fn test_ema_identity_and_invariance() {
    // Identity check: alpha = 1.0 -> output must strictly equal current
    let res_identity = ema_step_f64(42.5, 10.0, 1.0);
    assert!(res_identity.is_some());
    // @safety-proof: Safe unwrap confirmed by previous assertion on deterministic return.
    assert!((res_identity.unwrap() - 42.5).abs() < 1e-12);

    // Invariance check: current == prev -> output must remain unchanged
    let res_invariant = ema_step_f64(50.0, 50.0, 0.1);
    assert!(res_invariant.is_some());
    // @safety-proof: Safe unwrap confirmed by previous assertion on deterministic return.
    assert!((res_invariant.unwrap() - 50.0).abs() < 1e-12);
}

#[test]
fn test_ema_golden_value() {
    // Golden test: current = 100.0, prev = 0.0, alpha = 0.25 -> output = 25.0
    let res = ema_step_f64(100.0, 0.0, 0.25);
    assert!(res.is_some());
    // @safety-proof: Inputs are strictly within valid domain bounds.
    let val = res.unwrap();
    assert!((val - 25.0).abs() < 1e-6);
}

#[test]
fn test_ema_domain_boundary_rejection() {
    // Illegal alpha bounds
    assert_eq!(ema_step_f64(100.0, 0.0, 0.0), None);
    assert_eq!(ema_step_f64(100.0, 0.0, 0.00009), None);
    assert_eq!(ema_step_f64(100.0, 0.0, 1.5), None);
    assert_eq!(ema_step_f64(100.0, 0.0, f64::NAN), None);
    assert_eq!(ema_step_f64(100.0, 0.0, f64::INFINITY), None);

    // Out-of-domain current & prev values
    assert_eq!(ema_step_f64(1_000_001.0, 0.0, 0.5), None);
    assert_eq!(ema_step_f64(-1_000_001.0, 0.0, 0.5), None);
    assert_eq!(ema_step_f64(f64::NAN, 0.0, 0.5), None);
    assert_eq!(ema_step_f64(0.0, f64::INFINITY, 0.5), None);
}

#[test]
fn test_dema_golden_value() {
    // DEMA golden test verification
    let res = dema_step_f64(100.0, 50.0, 50.0, 0.5);
    assert!(res.is_some());
    // @safety-proof: Input domains strictly validated.
    let (dema, ema1, ema2) = res.unwrap();
    // ema1 = 0.5 * 100 + 0.5 * 50 = 75.0
    // ema2 = 0.5 * 75 + 0.5 * 50 = 62.5
    // dema = 2 * 75 - 62.5 = 87.5
    assert!((ema1 - 75.0).abs() < 1e-6);
    assert!((ema2 - 62.5).abs() < 1e-6);
    assert!((dema - 87.5).abs() < 1e-6);
}
