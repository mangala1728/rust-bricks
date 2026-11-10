// ## Map
// Role: IEC 61158-4-12 EtherCAT 10-byte datagram header parser
// Inputs: header: &[u8]
// Outputs: Option<EthercatDatagramHeader>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// Representation of a 10‑byte EtherCAT datagram header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthercatDatagramHeader {
    pub cmd: u8,
    pub idx: u8,
    pub address: u32,
    pub data_len: u16,
    pub more_follows: bool,
    pub irq: u16,
}

/// @outputs: 1
/// @domain: header.len() >= 10, cmd > 0 && cmd <= 14, data_len <= 1498
pub fn ethercat_parse_datagram_header(header: &[u8]) -> Option<EthercatDatagramHeader> {
    // Guard: header must contain at least 10 bytes.
    if header.len() < 10 {
        return None;
    }

    // Guard: command code must be in the range 1..=14.
    let cmd = header[0];
    if cmd == 0 || cmd > 14 {
        return None;
    }

    // Index byte – no range restriction.
    let idx = header[1];

    // 32‑bit address, little‑endian.
    let address = u32::from_le_bytes([header[2], header[3], header[4], header[5]]);

    // Length / flags field, little‑endian.
    let len_flags = u16::from_le_bytes([header[6], header[7]]);
    let data_len = len_flags & 0x07FF; // lower 11 bits
    // Guard: data length must not exceed the Ethernet payload limit.
    if data_len > 1498 {
        return None;
    }
    let more_follows = (len_flags & 0x8000) != 0;

    // IRQ field, little‑endian.
    let irq = u16::from_le_bytes([header[8], header[9]]);

    Some(EthercatDatagramHeader {
        cmd,
        idx,
        address,
        data_len,
        more_follows,
        irq,
    })
}
