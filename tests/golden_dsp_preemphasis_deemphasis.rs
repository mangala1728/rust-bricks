// 本檔由 cloud_farmer 依 specs/dsp_preemphasis_deemphasis.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_preemphasis_deemphasis::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((dsp_preemphasis_step(10.0, 8.0, 0.95).unwrap() - 2.4).abs() < 1e-12, "spec golden vector #1");
    assert!((dsp_deemphasis_step(2.4, 8.0, 0.95).unwrap() - 10.0).abs() < 1e-12, "spec golden vector #2");
    assert!(dsp_preemphasis_step(10.0, 8.0, 1.1).is_none(), "spec golden vector #3");
    assert!(dsp_deemphasis_step(10.0, 8.0, -0.1).is_none(), "spec golden vector #4");
}
