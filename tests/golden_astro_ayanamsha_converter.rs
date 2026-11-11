// 本檔由 cloud_farmer 依 specs/astro_ayanamsha_converter.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_ayanamsha_converter::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(astro_tropical_to_sidereal(150_000, 0) == Some(150_000), "spec golden vector #1");
    assert!(astro_tropical_to_sidereal(30_000, 85_871) == Some(6147), "spec golden vector #2");
    assert!(astro_sidereal_to_tropical(6147, 85_871) == Some(30_000), "spec golden vector #3");
    assert!(astro_tropical_to_sidereal(360_001, 85_871).is_none(), "spec golden vector #4");
    assert!(astro_tropical_to_sidereal(30_000, 1_296_001).is_none(), "spec golden vector #5");
}
