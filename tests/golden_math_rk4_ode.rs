// 本檔由 cloud_farmer 依 specs/math_rk4_ode.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_rk4_ode::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((rk4_step_1d(0.0, 1.0, 0.1, |_t: f64, y: f64| -y).unwrap() - 0.9048375).abs() < 1e-6, "spec golden vector #1");
    assert!(rk4_step_1d(0.0, 1.0, 0.0, |_t: f64, y: f64| -y).is_none(), "spec golden vector #2");
    assert!(rk4_step_1d(0.0, 1.0, 1001.0, |_t: f64, y: f64| -y).is_none(), "spec golden vector #3");
    assert!(rk4_step_2d(0.0, (1.0, 0.0), 0.0, |_t: f64, y: (f64, f64)| (y.1, -y.0)).is_none(), "spec golden vector #4");
}
