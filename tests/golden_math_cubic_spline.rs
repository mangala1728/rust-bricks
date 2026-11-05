// 本檔由 cloud_farmer 依 specs/math_cubic_spline.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_cubic_spline::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((math_cubic_spline_eval(1.0, 0.0, 2.0, 10.0, 20.0, 0.0, 0.0).unwrap() - 15.0).abs() < 1e-10, "spec golden vector #1");
    assert!((math_cubic_spline_eval(1.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 1.0).abs() < 1e-10, "spec golden vector #2");
    assert!((math_cubic_spline_eval(0.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 0.0).abs() < 1e-10, "spec golden vector #3");
    assert!((math_cubic_spline_eval(2.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 4.0).abs() < 1e-10, "spec golden vector #4");
    assert!(math_cubic_spline_eval(2.5, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).is_none(), "spec golden vector #5");
    assert!(math_cubic_spline_eval(1.0, 2.0, 0.0, 0.0, 4.0, 2.0, 2.0).is_none(), "spec golden vector #6");
}
