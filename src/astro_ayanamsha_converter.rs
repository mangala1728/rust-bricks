//! ## Map
//! Role:   Conversion between Tropical and Sidereal ecliptic longitudes.
//! Inputs: tropical_milli_deg ∈ [0..360000], sidereal_milli_deg ∈ [0..360000],
//!         ayanamsha_arcsec ∈ [0..1296000] (arc‑seconds of precession).
//! Outputs: Converted longitude in milli‑degrees (0..360000).
//! Invariants: All arithmetic stays within u32 bounds; results are always
//!             reduced modulo 360 000 milli‑deg.

/// @outputs: 1
/// @domain: tropical_milli_deg [0..360000]
/// @domain: ayanamsha_arcsec [0..1296000]
pub fn astro_tropical_to_sidereal(
    tropical_milli_deg: u32,
    ayanamsha_arcsec: u32,
) -> Option<u32> {
    // Guard: reject out‑of‑range inputs.
    if tropical_milli_deg > 360_000 || ayanamsha_arcsec > 1_296_000 {
        return None;
    }
    // Convert ayanamsha from arc‑seconds to milli‑degrees (rounded).
    let ayan_milli = (ayanamsha_arcsec * 1_000 + 1_800) / 3_600;
    // Apply the conversion formula and wrap around 360 000.
    Some((tropical_milli_deg + 360_000 - ayan_milli) % 360_000)
}

/// @outputs: 1
/// @domain: sidereal_milli_deg [0..360000]
/// @domain: ayanamsha_arcsec [0..1296000]
pub fn astro_sidereal_to_tropical(
    sidereal_milli_deg: u32,
    ayanamsha_arcsec: u32,
) -> Option<u32> {
    // Guard: reject out‑of‑range inputs.
    if sidereal_milli_deg > 360_000 || ayanamsha_arcsec > 1_296_000 {
        return None;
    }
    // Convert ayanamsha from arc‑seconds to milli‑degrees (rounded).
    let ayan_milli = (ayanamsha_arcsec * 1_000 + 1_800) / 3_600;
    // Apply the reverse conversion formula and wrap around 360 000.
    Some((sidereal_milli_deg + ayan_milli) % 360_000)
}
