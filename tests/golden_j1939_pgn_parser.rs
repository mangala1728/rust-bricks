// 本檔由 cloud_farmer 依 specs/j1939_pgn_parser.md 的 ```golden 區塊自動產生：預期值只能改規格書，不得手改本檔。
use rust_bricks::j1939_pgn_parser::*;

#[test]
fn golden_vectors_from_spec() {
    assert!(j1939_parse_can_id(0x18FEE600) == Some((6, 65254, 0xE6, 0x00)), "spec golden vector #1");
    assert!(j1939_parse_can_id(0x18EA0001) == Some((6, 59904, 0x00, 0x01)), "spec golden vector #2");
    assert!(j1939_parse_can_id(0x2000_0000).is_none(), "spec golden vector #3");
    assert!(j1939_parse_can_id(0xFFFF_FFFF).is_none(), "spec golden vector #4");
    assert!(j1939_is_pdu1(0xEA), "spec golden vector #5");
    assert!(!j1939_is_pdu1(0xFE), "spec golden vector #6");
    assert!(j1939_is_pdu1(239), "spec golden vector #7");
    assert!(!j1939_is_pdu1(240), "spec golden vector #8");
}
