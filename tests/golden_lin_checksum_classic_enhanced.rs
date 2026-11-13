// 本檔由 cloud_farmer 依 specs/lin_checksum_classic_enhanced.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::lin_checksum_classic_enhanced::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(lin_compute_checksum(0x00, &[0x01, 0x02, 0x03], false) == Some(0xF9), "spec golden vector #1");
    assert!(lin_compute_checksum(0x22, &[0x4A, 0x55, 0x93, 0xE5], true) == Some(0xC4), "spec golden vector #2");
    assert!(lin_compute_checksum(0x3C, &[0x01, 0x02], true) == Some(0xFC), "spec golden vector #3");
    assert!(lin_compute_checksum(0x00, &[], false).is_none(), "spec golden vector #4");
    assert!(lin_compute_checksum(0x00, &[1, 2, 3, 4, 5, 6, 7, 8, 9], false).is_none(), "spec golden vector #5");
}
