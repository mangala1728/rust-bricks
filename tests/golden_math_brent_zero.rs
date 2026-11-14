// 本檔由 cloud_farmer 依 specs/math_brent_zero.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::math_brent_zero::*;

#[test]
fn golden_vectors_from_spec() {
    let s = math_brent_candidate_step(0.0, 2.0, 0.0, -2.0, 2.0, -2.0).unwrap();
    assert!((s - 1.0).abs() < 1e-12, "spec golden vector #1");
    assert!(math_brent_candidate_step(0.0, 2.0, 1.0, 1.0, 1.0, 2.0).is_none(), "spec golden vector #2");
}
