// 本檔由 cloud_farmer 依 specs/dsp_pink_noise.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_pink_noise::*;

#[test]
fn golden_vectors_from_spec() {
    let mut s = [0.0, 0.0, 0.0];
    assert!((dsp_pink_filter_step(1.0, &mut s).unwrap() - 1.6330537).abs() < 1e-5, "spec golden vector #1");
    assert!((s[0] - 0.0990460).abs() < 1e-6, "spec golden vector #2");
    assert!((s[1] - 0.2965164).abs() < 1e-6, "spec golden vector #3");
    assert!((s[2] - 1.0526913).abs() < 1e-6, "spec golden vector #4");
    assert!(dsp_pink_filter_step(1.5, &mut s).is_none(), "spec golden vector #5");
    assert!(dsp_pink_filter_step(-1.5, &mut s).is_none(), "spec golden vector #6");
    assert!(dsp_pink_filter_step(f64::NAN, &mut s).is_none(), "spec golden vector #7");
}
