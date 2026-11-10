// 本檔由 cloud_farmer 依 specs/fin_macd_oscillator.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_macd_oscillator::*;

#[test]
fn golden_vectors_from_spec() {
    let res = fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).unwrap();
    assert!((res.0 - 11.0).abs() < 1e-12, "spec golden vector #1");
    assert!((res.1 - 8.0).abs() < 1e-12, "spec golden vector #2");
    assert!((res.2 - 3.0).abs() < 1e-12, "spec golden vector #3");
    assert!((res.3 - 95.0).abs() < 1e-12, "spec golden vector #4");
    assert!((res.4 - 84.0).abs() < 1e-12, "spec golden vector #5");
    assert!(fin_macd_step(100.0, 90.0, 80.0, 5.0, 0.2, 0.5, 0.5).is_none(), "spec golden vector #6");
    assert!(fin_macd_step(-10.0, 90.0, 80.0, 5.0, 0.5, 0.2, 0.5).is_none(), "spec golden vector #7");
}
