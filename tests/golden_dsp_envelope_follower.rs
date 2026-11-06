// 本檔由 cloud_farmer 依 specs/dsp_envelope_follower.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_envelope_follower::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((dsp_envelope_follower_step(-2.0, 0.0, 0.8, 0.2).unwrap() - 0.4).abs() < 1e-12, "spec golden vector #1");
    assert!((dsp_envelope_follower_step(0.0, 1.0, 0.8, 0.9).unwrap() - 0.9).abs() < 1e-12, "spec golden vector #2");
    assert!(dsp_envelope_follower_step(1.0, -0.1, 0.8, 0.2).is_none(), "spec golden vector #3");
    assert!(dsp_envelope_follower_step(1.0, 0.5, 1.2, 0.2).is_none(), "spec golden vector #4");
    assert!(dsp_envelope_follower_step(f64::NAN, 0.5, 0.8, 0.2).is_none(), "spec golden vector #5");
}
