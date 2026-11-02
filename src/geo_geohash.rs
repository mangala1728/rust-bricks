// ## Map
// Role: Geohash Bit‑Interleaving Codec
// Inputs: lat f64, lon f64, precision_bits u8, hash u64
// Outputs: encoded u64, decoded bounds ((f64,f64),(f64,f64))
// Invariants: 1 ≤ precision_bits ≤ 64, lat ∈ [‑90.0, 90.0], lon ∈ [‑180.0, 180.0]

/// Geohash encoder – converts latitude / longitude to a bit‑interleaved integer.
///
/// @outputs: 1
/// @domain: lat [-90.0..=90.0]
/// @domain: lon [-180.0..=180.0]
/// @domain: precision_bits [1..=64]
pub fn geo_geohash_encode_bits(lat: f64, lon: f64, precision_bits: u8) -> Option<u64> {
    // Guard checks
    if !lat.is_finite() || lat < -90.0 || lat > 90.0 {
        return None;
    }
    if !lon.is_finite() || lon < -180.0 || lon > 180.0 {
        return None;
    }
    if precision_bits == 0 || precision_bits > 64 {
        return None;
    }

    let mut lat_min = -90.0_f64;
    let mut lat_max = 90.0_f64;
    let mut lon_min = -180.0_f64;
    let mut lon_max = 180.0_f64;
    let mut hash: u64 = 0;

    let mut i: u8 = 0;
    while i < precision_bits {
        if i % 2 == 0 {
            // longitude step
            let mid = (lon_min + lon_max) * 0.5;
            if lon >= mid {
                hash = (hash << 1) | 1;
                lon_min = mid;
            } else {
                hash <<= 1;
                lon_max = mid;
            }
        } else {
            // latitude step
            let mid = (lat_min + lat_max) * 0.5;
            if lat >= mid {
                hash = (hash << 1) | 1;
                lat_min = mid;
            } else {
                hash <<= 1;
                lat_max = mid;
            }
        }
        i += 1;
    }
    Some(hash)
}

/// Geohash decoder – expands a bit‑interleaved integer into its latitude / longitude bounding box.
///
/// @outputs: 4
/// @domain: precision_bits [1..=64]
pub fn geo_geohash_decode_bounds(
    hash: u64,
    precision_bits: u8,
) -> Option<((f64, f64), (f64, f64))> {
    // Guard checks
    if precision_bits == 0 || precision_bits > 64 {
        return None;
    }

    let mut lat_min = -90.0_f64;
    let mut lat_max = 90.0_f64;
    let mut lon_min = -180.0_f64;
    let mut lon_max = 180.0_f64;

    // Process bits from most‑significant to least‑significant
    let mut bit_index: i8 = (precision_bits as i8) - 1;
    while bit_index >= 0 {
        let step = (precision_bits as i8 - 1 - bit_index) as u8; // 0‑based step count
        let bit_val = (hash >> (bit_index as u32)) & 1;
        if step % 2 == 0 {
            // longitude step
            let mid = (lon_min + lon_max) * 0.5;
            if bit_val == 1 {
                lon_min = mid;
            } else {
                lon_max = mid;
            }
        } else {
            // latitude step
            let mid = (lat_min + lat_max) * 0.5;
            if bit_val == 1 {
                lat_min = mid;
            } else {
                lat_max = mid;
            }
        }
        bit_index -= 1;
    }

    Some(((lat_min, lat_max), (lon_min, lon_max)))
}
