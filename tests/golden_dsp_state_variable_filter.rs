// 本檔由 cloud_farmer 依 specs/dsp_state_variable_filter.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_state_variable_filter::*;

#[test]
fn golden_vectors_from_spec() {
    let res = dsp_svf_step(1.0, 0.0, 0.0, 0.5, 1.0).unwrap();
    assert!((res.0 - 0.25).abs() < 1e-12, "spec golden vector #1");
    assert!((res.1 - 0.50).abs() < 1e-12, "spec golden vector #2");
    assert!((res.2 - 1.00).abs() < 1e-12, "spec golden vector #3");
    assert!((res.3 - 1.25).abs() < 1e-12, "spec golden vector #4");
    assert!((res.4 - 0.25).abs() < 1e-12, "spec golden vector #5");
    assert!((res.5 - 0.50).abs() < 1e-12, "spec golden vector #6");
    assert!(dsp_svf_step(1.0, 0.0, 0.0, 0.9, 1.0).is_none(), "spec golden vector #7");
    assert!(dsp_svf_step(1.0, 0.0, 0.0, 0.5, 0.0).is_none(), "spec golden vector #8");
}
