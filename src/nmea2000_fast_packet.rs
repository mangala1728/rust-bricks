// ## Map
// Role: NMEA 2000 fast packet protocol multi-frame header parser
// Inputs: first_byte: u8, can_frame: &[u8; 8], out_buf: &mut [u8; 223]
// Outputs: (u8, u8) or Option<(u8, usize)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: first_byte <= 0xFF
pub fn nmea2000_parse_header(first_byte: u8) -> (u8, u8) {
    let sequence_id = (first_byte >> 5) & 0x07;
    let frame_index = first_byte & 0x1F;
    (sequence_id, frame_index)
}

/// @outputs: 2
/// @domain: can_frame.len() == 8
pub fn nmea2000_extract_first_frame(
    can_frame: &[u8; 8],
    out_buf: &mut [u8; 223],
) -> Option<(u8, usize)> {
    let (seq, index) = nmea2000_parse_header(can_frame[0]);
    if index != 0 {
        return None;
    }
    let total_len = can_frame[1] as usize;
    if total_len == 0 || total_len > 223 {
        return None;
    }

    let copy_len = total_len.min(6);
    out_buf[..copy_len].copy_from_slice(&can_frame[2..2 + copy_len]);

    Some((seq, total_len))
}

/// @outputs: 1
/// @domain: expected_seq <= 7, expected_index in [1..31]
pub fn nmea2000_extract_subsequent_frame(
    can_frame: &[u8; 8],
    expected_seq: u8,
    expected_index: u8,
    out_buf: &mut [u8; 223],
    total_len: usize,
) -> Option<usize> {
    if expected_seq > 7 || expected_index == 0 || expected_index > 31 || total_len > 223 {
        return None;
    }
    let (seq, index) = nmea2000_parse_header(can_frame[0]);
    if seq != expected_seq || index != expected_index {
        return None;
    }

    let offset = 6 + (index as usize - 1) * 7;
    if offset >= total_len {
        return None;
    }
    let remaining = total_len - offset;
    let copy_len = remaining.min(7);
    out_buf[offset..offset + copy_len].copy_from_slice(&can_frame[1..1 + copy_len]);

    Some(offset + copy_len)
}
