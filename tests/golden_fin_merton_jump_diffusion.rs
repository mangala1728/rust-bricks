// 本檔由 cloud_farmer 依 specs/fin_merton_jump_diffusion.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_merton_jump_diffusion::*;

#[test]
fn golden_vectors_from_spec() {
    let w0 = fin_merton_poisson_weight(0.0, 1.0, 0).unwrap();
    assert!((w0 - 1.0).abs() < 1e-12, "spec golden vector #1");
    let w1 = fin_merton_poisson_weight(0.0, 1.0, 1).unwrap();
    assert!((w1 - 0.0).abs() < 1e-12, "spec golden vector #2");
    let (k, lp) = fin_merton_jump_parameters(2.0, 0.0, 0.0).unwrap();
    assert!((k - 0.0).abs() < 1e-12, "spec golden vector #3");
    assert!((lp - 2.0).abs() < 1e-12, "spec golden vector #4");
    assert!(fin_merton_poisson_weight(-1.0, 1.0, 0).is_none(), "spec golden vector #5");
    assert!(fin_merton_poisson_weight(1.0, 1.0, 31).is_none(), "spec golden vector #6");
}
