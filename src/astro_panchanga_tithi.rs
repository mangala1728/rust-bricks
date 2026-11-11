//! ## Map
//! - Role: Pure integer Vedic Panchanga Tithi (1..30) and progress (0..9999 bp) calculation engine.
//! - Inputs: Moon celestial longitude (milli-degrees, 0..360000), Sun celestial longitude (milli-degrees, 0..360000).
//! - Outputs: `Option<(u8, u16)>` representing Tithi index (1..30) and progress basis points (0..9999 bp).
//! - Invariants: Fixed-point integer math only, zero float precision loss, strict parameter domain guard [0..360000].

/// @outputs: 1
/// @domain: moon_milli_deg [0..360000]
/// @domain: sun_milli_deg [0..360000]
pub fn astro_panchanga_tithi(moon_milli_deg: u32, sun_milli_deg: u32) -> Option<(u8, u16)> {
    if moon_milli_deg > 360_000 || sun_milli_deg > 360_000 {
        return None;
    }

    let delta = (moon_milli_deg + 360_000 - sun_milli_deg) % 360_000;
    let tithi_id = ((delta / 12_000) + 1) as u8;
    let rem = delta % 12_000;
    let progress_bp = ((rem * 10_000) / 12_000) as u16;

    Some((tithi_id, progress_bp))
}
