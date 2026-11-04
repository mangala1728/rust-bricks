// 本檔由 cloud_farmer 依 specs/canopen_sdo_handler.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::canopen_sdo_handler::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(parse_sdo_download_request(&[0x2B, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 2)), "spec golden vector #1");
    assert!(parse_sdo_download_request(&[0x22, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 4)), "spec golden vector #2");
    assert!(parse_sdo_download_request(&[0x20, 0x17, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 0)), "spec golden vector #3");
    assert!(parse_sdo_download_request(&[0x40, 0x17, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]).is_none(), "spec golden vector #4");
    let mut frame = [0u8; 8];
    assert!(encode_sdo_abort(0x1017, 0x00, 0x0602_0000, &mut frame), "spec golden vector #5");
    assert!(frame == [0x80, 0x17, 0x10, 0x00, 0x00, 0x00, 0x02, 0x06], "spec golden vector #6");
}
