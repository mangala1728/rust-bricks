// ## Map
// Role: ISO 11898-1:2015 CAN-FD 17-bit CRC calculation
// Inputs: crc: u32, bit: u8, data: &[u8]
// Outputs: Option<u32> or u32
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: crc [0..0x1FFFF], bit [0..1]
pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
    // domain guard
    if crc > 0x1_FFFF || bit > 1 {
        return None;
    }

    // extract most‑significant bit of the 17‑bit CRC
    let msb = (crc >> 16) & 1;
    // shift left, keep only 17 bits, and insert the new input bit
    let next = ((crc << 1) & 0x1_FFFF) | (bit as u32);
    // polynomial: x^17 + x^16 + x^14 + x^13 + x^11 + x^6 + x^4 + x^3 + x^1 + 1 = 0x1685B
    let res = if msb != 0 { next ^ 0x1685B } else { next };
    Some(res)
}

/// @outputs: 1
/// @domain: data.len() [0..64]
pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
    // domain guard
    if data.len() > 64 {
        return None;
    }

    let mut crc: u32 = 0;
    for &byte in data {
        // iterate from bit 7 (MSB) down to bit 0 (LSB)
        for shift in (0..8).rev() {
            let bit = (byte >> shift) & 1;
            // safe handling of the Option returned by `canfd_crc17_bit`
            if let Some(next_crc) = canfd_crc17_bit(crc, bit as u8) {
                crc = next_crc;
            } else {
                // This branch is unreachable because inputs are guarded,
                // but we return None to keep the function total.
                return None;
            }
        }
    }
    Some(crc)
}
