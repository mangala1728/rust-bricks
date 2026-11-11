// 本檔由 cloud_farmer 依 specs/astro_panchanga_tithi.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_panchanga_tithi::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(astro_panchanga_tithi(0, 0) == Some((1, 0)), "spec golden vector #1");
    assert!(astro_panchanga_tithi(180_000, 0) == Some((16, 0)), "spec golden vector #2");
    assert!(astro_panchanga_tithi(142_500, 30_000) == Some((10, 3750)), "spec golden vector #3");
    assert!(astro_panchanga_tithi(360_001, 10_000).is_none(), "spec golden vector #4");
    assert!(astro_panchanga_tithi(10_000, 360_001).is_none(), "spec golden vector #5");
}
