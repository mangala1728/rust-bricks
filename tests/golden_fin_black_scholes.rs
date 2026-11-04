// 本檔由 cloud_farmer 依 specs/fin_black_scholes.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_black_scholes::*;

#[test]
fn golden_vectors_from_spec() {
    assert!((standard_normal_cdf(0.35).unwrap() - 0.63683).abs() < 1e-3, "spec golden vector #1");
    assert!((standard_normal_cdf(0.15).unwrap() - 0.55962).abs() < 1e-3, "spec golden vector #2");
    assert!((black_scholes_european(100.0, 100.0, 0.05, 0.20, 1.0).unwrap().0 - 10.4506).abs() < 1e-3, "spec golden vector #3");
    assert!((black_scholes_european(100.0, 100.0, 0.05, 0.20, 1.0).unwrap().1 - 5.5735).abs() < 1e-3, "spec golden vector #4");
    assert!((black_scholes_european(100.0, 100.0, 0.05, 0.20, 1.0).unwrap().2 - 0.6368).abs() < 1e-3, "spec golden vector #5");
    assert!((black_scholes_european(100.0, 100.0, 0.05, 0.20, 1.0).unwrap().3 - (-0.3632)).abs() < 1e-3, "spec golden vector #6");
}
