// ## Map
// Role: Integration test for Modbus RTU Frame & CRC-16
// Inputs: none (uses public API)
// Outputs: test assertions
// Invariants: golden vectors verified against Modbus Protocol Specification

use rust_bricks::modbus_rtu_frame::*;

#[test]
fn test_modbus_crc16_golden_vector() {
    let request = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01];
    let crc = modbus_crc16(&request);
    assert_eq!(crc, 0x0A84, "Modbus CRC-16 mismatch against golden standard");
}

#[test]
fn test_modbus_validate_frame_valid() {
    let frame = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x84, 0x0A];
    let res = modbus_validate_frame(&frame);
    assert_eq!(res, Some((1, 3, 4)));
}

#[test]
fn test_modbus_validate_frame_corrupt_and_short() {
    let corrupt = [0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0xFF, 0xFF];
    assert!(modbus_validate_frame(&corrupt).is_none());

    let too_short = [0x01, 0x03, 0x00];
    assert!(modbus_validate_frame(&too_short).is_none());
}
