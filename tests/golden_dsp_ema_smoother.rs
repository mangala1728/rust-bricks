// 本檔由 cloud_farmer 依 specs/dsp_ema_smoother.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_ema_smoother::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((ema_step_f64(100.0, 0.0, 0.25).unwrap() - 25.0).abs() < 1e-6, "spec golden vector #1");
    assert!((ema_step_f64(42.0, 7.0, 1.0).unwrap() - 42.0).abs() < 1e-12, "spec golden vector #2");
    assert!((ema_step_f64(5.0, 5.0, 0.3).unwrap() - 5.0).abs() < 1e-12, "spec golden vector #3");
    assert!(ema_step_f64(1.0, 0.0, 0.0).is_none(), "spec golden vector #4");
    assert!(ema_step_f64(1.0, 0.0, 1.5).is_none(), "spec golden vector #5");
    assert!(ema_step_f64(f64::NAN, 0.0, 0.5).is_none(), "spec golden vector #6");
}
