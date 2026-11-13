// ## Map
// Role: WGS-84 coordinates to UTM Zone, hemisphere, and central meridian calculator
// Inputs: lat_deg: f64, lon_deg: f64
// Outputs: Option<(u8, char, f64)> (zone, hemisphere, central_meridian_deg)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: lat_deg in [-80.0..=84.0], lon_deg in [-180.0..=180.0]
pub fn geo_determine_utm_zone(
    lat_deg: f64,
    lon_deg: f64,
) -> Option<(u8, char, f64)> {
    if !lat_deg.is_finite()
        || !lon_deg.is_finite()
        || lat_deg < -80.0
        || lat_deg > 84.0
        || lon_deg < -180.0
        || lon_deg > 180.0
    {
        return None;
    }

    let lon_eff = if lon_deg == 180.0 { 179.9999 } else { lon_deg };
    let zone = (((lon_eff + 180.0) / 6.0).floor() as i32) + 1;

    if zone < 1 || zone > 60 {
        return None;
    }

    let zone_u8 = zone as u8;
    let central_meridian = ((zone as f64) - 1.0) * 6.0 - 180.0 + 3.0;
    let hemisphere = if lat_deg >= 0.0 { 'N' } else { 'S' };

    Some((zone_u8, hemisphere, central_meridian))
}
