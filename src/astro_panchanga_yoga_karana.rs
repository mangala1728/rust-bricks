//! ## Map
//! - **Role**: Vedic Panchanga Yoga (27) and Karana (60) integer calculation module.
//! - **Inputs**: `moon_milli_deg` (u32, [0..360000]), `sun_milli_deg` (u32, [0..360000]).
//! - **Outputs**: `Option<(u8, u8)>` containing (Yoga ID [1..27], Karana ID [1..60]).
//! - **Invariants**: Fixed-point integer arithmetic only; strict domain guards; zero allocation.

/// @outputs: 1
/// @domain: moon_milli_deg [0..360000]
/// @domain: sun_milli_deg [0..360000]
pub fn astro_panchanga_yoga_karana(moon_milli_deg: u32, sun_milli_deg: u32) -> Option<(u8, u8)> {
    if moon_milli_deg > 360_000 || sun_milli_deg > 360_000 {
        return None;
    }

    let sun_min = (sun_milli_deg * 3) / 50;
    let moon_min = (moon_milli_deg * 3) / 50;

    let sum_min = (sun_min + moon_min) % 21_600;
    let yoga_id = (sum_min / 800) + 1;

    let delta = (moon_milli_deg + 360_000 - sun_milli_deg) % 360_000;
    let karana_id = (delta / 6_000) + 1;

    Some((yoga_id as u8, karana_id as u8))
}
