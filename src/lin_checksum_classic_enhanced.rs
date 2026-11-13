// ## Map
// Role: LIN 1.3 Classic and LIN 2.x Enhanced checksum computation
// Inputs: pid: u8, data: &[u8], enhanced: bool
// Outputs: Option<u8>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: data.len() in [1..=8]
pub fn lin_compute_checksum(pid: u8, data: &[u8], enhanced: bool) -> Option<u8> {
    // domain guard – reject empty or over‑long payloads
    if data.is_empty() || data.len() > 8 {
        return None;
    }

    // Determine whether the PID participates in the sum.
    // Diagnostic frames (0x3C, 0x3D, or their 6‑bit masked forms) force Classic mode.
    let diagnostic = matches!(pid, 0x3C | 0x3D) || matches!(pid & 0x3F, 0x3C | 0x3D);
    let mut sum: u16 = if enhanced && !diagnostic { pid as u16 } else { 0 };

    // Accumulate payload bytes.
    for &b in data {
        sum += b as u16;
    }

    // First carry‑fold.
    if sum > 0xFF {
        sum = (sum & 0xFF) + (sum >> 8);
    }
    // Second possible fold (specification requires at most two).
    if sum > 0xFF {
        sum = (sum & 0xFF) + (sum >> 8);
    }

    // One’s complement of the low byte yields the checksum.
    Some(!(sum as u8))
}
