// 本檔由 cloud_farmer 依 specs/astro_panchanga_nakshatra.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_panchanga_nakshatra::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(astro_panchanga_nakshatra(0) == Some((1, 1, 0)), "spec golden vector #1");
    assert!(astro_panchanga_nakshatra(142_500) == Some((11, 3, 6875)), "spec golden vector #2");
    assert!(astro_panchanga_nakshatra(360_001).is_none(), "spec golden vector #3");
}
