// 本檔由 cloud_farmer 依 specs/canfd_crc.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::canfd_crc::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(canfd_crc17_slice(b"123456789") == Some(0x04F03), "spec golden vector #1");
    assert!(canfd_crc17_slice(&[0x80]) == Some(0x0587B), "spec golden vector #2");
    assert!(canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB), "spec golden vector #3");
    assert!(canfd_crc17_slice(&[0xFF]) == Some(0x19052), "spec golden vector #4");
    assert!(canfd_crc17_slice(&[]) == Some(0), "spec golden vector #5");
    assert!(canfd_crc17_bit(0, 1) == Some(0x1685B), "spec golden vector #6");
    assert!(canfd_crc17_bit(0x10000, 0) == Some(0x1685B), "spec golden vector #7");
    assert!(canfd_crc17_bit(0x10000, 1) == Some(0), "spec golden vector #8");
    assert!(canfd_crc17_bit(1, 0) == Some(2), "spec golden vector #9");
    assert!(canfd_crc17_slice(&[0u8; 16]) == Some(0), "spec golden vector #10");
    assert!(canfd_crc17_slice(&[0u8; 17]).is_none(), "spec golden vector #11");
    assert!(canfd_crc17_bit(0x20000, 0).is_none(), "spec golden vector #12");
    assert!(canfd_crc17_bit(0, 2).is_none(), "spec golden vector #13");
}
