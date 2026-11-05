// ## Map
// Role: integration test for NMEA 2000 fast packet protocol
// Inputs: none (uses public API)
// Outputs: test results via assertions
// Invariants: 0 external dependencies, functions <= 60 lines

use rust_bricks::nmea2000_fast_packet::*;

#[test]
fn test_nmea2000_parse_header_golden() {
    assert_eq!(nmea2000_parse_header(0x40), (2, 0));
    assert_eq!(nmea2000_parse_header(0xA3), (5, 3));
}

#[test]
fn test_nmea2000_extract_first_frame_golden() {
    let mut buf = [0u8; 223];
    let f = [0x40, 10, b'A', b'B', b'C', b'D', b'E', b'F'];
    assert_eq!(nmea2000_extract_first_frame(&f, &mut buf), Some((2, 10)));
    assert_eq!(&buf[..6], b"ABCDEF");
}

#[test]
fn test_nmea2000_extract_subsequent_frame_golden() {
    let mut buf = [0u8; 223];
    let f1 = [0x40, 10, b'A', b'B', b'C', b'D', b'E', b'F'];
    assert_eq!(nmea2000_extract_first_frame(&f1, &mut buf), Some((2, 10)));

    let f2 = [0x41, b'G', b'H', b'I', b'J', 0xFF, 0xFF, 0xFF];
    let progress = nmea2000_extract_subsequent_frame(&f2, 2, 1, &mut buf, 10);
    assert_eq!(progress, Some(10));
    assert_eq!(&buf[..10], b"ABCDEFGHIJ");
}

#[test]
fn test_nmea2000_guards() {
    let mut buf = [0u8; 223];
    let f_not_first = [0x41, 10, 0, 0, 0, 0, 0, 0];
    assert!(nmea2000_extract_first_frame(&f_not_first, &mut buf).is_none());

    let f_overflow = [0x40, 250, 0, 0, 0, 0, 0, 0];
    assert!(nmea2000_extract_first_frame(&f_overflow, &mut buf).is_none());

    let f_zero = [0x40, 0, 0, 0, 0, 0, 0, 0];
    assert!(nmea2000_extract_first_frame(&f_zero, &mut buf).is_none());
}
