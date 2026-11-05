// 本檔由 cloud_farmer 依 specs/control_pid_antiwindup.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::control_pid_antiwindup::*;

#[test]
fn golden_vectors_from_spec() {
    let (out1, next_i1) = control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -100.0, 100.0).unwrap();
    assert!((out1 - 4.2).abs() < 1e-12, "spec golden vector #1");
    assert!((next_i1 - 0.2).abs() < 1e-12, "spec golden vector #2");
    let (out2, next_i2) = control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -10.0, 3.0).unwrap();
    assert!((out2 - 3.0).abs() < 1e-12, "spec golden vector #3");
    assert!((next_i2 - 0.0).abs() < 1e-12, "spec golden vector #4");
    assert!(control_pid_step(10.0, 8.0, 0.0, 2.0, 0.0, 2.0, 1.0, 0.5, -10.0, 10.0).is_none(), "spec golden vector #5");
    assert!(control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, 10.0, -10.0).is_none(), "spec golden vector #6");
}
