// 本檔由 cloud_farmer 依 specs/math_secant_method.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_secant_method::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((math_secant_step(1.0, 3.0, -3.0, 5.0).unwrap() - 1.75).abs() < 1e-10, "spec golden vector #1");
    assert!(math_secant_step(1.0, 3.0, 5.0, 5.0).is_none(), "spec golden vector #2");
    let (root, iters) = math_secant_solve(1.0, 3.0, 1e-6, 50, |x| x * x - 4.0).unwrap();
    assert!((root - 2.0).abs() < 1e-5, "spec golden vector #3");
    assert!(iters > 0 && iters <= 20, "spec golden vector #4");
    assert!(math_secant_solve(1.0, 1.0, 1e-6, 50, |x| x * x - 4.0).is_none(), "spec golden vector #5");
    assert!(math_secant_solve(1.0, 3.0, -1e-6, 50, |x| x * x - 4.0).is_none(), "spec golden vector #6");
}
