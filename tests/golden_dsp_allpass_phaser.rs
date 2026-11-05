// 本檔由 cloud_farmer 依 specs/dsp_allpass_phaser.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_allpass_phaser::*;

#[test]
fn golden_vectors_from_spec() {
    let mut st = 0.0;
    assert!((dsp_allpass_first_order_step(1.0, &mut st, 0.5).unwrap() - (-0.5)).abs() < 1e-10, "spec golden vector #1");
    assert!((st - 1.0).abs() < 1e-10, "spec golden vector #2");
    assert!((dsp_allpass_first_order_step(0.0, &mut st, 0.5).unwrap() - 0.75).abs() < 1e-10, "spec golden vector #3");
    assert!((dsp_allpass_first_order_step(0.0, &mut st, 0.5).unwrap() - 0.375).abs() < 1e-10, "spec golden vector #4");
    assert!(dsp_allpass_first_order_step(1.0, &mut st, 1.0).is_none(), "spec golden vector #5");
    assert!(dsp_allpass_first_order_step(1.0, &mut st, -1.0).is_none(), "spec golden vector #6");
    assert!(dsp_allpass_first_order_step(f64::NAN, &mut st, 0.5).is_none(), "spec golden vector #7");
}
