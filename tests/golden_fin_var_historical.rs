// 本檔由 cloud_farmer 依 specs/fin_var_historical.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::fin_var_historical::*;

#[test]
fn golden_vectors_from_spec() {
    let r = [-0.05, -0.04, -0.03, -0.02, -0.01, 0.01, 0.02, 0.03, 0.04, 0.05];
    assert!((fin_var_historical(&r, 0.90).unwrap() - 0.04).abs() < 1e-4, "spec golden vector #1");
    assert!(fin_var_historical(&[-0.01], 0.90).is_none(), "spec golden vector #2");
    assert!(fin_var_historical(&r, 0.0).is_none(), "spec golden vector #3");
    assert!(fin_var_historical(&r, 1.0).is_none(), "spec golden vector #4");
    assert!(fin_var_historical(&r, f64::NAN).is_none(), "spec golden vector #5");
    assert!(fin_var_historical(&[-0.01, f64::NAN], 0.90).is_none(), "spec golden vector #6");
    assert!(fin_cvar_historical(&[-0.01], 0.90).is_none(), "spec golden vector #7");
    assert!(fin_cvar_historical(&r, 0.0).is_none(), "spec golden vector #8");
    assert!(fin_cvar_historical(&r, 1.0).is_none(), "spec golden vector #9");
    assert!(fin_cvar_historical(&[-0.01, f64::INFINITY], 0.90).is_none(), "spec golden vector #10");
    assert!((fin_cvar_historical(&r, 0.90).unwrap() - 0.05).abs() < 1e-4, "spec golden vector #11");
    assert!((fin_cvar_historical(&r, 0.80).unwrap() - 0.045).abs() < 1e-4, "spec golden vector #12");
    assert!((fin_cvar_historical(&r, 0.70).unwrap() - 0.04).abs() < 1e-4, "spec golden vector #13");
    let r20 = [-0.10, -0.09, -0.08, -0.07, -0.06, -0.05, -0.04, -0.03, -0.02, -0.01, 0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10];
    assert!((fin_cvar_historical(&r20, 0.95).unwrap() - 0.10).abs() < 1e-4, "spec golden vector #14");
}
