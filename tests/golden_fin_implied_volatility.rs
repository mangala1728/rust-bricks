// 本檔由 cloud_farmer 依 specs/fin_implied_volatility.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_implied_volatility::*;

#[test]
fn golden_vectors_from_spec() {
    let p = fin_bs_call_price(100.0, 100.0, 0.05, 1.0, 0.20).unwrap();
    assert!((p - 10.45058).abs() < 1e-4, "spec golden vector #1");
    let iv = fin_implied_volatility_bisection(10.45058, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).unwrap();
    assert!((iv - 0.20).abs() < 1e-3, "spec golden vector #2");
    assert!(fin_implied_volatility_bisection(150.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none(), "spec golden vector #3");
    assert!(fin_implied_volatility_bisection(-1.0, 100.0, 100.0, 0.05, 1.0, 1e-5, 50).is_none(), "spec golden vector #4");
}
