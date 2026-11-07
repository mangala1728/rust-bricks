use rust_bricks::dsp_iir_comb_filter::*;

#[test]
fn test_golden_dsp_iir_comb_feedback_step() {
    let res = dsp_iir_comb_feedback_step(1.0, 0.5, 0.8);
    // @safety-proof: Inputs are valid constants defined in spec
    let val = res.unwrap();
    assert!((val - 1.4).abs() < 1e-12);
}

#[test]
fn test_golden_dsp_comb_feedforward_step() {
    let res = dsp_comb_feedforward_step(0.5, 0.5, -1.0);
    // @safety-proof: Inputs are valid constants defined in spec
    let val = res.unwrap();
    assert!((val - 0.0).abs() < 1e-12);
}

#[test]
fn test_golden_rejection_cases() {
    assert!(dsp_iir_comb_feedback_step(1.0, 0.5, 1.0).is_none());
    assert!(dsp_iir_comb_feedback_step(1.0, 0.5, -1.0).is_none());
    assert!(dsp_iir_comb_feedback_step(f64::NAN, 0.5, 0.5).is_none());
    assert!(dsp_comb_feedforward_step(1.0, 0.5, 1.05).is_none());
}

#[test]
fn test_extended_out_of_domain() {
    assert!(dsp_iir_comb_feedback_step(1e7, 0.5, 0.5).is_none());
    assert!(dsp_comb_feedforward_step(1.0, -1e7, 0.5).is_none());
    assert!(dsp_comb_feedforward_step(1.0, 0.5, -1.05).is_none());
}
