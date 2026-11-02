// Integration test for the `canopen_sdo_handler` module.
// Uses deterministic golden vectors from the CiA 301 standard.

use rust_bricks::canopen_sdo_handler::*;

/// Test parsing of a known Initiate Download Request (expedited, size indicated).
#[test]
fn test_parse_sdo_download_request() {
    // Golden payload: 0x2B 0x17 0x10 0x00 0x42 0x00 0x00 0x00
    let payload: [u8; 8] = [0x2B, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00];
    let result = parse_sdo_download_request(&payload);
    assert_eq!(result, Some((1, 0x1017, 0x00, 2)));
}

/// Test encoding of an SDO Abort Transfer response.
#[test]
fn test_encode_sdo_abort() {
    let mut frame = [0u8; 8];
    let index = 0x2000u16;
    let sub_index = 0x01u8;
    let abort_code = 0x06020000u32; // Example abort code: Object does not exist.

    let ok = encode_sdo_abort(index, sub_index, abort_code, &mut frame);
    assert!(ok, "encode_sdo_abort should return true");

    // Expected frame layout.
    let expected: [u8; 8] = [
        0x80,               // CS = Abort
        0x00,               // index low byte
        0x20,               // index high byte
        0x01,               // sub‑index
        0x00, 0x00, 0x02, 0x06, // abort code (little‑endian)
    ];
    assert_eq!(frame, expected);
}

#[test]
fn test_parse_sdo_invalid_ccs() {
    // If ccs != 1 (e.g. 0x40 -> ccs = 2, Initiate Upload Request)
    let payload: [u8; 8] = [0x40, 0x17, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00];
    assert!(parse_sdo_download_request(&payload).is_none());
}
