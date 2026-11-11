//! ## Map
//! **Role**: Compute Vedic 27‑Nakshatra (宿) and 4‑Pada (足) from lunar longitude.
//! **Inputs**: `moon_milli_deg` – lunar ecliptic longitude in milli‑degrees (0 .. 360 000).
//! **Outputs**: `(nakshatra_id, pada_id, progress_bp)` – 1‑based Nakshatra (1‑27),
//! 1‑based Pada (1‑4) and progress within the Pada in base‑points (0 .. 9 999).
//! **Invariants**
//! - `nakshatra_id ∈ [1,27]`
//! - `pada_id ∈ [1,4]`
//! - `progress_bp ∈ [0,9 999]`
//! - All calculations are pure integer arithmetic; no floating‑point is used.

/// @outputs: 1
/// @domain: moon_milli_deg [0..360000]
///
/// Returns `None` if the input is outside the allowed domain.
/// Otherwise returns a tuple `(nakshatra_id, pada_id, progress_bp)`.
pub fn astro_panchanga_nakshatra(moon_milli_deg: u32) -> Option<(u8, u8, u16)> {
    // Guard: enforce domain 0..=360_000
    if moon_milli_deg > 360_000 {
        return None;
    }

    // Convert milli‑degrees to arc‑minutes (integer arithmetic)
    // arcmin = (moon_milli_deg * 3) / 50
    let arcmin = (moon_milli_deg * 3) / 50; // max 21 600, fits u32

    // Special handling for the exact 360° boundary
    if arcmin == 21_600 {
        // 27th Nakshatra, 4th Pada, 9999 bp (full progress)
        return Some((27, 4, 9_999));
    }

    // Each Nakshatra spans 800 arc‑minutes (13°20')
    let nakshatra_id = (arcmin / 800) + 1; // 1‑based, max 27
    let rem = arcmin % 800;                // remainder inside the Nakshatra

    // Each Pada spans 200 arc‑minutes (3°20')
    let pada_id = (rem / 200) + 1; // 1‑based, max 4

    // Progress within the Pada expressed in base‑points (0..9 999)
    // progress_bp = floor(rem * 10_000 / 800)
    let progress_bp = ((rem * 10_000) / 800) as u16;

    Some((nakshatra_id as u8, pada_id as u8, progress_bp))
}
