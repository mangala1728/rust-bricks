// ## Map
// Role: Modbus RTU Frame Validator & CRC-16 Engine
// Inputs: data: &[u8], frame: &[u8]
// Outputs: crc: u16, Option<(slave_id: u8, function_code: u8, payload_len: usize)>
// Invariants: zero external dependencies, no recursion, deterministic CRC-16 (poly 0xA001)

/// @outputs: 1
pub fn modbus_crc16(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// @outputs: 3
pub fn modbus_validate_frame(frame: &[u8]) -> Option<(u8, u8, usize)> {
    if frame.len() < 4 {
        return None;
    }
    let data_end = frame.len() - 2;
    let expected_crc = modbus_crc16(&frame[..data_end]);
    let actual_crc = (frame[data_end] as u16) | ((frame[data_end + 1] as u16) << 8);
    if actual_crc != expected_crc {
        return None;
    }
    let slave_id = frame[0];
    let function_code = frame[1];
    let payload_len = data_end - 2;
    Some((slave_id, function_code, payload_len))
}
