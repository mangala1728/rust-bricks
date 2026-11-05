// 本檔由 cloud_farmer 依 specs/canfd_crc.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::canfd_crc::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(canfd_crc17_bit(0x10000, 0) == Some(0x1685B), "spec golden vector #1");
    assert!(canfd_crc17_bit(0x00001, 1) == Some(3), "spec golden vector #2");
    assert!(canfd_crc17_bit(0x20000, 0).is_none(), "spec golden vector #3");
    assert!(canfd_crc17_bit(0, 2).is_none(), "spec golden vector #4");
    assert!(canfd_crc17_slice(&[]) == Some(0), "spec golden vector #5");
    assert!(canfd_crc17_slice(&[0x00]) == Some(0), "spec golden vector #6");
    assert!(canfd_crc17_slice(&[0x80]) == Some(128), "spec golden vector #7");
}
