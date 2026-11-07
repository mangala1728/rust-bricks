// 本檔由 cloud_farmer 依 specs/dsp_iir_comb_filter.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_iir_comb_filter::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((dsp_iir_comb_feedback_step(1.0, 0.5, 0.8).unwrap() - 1.4).abs() < 1e-12, "spec golden vector #1");
    assert!((dsp_comb_feedforward_step(0.5, 0.5, -1.0).unwrap() - 0.0).abs() < 1e-12, "spec golden vector #2");
    assert!(dsp_iir_comb_feedback_step(1.0, 0.5, 1.0).is_none(), "spec golden vector #3");
    assert!(dsp_iir_comb_feedback_step(1.0, 0.5, -1.0).is_none(), "spec golden vector #4");
    assert!(dsp_iir_comb_feedback_step(f64::NAN, 0.5, 0.5).is_none(), "spec golden vector #5");
    assert!(dsp_comb_feedforward_step(1.0, 0.5, 1.05).is_none(), "spec golden vector #6");
}
