// ## Map
// Role: CRC-17/CAN-FD（RevEng 參數）位元組資料校驗測試
// Inputs: canfd_crc17_bit, canfd_crc17_slice
// Outputs: test results
// Invariants: 0 external dependencies, pure Rust tests
use rust_bricks::canfd_crc::{canfd_crc17_bit, canfd_crc17_slice};

#[test]
fn golden_check_value() {
    // RevEng CRC‑Catalogue check for "123456789"
    assert_eq!(canfd_crc17_slice(b"123456789"), Some(0x04F03));
}

#[test]
fn golden_polynomial_modulo() {
    // Independent polynomial long‑division results
    assert_eq!(canfd_crc17_slice(&[0x80]), Some(0x0587B));
    assert_eq!(canfd_crc17_slice(&[0x12, 0x34]), Some(0x1ACDB));
    assert_eq!(canfd_crc17_slice(&[0xFF]), Some(0x19052));
    assert_eq!(canfd_crc17_slice(&[]), Some(0));
}

#[test]
fn golden_shift_register_steps() {
    // ISO 11898‑1 shift‑register definition examples
    assert_eq!(canfd_crc17_bit(0, 1), Some(0x1685B));
    assert_eq!(canfd_crc17_bit(0x10000, 0), Some(0x1685B));
    assert_eq!(canfd_crc17_bit(0x10000, 1), Some(0));
    assert_eq!(canfd_crc17_bit(1, 0), Some(2));
}

#[test]
fn golden_boundary_conditions() {
    // Length / value‑range guards
    let zeros_16 = [0u8; 16];
    assert_eq!(canfd_crc17_slice(&zeros_16), Some(0));

    let zeros_17 = [0u8; 17];
    assert!(canfd_crc17_slice(&zeros_17).is_none());

    assert!(canfd_crc17_bit(0x20000, 0).is_none());
    assert!(canfd_crc17_bit(0, 2).is_none());
}
