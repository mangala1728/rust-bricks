// ## Map
// Role:   Pure‑Rust packing / unpacking of Vimshottari Dasha data.
// Inputs: maha, antar, prat, sookshma, prana ∈ 1..=9, balance_permille ∈ 0..=4095
// Outputs: 1 packed u32, or unpacked tuple.
// Invariants: each field fits its allocated bit‑width, no overflow, no panics.

/// @outputs: 1
/// @domain: maha [1..9]
/// @domain: antar [1..9]
/// @domain: prat [1..9]
/// @domain: sookshma [1..9]
/// @domain: prana [1..9]
/// @domain: balance_permille [0..4095]
pub fn astro_vimshottari_dasha_pack(
    maha: u8,
    antar: u8,
    prat: u8,
    sookshma: u8,
    prana: u8,
    balance_permille: u16,
) -> Option<u32> {
    // guard: domain checks
    if maha < 1 || maha > 9 { return None; }
    if antar < 1 || antar > 9 { return None; }
    if prat < 1 || prat > 9 { return None; }
    if sookshma < 1 || sookshma > 9 { return None; }
    if prana < 1 || prana > 9 { return None; }
    if balance_permille > 4095 { return None; }

    // pack according to bit allocation
    let packed = (maha as u32) << 0
        | (antar as u32) << 4
        | (prat as u32) << 8
        | (sookshma as u32) << 12
        | (prana as u32) << 16
        | (balance_permille as u32) << 20;
    Some(packed)
}

/// @outputs: 1
pub fn astro_vimshottari_dasha_unpack(packed: u32) -> (u8, u8, u8, u8, u8, u16) {
    // extract each field using masks
    let maha = ((packed >> 0) & 0xF) as u8;
    let antar = ((packed >> 4) & 0xF) as u8;
    let prat = ((packed >> 8) & 0xF) as u8;
    let sookshma = ((packed >> 12) & 0xF) as u8;
    let prana = ((packed >> 16) & 0xF) as u8;
    let balance_permille = ((packed >> 20) & 0xFFF) as u16;
    (maha, antar, prat, sookshma, prana, balance_permille)
}
