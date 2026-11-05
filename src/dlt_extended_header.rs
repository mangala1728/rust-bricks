// ## Map
// Role: AUTOSAR DLT extended header binary field parser
// Inputs: buf: &[u8]
// Outputs: Option<(bool, u8, u8, u8, [u8; 4], [u8; 4])>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: buf.len() [10..usize::MAX]
///
/// Parse a 10‑byte DLT extended header according to AUTOSAR PRS_DLT_00001.
///
/// Returns `None` when the input slice is shorter than 10 bytes, otherwise
/// returns a tuple containing:
/// - `is_verbose` – verbose flag (bit 0 of MSIN)
/// - `msg_type`   – message type (bits 1‑3 of MSIN)
/// - `log_level`  – log level (bits 4‑7 of MSIN)
/// - `noar`       – number of additional arguments (byte 1)
/// - `app_id`     – 4‑byte application identifier (bytes 2‑5)
/// - `context_id` – 4‑byte context identifier (bytes 6‑9)
pub fn dlt_parse_extended_header(
    buf: &[u8],
) -> Option<(bool, u8, u8, u8, [u8; 4], [u8; 4])> {
    // @safety-proof: length guard guarantees safe indexing
    if buf.len() < 10 {
        return None;
    }

    let msin = buf[0];
    let is_verbose = (msin & 0x01) != 0;
    let msg_type = (msin >> 1) & 0x07;
    let log_level = (msin >> 4) & 0x0F;
    let noar = buf[1];

    let app_id = [buf[2], buf[3], buf[4], buf[5]];
    let context_id = [buf[6], buf[7], buf[8], buf[9]];

    Some((is_verbose, msg_type, log_level, noar, app_id, context_id))
}
