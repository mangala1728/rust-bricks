// 本檔由 cloud_farmer 依 specs/modbus_rtu_frame.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::modbus_rtu_frame::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(modbus_crc16(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x01]) == 0x0A84, "spec golden vector #1");
    assert!(modbus_validate_frame(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x84, 0x0A]) == Some((1, 3, 4)), "spec golden vector #2");
    assert!(modbus_validate_frame(&[0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x0A, 0x84]).is_none(), "spec golden vector #3");
    assert!(modbus_validate_frame(&[0x01, 0x03, 0x84]).is_none(), "spec golden vector #4");
}
