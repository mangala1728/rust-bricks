// ## Map
// Role: DLT Standard Header Parser
// Inputs: buf: &[u8] (binary DLT message)
// Outputs: (vers: u8, ueh: bool, len: u16, header_len: usize)
// Invariants: buf.len() >= 4, version == 1, length field >= header_len

/// AUTOSAR DLT – parse the *Standard Header* (HTYP, MCNT, LEN, optional fields).
///
/// @outputs: 4
/// @domain: buf.len() [4..usize::MAX]
pub fn dlt_parse_standard_header(buf: &[u8]) -> Option<(u8, bool, u16, usize)> {
    // Guard: minimum size for the fixed part of the header
    if buf.len() < 4 {
        return None;
    }

    let htyp = buf[0];
    let vers = (htyp >> 5) & 0x07;
    // Version must be 1 according to the AUTOSAR DLT spec
    if vers != 1 {
        return None;
    }

    let ueh = (htyp & 0x01) != 0;          // Use Extended Header flag
    let big_endian = (htyp & 0x02) != 0;   // Endianness flag
    let weid = (htyp & 0x04) != 0;         // With ECU ID flag
    let wsid = (htyp & 0x08) != 0;         // With Session ID flag
    let wtms = (htyp & 0x10) != 0;         // With Timestamp flag

    // MCNT is present but not needed for the current contract
    let _mcnt = buf[1];

    // Decode the length field according to the endianness flag
    let len = if big_endian {
        u16::from_be_bytes([buf[2], buf[3]])
    } else {
        u16::from_le_bytes([buf[2], buf[3]])
    };

    // Compute the total header length including optional fields
    let header_len = 4
        + if weid { 4 } else { 0 }
        + if wsid { 4 } else { 0 }
        + if wtms { 4 } else { 0 };

    // Validate that the buffer contains the whole header and that the length field
    // is not smaller than the header itself.
    if buf.len() < header_len || (len as usize) < header_len {
        return None;
    }

    Some((vers, ueh, len, header_len))
}

/// Extract the 4‑byte ECU ID from a DLT message that carries the *WEID* flag.
///
/// @outputs: 1
/// @domain: buf.len() [8..usize::MAX]
pub fn dlt_extract_ecu_id(buf: &[u8]) -> Option<[u8; 4]> {
    // Guard: need at least the fixed header + ECU ID field
    if buf.len() < 8 {
        return None;
    }

    // Verify that the WEID flag is set in HTYP
    if (buf[0] & 0x04) == 0 {
        return None;
    }

    // ECU ID starts at offset 4 (immediately after the fixed 4‑byte header)
    Some([buf[4], buf[5], buf[6], buf[7]])
}
