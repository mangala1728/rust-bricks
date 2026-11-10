// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Ultra-high-order divisional chart (Varga: D81, D108, D144, D150) exact integer solver
//! - Inputs:
//!   - sidereal_arcsec: u32 (0..=1296000, 360 deg in exact arcseconds)
//!   - varga_n: u16 (1..=300, e.g. 81, 108, 144, 150)
//! - Outputs:
//!   - Option<(u8, u16, u16)>: (varga_sign [1..12], sub_index [0..varga_n-1], intra_permille [0..999])
//!   - Option<u16>: Exact integer steps per Nakshatra Pada (3°20' = 12,000")
//! - Invariants:
//!   1. Strict 09 contract boundary & NASA JPL <= 60 lines per function.
//!   2. 100% pure integer arithmetic: 1 sign = 108,000", D108 step = 1,000", D144 step = 750".
//!   3. Mathematical harmony: 1 Pada = 9 steps of D81, 12 steps of D108, 16 steps of D144.

/// Computes the exact high-order Varga placement using arcsecond integer math.
/// Returns (varga_sign [1..12], sub_index [0..varga_n-1], intra_permille [0..999]).
/// @outputs: 1
/// @domain: sidereal_arcsec [0..=1296000]
/// @domain: varga_n [1..=300]
pub fn astro_super_varga_arcsec(
    sidereal_arcsec: u32,
    varga_n: u16,
) -> Option<(u8, u16, u16)> {
    if sidereal_arcsec > 1_296_000 || !(1..=300).contains(&varga_n) {
        return None;
    }

    let clamped = sidereal_arcsec % 1_296_000;
    let rashi_sign = (clamped / 108_000) as u8; // 0=Aries .. 11=Pisces
    let pos_in_sign = clamped % 108_000; // 0..107999"

    // Pure integer projection: sub_idx = (pos * varga_n) / 108,000
    let pos_u64 = pos_in_sign as u64;
    let n_u64 = varga_n as u64;
    let scaled = pos_u64.saturating_mul(n_u64);
    let sub_idx = (scaled / 108_000) as u16;

    // Intra-subdivision progress in permille (0..999 permille)
    let rem = scaled % 108_000;
    let intra_permille = ((rem.saturating_mul(1_000)) / 108_000) as u16;

    // Varga cycle projection: D81/D9 use triplicity start, D108/D12/D144 use sign start
    let start_offset = if varga_n % 9 == 0 && varga_n % 12 != 0 {
        // Navamsha-based cyclic family (D9, D81): Movable, Fixed, Dual
        match rashi_sign % 4 {
            0 => rashi_sign,
            1 => (rashi_sign + 8) % 12,
            _ => (rashi_sign + 4) % 12,
        }
    } else {
        // Dwadasamsa-based family (D12, D108, D144) starts from the sign itself
        rashi_sign
    };

    let target_sign = ((start_offset as u16 + sub_idx) % 12) as u8 + 1;
    Some((target_sign, sub_idx, intra_permille))
}

/// Returns the exact number of Varga subdivision steps contained in 1 Nakshatra Pada (3°20' = 12,000").
/// If 12,000" is not evenly divisible by the step size, returns None.
/// @outputs: 1
/// @domain: varga_n [1..=300]
pub fn astro_pada_to_super_varga_steps(varga_n: u16) -> Option<u16> {
    if !(1..=300).contains(&varga_n) {
        return None;
    }

    // 1 sign = 108,000". 1 Pada = 12,000" = 1/9 of a sign.
    // Therefore, steps_in_pada = varga_n / 9.
    if varga_n % 9 == 0 {
        Some(varga_n / 9)
    } else if (12_000u32.saturating_mul(varga_n as u32)) % 108_000 == 0 {
        Some(((12_000u32 * (varga_n as u32)) / 108_000) as u16)
    } else {
        None
    }
}
