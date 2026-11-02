// ## Map
// Role: NMEA‑0183 GPRMC parser – checksum validation & latitude/longitude conversion.
// Inputs: &str sentences, raw latitude/longitude strings, direction chars.
// Outputs: bool (checksum ok), Option<(f64,f64)> (decimal degrees).
// Invariants:
//   - Sentence must start with '$' and contain '*' followed by exactly two hex digits.
//   - Latitude raw format: ddmm.mmmm, Longitude raw format: dddmm.mmmm.
//   - Direction chars limited to N,S,E,W.
//   - Resulting decimal degrees must lie within [-90,90] for latitude and [-180,180] for longitude.

/// Validates the XOR checksum of an NMEA‑0183 sentence.
///
/// @outputs: 1
pub fn nmea_validate_checksum(sentence: &str) -> bool {
    // Sentence must start with '$'
    if !sentence.starts_with('$') {
        return false;
    }

    // Locate the asterisk that precedes the checksum
    let asterisk_pos = match sentence.find('*') {
        Some(pos) => pos,
        None => return false,
    };

    // There must be exactly two hex digits after '*'
    if sentence.len() < asterisk_pos + 3 {
        return false;
    }
    let checksum_str = &sentence[asterisk_pos + 1..asterisk_pos + 3];

    // Compute XOR of all bytes between '$' (excluded) and '*' (excluded)
    let mut calc: u8 = 0;
    for b in sentence.as_bytes().iter().skip(1).take(asterisk_pos - 1) {
        calc ^= *b;
    }

    // Parse the provided checksum (hex)
    let provided = match u8::from_str_radix(checksum_str, 16) {
        Ok(v) => v,
        Err(_) => return false,
    };

    calc == provided
}

/// Parses latitude and longitude expressed in NMEA ddmm.mmmm / dddmm.mmmm format
/// and converts them to decimal degrees.
///
/// @outputs: 2
pub fn nmea_parse_lat_lon(
    raw_lat: &str,
    lat_dir: char,
    raw_lon: &str,
    lon_dir: char,
) -> Option<(f64, f64)> {
    // ----- Latitude -----
    // Expect at least 4 characters (ddmm) plus optional fraction
    if raw_lat.len() < 4 {
        return None;
    }
    // First two chars = degrees, rest = minutes
    let (lat_deg_str, lat_min_str) = raw_lat.split_at(2);
    let lat_deg: f64 = match lat_deg_str.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let lat_min: f64 = match lat_min_str.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let mut lat = lat_deg + lat_min / 60.0;
    match lat_dir {
        'N' => {}
        'S' => lat = -lat,
        _ => return None,
    }

    // ----- Longitude -----
    if raw_lon.len() < 5 {
        return None;
    }
    // First three chars = degrees, rest = minutes
    let (lon_deg_str, lon_min_str) = raw_lon.split_at(3);
    let lon_deg: f64 = match lon_deg_str.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let lon_min: f64 = match lon_min_str.parse() {
        Ok(v) => v,
        Err(_) => return None,
    };
    let mut lon = lon_deg + lon_min / 60.0;
    match lon_dir {
        'E' => {}
        'W' => lon = -lon,
        _ => return None,
    }

    // ----- Bounds check -----
    if !( -90.0..=90.0 ).contains(&lat) {
        return None;
    }
    if !( -180.0..=180.0 ).contains(&lon) {
        return None;
    }

    Some((lat, lon))
}
