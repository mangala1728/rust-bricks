// 本檔由 cloud_farmer 依 specs/nmea2000_fast_packet.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::nmea2000_fast_packet::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(nmea2000_parse_header(0x40) == (2, 0), "spec golden vector #1");
    assert!(nmea2000_parse_header(0xA3) == (5, 3), "spec golden vector #2");
    let mut buf = [0u8; 223];
    let f = [0x40, 10, b'A', b'B', b'C', b'D', b'E', b'F'];
    assert!(nmea2000_extract_first_frame(&f, &mut buf) == Some((2, 10)), "spec golden vector #3");
    assert!(&buf[..6] == b"ABCDEF", "spec golden vector #4");
    let f_not_first = [0x41, 10, 0, 0, 0, 0, 0, 0];
    assert!(nmea2000_extract_first_frame(&f_not_first, &mut buf).is_none(), "spec golden vector #5");
    let f_overflow = [0x40, 250, 0, 0, 0, 0, 0, 0];
    assert!(nmea2000_extract_first_frame(&f_overflow, &mut buf).is_none(), "spec golden vector #6");
}
