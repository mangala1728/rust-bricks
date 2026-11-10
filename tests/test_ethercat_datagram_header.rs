// ## Map
// Role: IEC 61158-4-12 EtherCAT 10-byte datagram header parser integration test
// Inputs: none (uses golden vectors)
// Outputs: test results
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::ethercat_datagram_header::*;

#[test]
fn parse_standard_header() {
    let raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
    let parsed = ethercat_parse_datagram_header(&raw).expect("valid header");
    assert_eq!(parsed.cmd, 0x01);
    assert_eq!(parsed.idx, 0x05);
    assert_eq!(parsed.address, 0x0001_0000);
    assert_eq!(parsed.data_len, 4);
    assert_eq!(parsed.more_follows, false);
    assert_eq!(parsed.irq, 0x0000);
}

#[test]
fn parse_header_with_more_flag() {
    let mut raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
    raw[7] = 0x80; // set the More‑Datagrams flag (bit 15)
    let parsed = ethercat_parse_datagram_header(&raw).expect("valid header with M flag");
    assert_eq!(parsed.more_follows, true);
    assert_eq!(parsed.data_len, 4);
}

#[test]
fn reject_invalid_command_zero() {
    let mut raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
    raw[0] = 0x00; // illegal command
    assert!(ethercat_parse_datagram_header(&raw).is_none());
}

#[test]
fn reject_invalid_command_too_large() {
    let mut raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
    raw[0] = 0x0F; // 15 > 14
    assert!(ethercat_parse_datagram_header(&raw).is_none());
}

#[test]
fn reject_too_short_slice() {
    let raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00]; // 9 bytes
    assert!(ethercat_parse_datagram_header(&raw).is_none());
}

#[test]
fn reject_excessive_data_length() {
    // data_len = 1499 (0x05DB) – exceeds allowed 1498.
    let len_flags = 0x05DBu16; // 0b0000_0101_1101_1011
    let raw = [
        0x01, // cmd
        0x05, // idx
        0x00, 0x00, 0x01, 0x00, // address = 0x0001_0000
        (len_flags & 0x00FF) as u8,
        (len_flags >> 8) as u8,
        0x00,
        0x00,
    ];
    assert!(ethercat_parse_datagram_header(&raw).is_none());
}
