// 本檔由 cloud_farmer 依 specs/astro_vimshottari_dasha.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::astro_vimshottari_dasha::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(astro_vimshottari_dasha_pack(3, 7, 1, 4, 5, 2500) == Some(2621784435), "spec golden vector #1");
    assert!(astro_vimshottari_dasha_unpack(2621784435) == (3, 7, 1, 4, 5, 2500), "spec golden vector #2");
    assert!(astro_vimshottari_dasha_pack(0, 1, 1, 1, 1, 0).is_none(), "spec golden vector #3");
    assert!(astro_vimshottari_dasha_pack(10, 1, 1, 1, 1, 0).is_none(), "spec golden vector #4");
    assert!(astro_vimshottari_dasha_pack(1, 1, 1, 1, 1, 4096).is_none(), "spec golden vector #5");
}
