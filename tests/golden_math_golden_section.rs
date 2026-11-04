// 本檔由 cloud_farmer 依 specs/math_golden_section.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_golden_section::*;

#[test]
fn golden_vectors_from_spec() {
    let (x_opt, y_opt) = golden_section_search_min(0.0, 5.0, 1.0e-5, 100, |x: f64| (x - 2.0).powi(2) + 3.0).unwrap();
    assert!((x_opt - 2.0).abs() < 1e-4, "spec golden vector #1");
    assert!((y_opt - 3.0).abs() < 1e-4, "spec golden vector #2");
    assert!(golden_section_search_min(5.0, 0.0, 1.0e-5, 100, |x: f64| x * x).is_none(), "spec golden vector #3");
    assert!(golden_section_search_min(1.0, 1.0, 1.0e-5, 100, |x: f64| x * x).is_none(), "spec golden vector #4");
    assert!(golden_section_search_min(0.0, 5.0, 0.0, 100, |x: f64| x * x).is_none(), "spec golden vector #5");
    assert!(golden_section_search_min(0.0, 5.0, 1.0e-5, 0, |x: f64| x * x).is_none(), "spec golden vector #6");
}
