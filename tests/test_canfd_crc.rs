// ## Map
// Role: integration test for CAN-FD 17‑bit CRC implementation
// Inputs: none (uses public API)
// Outputs: test results via `assert_eq!`
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::canfd_crc::*;

#[test]
fn golden_canfd_crc_bit() {
    // valid cases
    assert_eq!(canfd_crc17_bit(0x1_0000, 0), Some(0x1685B));
    assert_eq!(canfd_crc17_bit(0x0_0001, 1), Some(3));

    // out‑of‑range inputs → None
    assert!(canfd_crc17_bit(0x2_0000, 0).is_none());
    assert!(canfd_crc17_bit(0, 2).is_none());
}

#[test]
fn golden_canfd_crc_slice() {
    // empty payload
    assert_eq!(canfd_crc17_slice(&[]), Some(0));

    // single‑byte payloads
    assert_eq!(canfd_crc17_slice(&[0x00]), Some(0));
    assert_eq!(canfd_crc17_slice(&[0x80]), Some(128));

    // length guard
    let long: Vec<u8> = vec![0; 65];
    assert!(canfd_crc17_slice(&long).is_none());
}
