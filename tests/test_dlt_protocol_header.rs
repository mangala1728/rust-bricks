use rust_bricks::dlt_protocol_header::*;

#[test]
fn test_parse_standard_header_vector1() {
    // Vector 1 – basic 4‑byte header, big‑endian, no optional fields
    let buf = [0x22, 0x01, 0x00, 0x10];
    let result = dlt_parse_standard_header(&buf);
    assert_eq!(result, Some((1, false, 16, 4)));
}

#[test]
fn test_parse_standard_header_vector2() {
    // Vector 2 – includes ECU ID, little‑endian, ECU = "ECU1"
    let buf = [
        0x24, // HTYP: version=1, little‑endian, WEID=1
        0x05, // MCNT
        0x20, 0x00, // LEN = 0x0020 (32) little‑endian
        b'E', b'C', b'U', b'1', // ECU ID
    ];
    let result = dlt_parse_standard_header(&buf);
    assert_eq!(result, Some((1, false, 32, 8)));
}

#[test]
fn test_extract_ecu_id_vector2() {
    let buf = [
        0x24, 0x05, 0x20, 0x00,
        b'E', b'C', b'U', b'1',
    ];
    let ecu_id = dlt_extract_ecu_id(&buf);
    assert_eq!(ecu_id, Some([b'E', b'C', b'U', b'1']));
}
