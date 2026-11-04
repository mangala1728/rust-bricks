// 本檔由 cloud_farmer 依 specs/math_simpson_integral.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_simpson_integral::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((simpson_integral_1d(0.0, std::f64::consts::PI, 100, |x: f64| x.sin()).unwrap() - 2.0).abs() < 1e-4, "spec golden vector #1");
    assert!((simpson_integral_1d(0.0, 3.0, 100, |x: f64| x * x).unwrap() - 9.0).abs() < 1e-6, "spec golden vector #2");
    assert!((simpson_integral_1d(0.0, 3.0, 3, |x: f64| x * x).unwrap() - 9.0).abs() < 1e-6, "spec golden vector #3");
    assert!(simpson_integral_1d(1.0, 1.0, 10, |x: f64| x).is_none(), "spec golden vector #4");
    assert!(simpson_integral_1d(2.0, 1.0, 10, |x: f64| x).is_none(), "spec golden vector #5");
    assert!(simpson_integral_1d(0.0, 1.0, 1, |x: f64| x).is_none(), "spec golden vector #6");
    assert!(simpson_integral_1d(0.0, 1.0, 100001, |x: f64| x).is_none(), "spec golden vector #7");
    assert!(simpson_integral_1d(f64::NAN, 1.0, 10, |x: f64| x).is_none(), "spec golden vector #8");
    assert!(simpson_integral_1d(0.0, f64::INFINITY, 10, |x: f64| x).is_none(), "spec golden vector #9");
}
