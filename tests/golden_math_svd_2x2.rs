// 本檔由 cloud_farmer 依 specs/math_svd_2x2.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_svd_2x2::*;

#[test]
fn golden_vectors_from_spec() {
    let (s1, s2) = math_svd_2x2_singular_values(1.0, 0.0, 0.0, 1.0).unwrap();
    assert!((s1 - 1.0).abs() < 1e-10 && (s2 - 1.0).abs() < 1e-10, "spec golden vector #1");
    assert!((math_matrix_2x2_condition_number(1.0, 0.0, 0.0, 1.0).unwrap() - 1.0).abs() < 1e-10, "spec golden vector #2");
    let (s1, s2) = math_svd_2x2_singular_values(3.0, 0.0, 0.0, 2.0).unwrap();
    assert!((s1 - 3.0).abs() < 1e-10 && (s2 - 2.0).abs() < 1e-10, "spec golden vector #3");
    assert!((math_matrix_2x2_condition_number(3.0, 0.0, 0.0, 2.0).unwrap() - 1.5).abs() < 1e-10, "spec golden vector #4");
    let (s1, s2) = math_svd_2x2_singular_values(1.0, 2.0, 2.0, 4.0).unwrap();
    assert!((s1 - 5.0).abs() < 1e-10 && s2.abs() < 1e-10, "spec golden vector #5");
    assert!(math_matrix_2x2_condition_number(1.0, 2.0, 2.0, 4.0).is_none(), "spec golden vector #6");
    assert!(math_svd_2x2_singular_values(f64::NAN, 0.0, 0.0, 1.0).is_none(), "spec golden vector #7");
}
