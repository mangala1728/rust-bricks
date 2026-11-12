// ## Map
// Role: Detector
// Inputs: tithi_start_ts: u64, tithi_end_ts: u64, sunrise_curr_ts: u64, sunrise_next_ts: u64
// Outputs: Option<u8> (0=Normal, 1=Kshaya, 2=Vriddhi)
// Invariants: timestamps ∈ [0, 4_102_444_800], tithi_start_ts < tithi_end_ts, sunrise_curr_ts < sunrise_next_ts

/// @outputs: 1
/// @domain: tithi_start_ts [0..4102444800]
/// @domain: tithi_end_ts [0..4102444800]
/// @domain: sunrise_curr_ts [0..4102444800]
/// @domain: sunrise_next_ts [0..4102444800]
pub fn astro_tithi_special_detector(
    tithi_start_ts: u64,
    tithi_end_ts: u64,
    sunrise_curr_ts: u64,
    sunrise_next_ts: u64,
) -> Option<u8> {
    // Guard: timestamps must not exceed the maximum allowed value
    if tithi_start_ts > 4_102_444_800
        || tithi_end_ts > 4_102_444_800
        || sunrise_curr_ts > 4_102_444_800
        || sunrise_next_ts > 4_102_444_800
    {
        return None;
    }
    // Guard: logical ordering of intervals
    if tithi_start_ts >= tithi_end_ts || sunrise_curr_ts >= sunrise_next_ts {
        return None;
    }
    // Kshaya (missing) – entirely between two sunrises
    if tithi_start_ts > sunrise_curr_ts && tithi_end_ts < sunrise_next_ts {
        return Some(1);
    }
    // Vriddhi (extra) – spans both sunrises
    if tithi_start_ts <= sunrise_curr_ts && tithi_end_ts >= sunrise_next_ts {
        return Some(2);
    }
    // Normal – any other valid configuration
    Some(0)
}
