use rust_bricks::j1939_pgn_parser::*;

#[test]
fn golden_vector_pdu2_broadcast() {
    // 0x18FEE600 – Time/Date (broadcast)
    let can_id = 0x18FEE600_u32;
    let expected = Some((6_u8, 0xFEE6_u32, 0xE6_u8, 0x00_u8));
    assert_eq!(j1939_parse_can_id(can_id), expected);
}

#[test]
fn golden_vector_pdu1_point_to_point() {
    // 0x18EA0001 – Request PGN (point‑to‑point)
    let can_id = 0x18EA0001_u32;
    let expected = Some((6_u8, 0xEA00_u32, 0x00_u8, 0x01_u8));
    assert_eq!(j1939_parse_can_id(can_id), expected);
}

#[test]
fn test_boundaries_and_pdu_check() {
    // Beyond 29-bit limit
    assert_eq!(j1939_parse_can_id(0x2000_0000), None);
    // Boundary max 29-bit
    assert_eq!(
        j1939_parse_can_id(0x1FFF_FFFF),
        Some((7_u8, 0x1FFFF_u32, 0xFF_u8, 0xFF_u8))
    );
    // Zero CAN ID
    assert_eq!(
        j1939_parse_can_id(0x0000_0000),
        Some((0_u8, 0x0000_u32, 0x00_u8, 0x00_u8))
    );
    // PDU1 vs PDU2 boundary (239 vs 240)
    assert!(j1939_is_pdu1(239));
    assert!(!j1939_is_pdu1(240));
}
