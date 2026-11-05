// 本檔由 cloud_farmer 依 specs/subtitle_cps_vad.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::subtitle_cps_vad::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(subtitle_cps_tenths(9, 1000) == Some(90), "spec golden vector #1");
    assert!(subtitle_cps_tenths(18, 2500) == Some(72), "spec golden vector #2");
    assert!(subtitle_cps_band(90, 0) == Some(0), "spec golden vector #3");
    assert!(subtitle_cps_band(91, 0) == Some(1), "spec golden vector #4");
    assert!(subtitle_cps_band(120, 0) == Some(1), "spec golden vector #5");
    assert!(subtitle_cps_band(121, 0) == Some(2), "spec golden vector #6");
    assert!(subtitle_cps_band(70, 1) == Some(0), "spec golden vector #7");
    assert!(subtitle_cps_band(72, 1) == Some(1), "spec golden vector #8");
    assert!(subtitle_vad_drift_ok(1040, 3480, 1000, 3500, 100) == Some((1, 40)), "spec golden vector #9");
    assert!(subtitle_vad_drift_ok(1350, 3850, 1000, 3500, 100) == Some((0, 350)), "spec golden vector #10");
    assert!(subtitle_cps_tenths(10, 0).is_none(), "spec golden vector #11");
    assert!(subtitle_cps_tenths(5001, 1000).is_none(), "spec golden vector #12");
    assert!(subtitle_cps_band(100, 2).is_none(), "spec golden vector #13");
    assert!(subtitle_vad_drift_ok(3500, 1000, 1000, 3500, 100).is_none(), "spec golden vector #14");
    assert!(subtitle_vad_drift_ok(1000, 3500, 1000, 3500, 0).is_none(), "spec golden vector #15");
}
