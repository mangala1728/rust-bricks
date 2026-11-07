use rust_bricks::dsp_preemphasis_deemphasis::*;

#[test]
fn test_dsp_preemphasis_deemphasis_golden() {
    // @safety-proof: Input values (10.0, 8.0, 0.95) are guaranteed within domain bounds
    let pre = dsp_preemphasis_step(10.0, 8.0, 0.95).unwrap();
    assert!((pre - 2.4).abs() < 1e-12);

    // @safety-proof: Input values (2.4, 8.0, 0.95) are guaranteed within domain bounds
    let de = dsp_deemphasis_step(2.4, 8.0, 0.95).unwrap();
    assert!((de - 10.0).abs() < 1e-12);

    assert!(dsp_preemphasis_step(10.0, 8.0, 1.1).is_none());
    assert!(dsp_deemphasis_step(10.0, 8.0, -0.1).is_none());
}

#[test]
fn test_dsp_preemphasis_deemphasis_boundary_guards() {
    // Test NaN and Infinities
    assert!(dsp_preemphasis_step(f64::NAN, 0.0, 0.95).is_none());
    assert!(dsp_preemphasis_step(0.0, f64::INFINITY, 0.95).is_none());
    assert!(dsp_preemphasis_step(0.0, 0.0, f64::NAN).is_none());

    assert!(dsp_deemphasis_step(f64::NAN, 0.0, 0.95).is_none());
    assert!(dsp_deemphasis_step(0.0, f64::NEG_INFINITY, 0.95).is_none());

    // Test domain limits on samples
    assert!(dsp_preemphasis_step(1e6 + 1.0, 0.0, 0.95).is_none());
    assert!(dsp_preemphasis_step(-1e6 - 1.0, 0.0, 0.95).is_none());
    assert!(dsp_deemphasis_step(0.0, 1e6 + 1.0, 0.95).is_none());

    // Test valid boundary alpha values [0.0, 1.0]
    // @safety-proof: Alpha 0.0 is exact valid lower bound
    let pre_zero = dsp_preemphasis_step(5.0, 3.0, 0.0).unwrap();
    assert!((pre_zero - 5.0).abs() < 1e-12);

    // @safety-proof: Alpha 1.0 is exact valid upper bound
    let pre_one = dsp_preemphasis_step(5.0, 3.0, 1.0).unwrap();
    assert!((pre_one - 2.0).abs() < 1e-12);
}
