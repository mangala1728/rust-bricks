// ## Map
// Role: J1939 29‑bit CAN identifier parser
// Inputs: can_id_29bit [0..0x1FFFFFFF], pf [0..0xFF]
// Outputs: (priority:u8, pgn:u32, ps:u8, sa:u8) or bool
// Invariants:
//   - `priority` is always 0..7
//   - `pgn` fits in 18 bits (0..0x3FFFF)
//   - `ps` and `sa` are raw 8‑bit fields from the identifier

/// @outputs: 4
/// @domain: can_id_29bit [0..0x1FFFFFFF]
pub fn j1939_parse_can_id(can_id_29bit: u32) -> Option<(u8, u32, u8, u8)> {
    // Guard: identifier must be 29‑bit
    if can_id_29bit > 0x1FFF_FFFF {
        return None;
    }

    // Extract fields (no nesting deeper than 2)
    let priority = ((can_id_29bit >> 26) & 0x07) as u8;
    let dp = ((can_id_29bit >> 24) & 0x01) as u32;
    let pf = ((can_id_29bit >> 16) & 0xFF) as u8;
    let ps = ((can_id_29bit >> 8) & 0xFF) as u8;
    let sa = (can_id_29bit & 0xFF) as u8;

    // PGN calculation follows the J1939 rule
    let pgn = if j1939_is_pdu1(pf) {
        // PDU1 – PS is destination address, not part of PGN
        (dp << 16) | ((pf as u32) << 8)
    } else {
        // PDU2 – PS (group extension) is part of PGN
        (dp << 16) | ((pf as u32) << 8) | (ps as u32)
    };

    Some((priority, pgn, ps, sa))
}

/// @outputs: 1
/// @domain: pf [0..0xFF]
pub fn j1939_is_pdu1(pf: u8) -> bool {
    pf < 240
}
