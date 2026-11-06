// 本檔由 cloud_farmer 依 specs/dsp_cross_correlation_lag.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::dsp_cross_correlation_lag::*;

#[test]
fn golden_vectors_from_spec() {
    let x = [0.0, 1.0, 2.0, 3.0, 0.0];
    let y = [0.0, 0.0, 1.0, 2.0, 3.0];
    let (lag, corr) = dsp_find_peak_cross_correlation_lag(&x, &y, 2).unwrap();
    assert!(lag == 1, "spec golden vector #1");
    assert!(corr > 10.0, "spec golden vector #2");
    assert!(dsp_find_peak_cross_correlation_lag(&x, &y, 0).is_none(), "spec golden vector #3");
    assert!(dsp_find_peak_cross_correlation_lag(&[1.0, 2.0], &y, 1).is_none(), "spec golden vector #4");
}
