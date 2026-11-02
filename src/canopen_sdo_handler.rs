// ## Map
// Role: SDO Handler – parses CANopen SDO download requests and builds abort responses.
// Inputs: payload (&[u8;8]), index (u16), sub_index (u8), abort_code (u32), out_frame (&mut [u8;8]).
// Outputs: Option<(u8, u16, u8, usize)>, bool.
// Invariants:
//   - For a download request the Command Specifier (CCS) must be 1.
//   - Abort frames always start with 0x80 (CCS = 4).

/// @outputs: 4
///
/// Parses an Initiate Download Request (8‑byte SDO payload).
///
/// Returns `Some((ccs, index, sub_index, valid_data_len))` when the payload
/// contains a valid download request, otherwise `None`.
pub fn parse_sdo_download_request(payload: &[u8; 8]) -> Option<(u8, u16, u8, usize)> {
    // Extract the Command Specifier (CCS) – bits 7..5.
    let ccs = (payload[0] >> 5) & 0x07;
    if ccs != 1 {
        return None;
    }

    // Flags
    let e = (payload[0] >> 1) & 0x01; // expedited flag
    let s = payload[0] & 0x01;       // size indicated flag

    // Index (little‑endian) and sub‑index.
    let index = (payload[1] as u16) | ((payload[2] as u16) << 8);
    let sub_index = payload[3];

    // Determine the length of valid data.
    let valid_len = if e == 1 {
        if s == 1 {
            // Number of unused bytes (n) = bits 3..2.
            let n = ((payload[0] >> 2) & 0x03) as usize;
            4usize.saturating_sub(n)
        } else {
            // Size not indicated – default to 4 bytes.
            4usize
        }
    } else {
        // Segmented transfer – no data in the header.
        0usize
    };

    Some((ccs, index, sub_index, valid_len))
}

/// @outputs: 1
///
/// @domain: index [0..65535]
/// @domain: sub_index [0..255]
/// @domain: abort_code [0..4294967295]
///
/// Encodes an SDO Abort Transfer response into `out_frame`.
///
/// The function always returns `true` after writing the frame.
pub fn encode_sdo_abort(
    index: u16,
    sub_index: u8,
    abort_code: u32,
    out_frame: &mut [u8; 8],
) -> bool {
    // Fixed abort header.
    out_frame[0] = 0x80; // CCS = 4 (Abort SDO Transfer)
    out_frame[1] = (index & 0x00FF) as u8;
    out_frame[2] = ((index >> 8) & 0x00FF) as u8;
    out_frame[3] = sub_index;

    // Abort code – little‑endian.
    out_frame[4] = (abort_code & 0x0000_00FF) as u8;
    out_frame[5] = ((abort_code >> 8) & 0x0000_00FF) as u8;
    out_frame[6] = ((abort_code >> 16) & 0x0000_00FF) as u8;
    out_frame[7] = ((abort_code >> 24) & 0x0000_00FF) as u8;

    true
}
