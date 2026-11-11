// ## Map
// Role: Conversion Brick (09‑號合約)
// Inputs: milli_deg: u32, arcmin: u32
// Outputs: Option<u32> (converted value)
// Invariants: 0 ≤ milli_deg ≤ 360_000, 0 ≤ arcmin ≤ 21_600
//
// This file contains **zero** external dependencies and follows the 09‑號合約
// boundary rules (≤ 60 行, nesting depth ≤ 2, no `static mut`, no recursion).

#![allow(dead_code)]

/// Convert an angle expressed in **milli‑degrees** to **arc‑minutes**.
///
/// @domain: milli_deg [0..=360_000]
/// @outputs: 1
///
/// The conversion formula is:
/// `arcmin = (milli_deg * 3) / 50`
///
/// Returns `None` if the input is outside its declared domain.
pub fn milli_deg_to_arcmin(milli_deg: u32) -> Option<u32> {
    // ---- boundary guard ----------------------------------------------------
    if milli_deg < 0 || milli_deg > 360_000 {
        return None;
    }
    // -----------------------------------------------------------------------
    // Use a wider integer to avoid overflow of the intermediate product.
    let product = milli_deg as u64 * 3;
    // The division is exact for all valid inputs because 3/50 = 0.06.
    // Integer division truncates toward zero, which matches the spec.
    Some((product / 50) as u32)
}

/// Convert an angle expressed in **arc‑minutes** to **milli‑degrees**.
///
/// @domain: arcmin [0..=21_600]
/// @outputs: 1
///
/// The conversion formula is:
/// `milli_deg = (arcmin * 50) / 3`
///
/// Returns `None` if the input is outside its declared domain.
pub fn arcmin_to_milli_deg(arcmin: u32) -> Option<u32> {
    // ---- boundary guard ----------------------------------------------------
    if arcmin < 0 || arcmin > 21_600 {
        return None;
    }
    // -----------------------------------------------------------------------
    let product = arcmin as u64 * 50;
    Some((product / 3) as u32)
}
