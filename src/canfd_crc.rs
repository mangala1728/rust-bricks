// ## Map
// Role: CRC-17/CAN-FD（RevEng 參數）位元組資料校驗
// Inputs: crc: u32, bit: u8, data: &[u8]
// Outputs: Option<u32>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines
/// @outputs: 2
/// @domain: crc [0..0x1FFFF]
/// @domain: bit [0..1]
pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
    // domain guard
    if crc > 0x1_FFFF || bit > 1 {
        return None;
    }

    // ISO 11898‑1 shift‑register step
    let feedback = ((crc >> 16) & 1) ^ (bit as u32);
    let shifted = (crc << 1) & 0x1_FFFF;

    // apply polynomial when feedback is 1
    if feedback != 0 {
        Some(shifted ^ 0x1685B)
    } else {
        Some(shifted)
    }
}

/// @outputs: 2
/// @domain: data.len() [0..16]
pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
    // domain guard
    if data.len() > 16 {
        return None;
    }

    let mut crc: u32 = 0;
    for &byte in data {
        // process bits MSB → LSB
        let mut mask = 0x80u8;
        while mask != 0 {
            let bit = if (byte & mask) != 0 { 1 } else { 0 };
            // safe propagation of the Option result
            match canfd_crc17_bit(crc, bit) {
                Some(next) => crc = next,
                None => return None, // should never happen because of guards
            }
            mask >>= 1;
        }
    }
    Some(crc)
}
