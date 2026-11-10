// ## Map
// Role: README – documentation & small utility bricks
// Inputs:  milli_deg: u32 [0..360_000]   // milli‑degrees (fixed‑point)
// Outputs: arcmin:   u32 [0..21_600]    // arc‑minutes (1/60 deg)
// Invariants:  arcmin = floor( milli_deg * 3 / 50 )
//               conversion never overflows for the declared domain.

/// Convert an angle expressed in **milli‑degrees** to **arc‑minutes**.
///
/// # Parameters
/// * `milli_deg` – angle in milli‑degrees.  
///
/// # Returns
/// * `Some(arcmin)` – the converted value when the input lies inside the
///   declared domain `0..=360_000`.
/// * `None` – when the input is out of range.
///
/// @outputs: 1
/// @domain: milli_deg [0..360_000]
pub fn milli_deg_to_arcmin(milli_deg: u32) -> Option<u32> {
    // @safety-proof: pure arithmetic, no overflow for the allowed range.
    if milli_deg > 360_000 {
        return None;
    }
    // The multiplication fits safely into u64 (max 1_080_000) → no overflow.
    let arcmin = (milli_deg as u64 * 3) / 50;
    Some(arcmin as u32)
}
