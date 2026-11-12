// 本檔由 cloud_farmer 依 specs/astro_tithi_special_detector.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_tithi_special_detector::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(astro_tithi_special_detector(1200, 1800, 1000, 2000) == Some(1), "spec golden vector #1");
    assert!(astro_tithi_special_detector(900, 2100, 1000, 2000) == Some(2), "spec golden vector #2");
    assert!(astro_tithi_special_detector(800, 1500, 1000, 2000) == Some(0), "spec golden vector #3");
    assert!(astro_tithi_special_detector(2000, 1000, 500, 1500).is_none(), "spec golden vector #4");
    assert!(astro_tithi_special_detector(1000, 2000, 1500, 1000).is_none(), "spec golden vector #5");
}
