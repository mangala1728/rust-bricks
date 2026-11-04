// 本檔由 cloud_farmer 依 specs/math_newton_raphson_1d.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_newton_raphson_1d::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((newton_raphson_1d(1.0, 1.0e-6, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).unwrap() - 1.41421356).abs() < 1e-4, "spec golden vector #1");
    assert!((newton_raphson_1d(0.5, 1.0e-6, 20, |x: f64| x.cos() - x, |x: f64| -x.sin() - 1.0).unwrap() - 0.739085).abs() < 1e-4, "spec golden vector #2");
    assert!(newton_raphson_1d(f64::NAN, 1.0e-6, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #3");
    assert!(newton_raphson_1d(1.0, 1.0e-13, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #4");
    assert!(newton_raphson_1d(1.0, 2.0, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #5");
    assert!(newton_raphson_1d(1.0, 1.0e-6, 0, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #6");
    assert!(newton_raphson_1d(1.0, 1.0e-6, 1001, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #7");
    assert!(newton_raphson_1d(0.0, 1.0e-6, 20, |x: f64| x * x + 1.0, |x: f64| 2.0 * x).is_none(), "spec golden vector #8");
}
