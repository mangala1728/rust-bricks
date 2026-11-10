// 本檔由 cloud_farmer 依 specs/control_sliding_mode_1d.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::control_sliding_mode_1d::*;

#[test]
fn golden_vectors_from_spec() {
    let (s1, u1) = control_sliding_mode_step(1.0, -0.5, 1.0, 10.0, 1.0).unwrap();
    assert!((s1 - 0.5).abs() < 1e-12, "spec golden vector #1");
    assert!((u1 - 5.0).abs() < 1e-12, "spec golden vector #2");
    let (s2, u2) = control_sliding_mode_step(2.0, 1.0, 1.0, 10.0, 1.0).unwrap();
    assert!((s2 - 3.0).abs() < 1e-12, "spec golden vector #3");
    assert!((u2 - 10.0).abs() < 1e-12, "spec golden vector #4");
    assert!(control_sliding_mode_step(1.0, 0.0, 0.0, 10.0, 1.0).is_none(), "spec golden vector #5");
    assert!(control_sliding_mode_step(1.0, 0.0, 1.0, 10.0, 0.0).is_none(), "spec golden vector #6");
}
