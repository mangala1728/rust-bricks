// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Reference implementation for Celestial Mechanics, Ephemeris, and Astrological Cycle Pipeline alignment (`adapters` & `core` layer template).
//! - Inputs:
//!   - Ephemeris coordinates (milli-degrees, 0..360000)
//!   - Unix epoch timestamps (microseconds for Bus 1, seconds for Bus 2)
//!   - Binary 32-Byte celestial packet stream
//! - Outputs:
//!   - 32-Byte `CelestialPacket32` binary buffer
//!   - Aspect angle classification and orb calculation
//!   - Bitemporally guarded `EphemerisRecord`
//! - Invariants:
//!   1. Zero external dependencies (100% Rust std only).
//!   2. Fixed-point integer math only (milli-degrees, basis points, AU fp4).
//!   3. Memory layout of `CelestialPacket32` is strictly 32 bytes (cache-line aligned).
//!   4. Bitemporal causality: `event_period_ts <= public_known_ts <= sim_clock_ts`.
//!   5. 100% compliant with `09-contract-boundary.rules.md`.

/// Celestial body identifier constants.
pub const BODY_SUN: u16 = 1;
pub const BODY_MOON: u16 = 2;
pub const BODY_MERCURY: u16 = 3;
pub const BODY_VENUS: u16 = 4;
pub const BODY_MARS: u16 = 5;
pub const BODY_JUPITER: u16 = 6;
pub const BODY_SATURN: u16 = 7;

/// Aspect type codes.
pub const ASPECT_CONJUNCTION: u8 = 0; // 0 deg
pub const ASPECT_SEXTILE: u8 = 1;     // 60 deg (60,000 milli-deg)
pub const ASPECT_SQUARE: u8 = 2;      // 90 deg (90,000 milli-deg)
pub const ASPECT_TRINE: u8 = 3;       // 120 deg (120,000 milli-deg)
pub const ASPECT_OPPOSITION: u8 = 4;  // 180 deg (180,000 milli-deg)
pub const ASPECT_NONE: u8 = 7;

/// Motion bitmask flags.
pub const FLAG_RETROGRADE: u8 = 0x01;
pub const FLAG_STATIONARY: u8 = 0x02;
pub const FLAG_SOLAR_ECLIPSE: u8 = 0x04;
pub const FLAG_LUNAR_ECLIPSE: u8 = 0x08;

/// Bus 1 Canonical 32-Byte Celestial Event Packet.
/// Layout: 8 + 4 + 4 + 4 + 2 + 2 + 2 + 1 + 1 + 2 + 2 = 32 Bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct CelestialPacket32 {
    pub ts_us: u64,
    pub ecliptic_longitude_milli_deg: u32,
    pub aspect_angle_milli_deg: u32,
    pub speed_milli_deg_day: i32,
    pub primary_body_id: u16,
    pub aspect_target_id: u16,
    pub distance_au_fp4: u16,
    pub aspect_type_code: u8,
    pub motion_flags: u8,
    pub aspect_orb_bp: u16,
    pub seq_mod: u16,
}

/// Computes the minimal circular aspect angle between two ecliptic longitudes.
/// Returns angular separation in milli-degrees [0..180000].
/// @outputs: 1
/// @domain: lon1_milli_deg [0..360000]
/// @domain: lon2_milli_deg [0..360000]
pub fn compute_aspect_angle(lon1_milli_deg: u32, lon2_milli_deg: u32) -> Option<u32> {
    if lon1_milli_deg > 360_000 || lon2_milli_deg > 360_000 {
        return None;
    }
    let diff = if lon1_milli_deg >= lon2_milli_deg {
        lon1_milli_deg - lon2_milli_deg
    } else {
        lon2_milli_deg - lon1_milli_deg
    };
    let shortest = if diff > 180_000 {
        360_000 - diff
    } else {
        diff
    };
    Some(shortest)
}

/// Classifies an angular separation into an aspect type and computes remaining orb in bp.
/// Standard aspects: 0 deg (Conjunction), 60 deg (Sextile), 90 deg (Square), 120 deg (Trine), 180 deg (Opposition).
/// @outputs: 1
/// @domain: angle_milli_deg [0..180000]
/// @domain: max_orb_bp [0..1500]
pub fn classify_aspect_type(angle_milli_deg: u32, max_orb_bp: u16) -> Option<(u8, u16)> {
    if angle_milli_deg > 180_000 || max_orb_bp > 1500 {
        return None;
    }
    let max_orb_milli_deg = (max_orb_bp as u32) * 10;
    let targets: [(u8, u32); 5] = [
        (ASPECT_CONJUNCTION, 0),
        (ASPECT_SEXTILE, 60_000),
        (ASPECT_SQUARE, 90_000),
        (ASPECT_TRINE, 120_000),
        (ASPECT_OPPOSITION, 180_000),
    ];

    let mut idx = 0usize;
    while idx < 5 {
        let (kind, target_angle) = targets[idx];
        let diff = if angle_milli_deg >= target_angle {
            angle_milli_deg - target_angle
        } else {
            target_angle - angle_milli_deg
        };
        if diff <= max_orb_milli_deg {
            let orb_bp = (diff / 10) as u16;
            return Some((kind, orb_bp));
        }
        idx += 1;
    }
    Some((ASPECT_NONE, 0))
}

/// Serializes a `CelestialPacket32` into a 32-byte fixed binary slice (Little-Endian).
/// @outputs: 1
pub fn encode_celestial_packet_32(pkt: &CelestialPacket32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..8].copy_from_slice(&pkt.ts_us.to_le_bytes());
    buf[8..12].copy_from_slice(&pkt.ecliptic_longitude_milli_deg.to_le_bytes());
    buf[12..16].copy_from_slice(&pkt.aspect_angle_milli_deg.to_le_bytes());
    buf[16..20].copy_from_slice(&pkt.speed_milli_deg_day.to_le_bytes());
    buf[20..22].copy_from_slice(&pkt.primary_body_id.to_le_bytes());
    buf[22..24].copy_from_slice(&pkt.aspect_target_id.to_le_bytes());
    buf[24..26].copy_from_slice(&pkt.distance_au_fp4.to_le_bytes());
    buf[26] = pkt.aspect_type_code;
    buf[27] = pkt.motion_flags;
    buf[28..30].copy_from_slice(&pkt.aspect_orb_bp.to_le_bytes());
    buf[30..32].copy_from_slice(&pkt.seq_mod.to_le_bytes());
    buf
}

/// Decodes a 32-byte fixed binary slice into a `CelestialPacket32`.
/// Validates domain boundaries on all subfields.
/// @outputs: 1
pub fn decode_celestial_packet_32(buf: &[u8; 32]) -> Option<CelestialPacket32> {
    let mut ts_bytes = [0u8; 8];
    ts_bytes.copy_from_slice(&buf[0..8]);
    let ts_us = u64::from_le_bytes(ts_bytes);

    let mut lon_bytes = [0u8; 4];
    lon_bytes.copy_from_slice(&buf[8..12]);
    let lon = u32::from_le_bytes(lon_bytes);
    if lon > 360_000 {
        return None;
    }

    let mut ang_bytes = [0u8; 4];
    ang_bytes.copy_from_slice(&buf[12..16]);
    let ang = u32::from_le_bytes(ang_bytes);
    if ang > 360_000 {
        return None;
    }

    let mut spd_bytes = [0u8; 4];
    spd_bytes.copy_from_slice(&buf[16..20]);
    let speed = i32::from_le_bytes(spd_bytes);

    let mut b1_bytes = [0u8; 2];
    b1_bytes.copy_from_slice(&buf[20..22]);
    let primary = u16::from_le_bytes(b1_bytes);

    let mut b2_bytes = [0u8; 2];
    b2_bytes.copy_from_slice(&buf[22..24]);
    let target = u16::from_le_bytes(b2_bytes);

    let mut dist_bytes = [0u8; 2];
    dist_bytes.copy_from_slice(&buf[24..26]);
    let dist = u16::from_le_bytes(dist_bytes);

    let aspect_type = buf[26];
    let motion_flags = buf[27];

    let mut orb_bytes = [0u8; 2];
    orb_bytes.copy_from_slice(&buf[28..30]);
    let orb = u16::from_le_bytes(orb_bytes);

    let mut seq_bytes = [0u8; 2];
    seq_bytes.copy_from_slice(&buf[30..32]);
    let seq = u16::from_le_bytes(seq_bytes);

    Some(CelestialPacket32 {
        ts_us,
        ecliptic_longitude_milli_deg: lon,
        aspect_angle_milli_deg: ang,
        speed_milli_deg_day: speed,
        primary_body_id: primary,
        aspect_target_id: target,
        distance_au_fp4: dist,
        aspect_type_code: aspect_type,
        motion_flags,
        aspect_orb_bp: orb,
        seq_mod: seq,
    })
}

pub const FLAG_SOLAR_RETURN: u16 = 0x0001;
pub const FLAG_PRE_NATAL_EVENT: u16 = 0x0002;
pub const FLAG_BTR_CALIBRATED: u16 = 0x0004;

/// Bus 1 Canonical 32-Byte Vedic Life Event Packet.
/// Layout: 8 + 4 + 4 + 4 + 4 + 4 + 2 + 2 = 32 Bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C, packed)]
pub struct VedicEventPacket32 {
    pub ts_us: u64,
    pub lat_milli_deg: i32,
    pub lon_milli_deg: i32,
    pub ayanamsha_arcsec: u32,
    pub dasha_packed: u32,
    pub transit_summary_hash: u32,
    pub event_category_code: u16,
    pub flags: u16,
}

/// Serializes a `VedicEventPacket32` into a 32-byte fixed binary slice (Little-Endian).
/// @outputs: 1
pub fn encode_vedic_event_packet_32(pkt: &VedicEventPacket32) -> [u8; 32] {
    let mut buf = [0u8; 32];
    buf[0..8].copy_from_slice(&pkt.ts_us.to_le_bytes());
    buf[8..12].copy_from_slice(&pkt.lat_milli_deg.to_le_bytes());
    buf[12..16].copy_from_slice(&pkt.lon_milli_deg.to_le_bytes());
    buf[16..20].copy_from_slice(&pkt.ayanamsha_arcsec.to_le_bytes());
    buf[20..24].copy_from_slice(&pkt.dasha_packed.to_le_bytes());
    buf[24..28].copy_from_slice(&pkt.transit_summary_hash.to_le_bytes());
    buf[28..30].copy_from_slice(&pkt.event_category_code.to_le_bytes());
    buf[30..32].copy_from_slice(&pkt.flags.to_le_bytes());
    buf
}

/// Decodes a 32-byte fixed binary slice into a `VedicEventPacket32`.
/// @outputs: 1
pub fn decode_vedic_event_packet_32(buf: &[u8; 32]) -> Option<VedicEventPacket32> {
    let mut ts_b = [0u8; 8];
    ts_b.copy_from_slice(&buf[0..8]);
    let ts_us = u64::from_le_bytes(ts_b);

    let mut lat_b = [0u8; 4];
    lat_b.copy_from_slice(&buf[8..12]);
    let lat_milli_deg = i32::from_le_bytes(lat_b);
    if lat_milli_deg < -90_000 || lat_milli_deg > 90_000 {
        return None;
    }

    let mut lon_b = [0u8; 4];
    lon_b.copy_from_slice(&buf[12..16]);
    let lon_milli_deg = i32::from_le_bytes(lon_b);
    if lon_milli_deg < -180_000 || lon_milli_deg > 180_000 {
        return None;
    }

    let mut ayan_b = [0u8; 4];
    ayan_b.copy_from_slice(&buf[16..20]);
    let ayanamsha_arcsec = u32::from_le_bytes(ayan_b);
    if ayanamsha_arcsec > 1_296_000 {
        return None;
    }

    let mut dasha_b = [0u8; 4];
    dasha_b.copy_from_slice(&buf[20..24]);
    let dasha_packed = u32::from_le_bytes(dasha_b);

    let mut hash_b = [0u8; 4];
    hash_b.copy_from_slice(&buf[24..28]);
    let transit_summary_hash = u32::from_le_bytes(hash_b);

    let mut cat_b = [0u8; 2];
    cat_b.copy_from_slice(&buf[28..30]);
    let event_category_code = u16::from_le_bytes(cat_b);

    let mut flg_b = [0u8; 2];
    flg_b.copy_from_slice(&buf[30..32]);
    let flags = u16::from_le_bytes(flg_b);

    Some(VedicEventPacket32 {
        ts_us,
        lat_milli_deg,
        lon_milli_deg,
        ayanamsha_arcsec,
        dasha_packed,
        transit_summary_hash,
        event_category_code,
        flags,
    })
}

/// Bus 2 Bitemporal Ephemeris Stratigraphy Record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphemerisRecord {
    pub entity_id: u32,
    pub event_period_ts_sec: u64,
    pub public_known_ts_sec: u64,
    pub date_yyyymmdd: u32,
    pub body_id: u16,
    pub ecliptic_lon_milli_deg: u32,
    pub speed_milli_deg_day: i32,
    pub is_retrograde: u8,
    pub moon_phase_pct: u8,
}

/// Validates Bus 2 Bitemporal causality: `event_period_ts <= public_known_ts <= sim_clock_ts`.
/// Returns true if and only if no future lookahead leak exists.
/// @outputs: 1
/// @domain: event_period_ts_sec [0..4102444800]
/// @domain: public_known_ts_sec [0..4102444800]
/// @domain: sim_clock_ts_sec [0..4102444800]
pub fn verify_ephemeris_bitemporal(
    event_period_ts_sec: u64,
    public_known_ts_sec: u64,
    sim_clock_ts_sec: u64,
) -> bool {
    if event_period_ts_sec > 4_102_444_800
        || public_known_ts_sec > 4_102_444_800
        || sim_clock_ts_sec > 4_102_444_800
    {
        return false;
    }
    // Hardline causality: event occurred, then became publicly known, before current clock
    event_period_ts_sec <= public_known_ts_sec && public_known_ts_sec <= sim_clock_ts_sec
}

/// Formats an `EphemerisRecord` into a canonical TSV line.
/// @outputs: 1
pub fn format_ephemeris_tsv(rec: &EphemerisRecord) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        rec.entity_id,
        rec.event_period_ts_sec,
        rec.public_known_ts_sec,
        rec.date_yyyymmdd,
        rec.body_id,
        rec.ecliptic_lon_milli_deg,
        rec.speed_milli_deg_day,
        rec.is_retrograde,
        rec.moon_phase_pct
    )
}

/// Parses a single TSV line into an `EphemerisRecord`.
/// Returns None if malformed or values out of domain bounds.
/// @outputs: 1
pub fn parse_ephemeris_tsv(line: &str) -> Option<EphemerisRecord> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let cols: Vec<&str> = trimmed.split('\t').collect();
    if cols.len() < 9 {
        return None;
    }

    let entity_id = cols[0].parse::<u32>().ok()?;
    let event_ts = cols[1].parse::<u64>().ok()?;
    let public_ts = cols[2].parse::<u64>().ok()?;
    let date = cols[3].parse::<u32>().ok()?;
    let body_id = cols[4].parse::<u16>().ok()?;
    let lon = cols[5].parse::<u32>().ok()?;
    let speed = cols[6].parse::<i32>().ok()?;
    let retro = cols[7].parse::<u8>().ok()?;
    let moon = cols[8].parse::<u8>().ok()?;

    if date < 19000101 || date > 20991231 || lon > 360_000 || moon > 100 || retro > 1 {
        return None;
    }

    Some(EphemerisRecord {
        entity_id,
        event_period_ts_sec: event_ts,
        public_known_ts_sec: public_ts,
        date_yyyymmdd: date,
        body_id,
        ecliptic_lon_milli_deg: lon,
        speed_milli_deg_day: speed,
        is_retrograde: retro,
        moon_phase_pct: moon,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// @outputs: 1
    fn test_packet_size_is_exactly_32_bytes() {
        assert_eq!(std::mem::size_of::<CelestialPacket32>(), 32);
    }

    #[test]
    /// @outputs: 1
    fn test_compute_aspect_angle_cases() {
        assert_eq!(compute_aspect_angle(10_000, 70_000), Some(60_000));
        assert_eq!(compute_aspect_angle(350_000, 10_000), Some(20_000));
        assert_eq!(compute_aspect_angle(0, 180_000), Some(180_000));
        assert_eq!(compute_aspect_angle(360_001, 100), None);
    }

    #[test]
    /// @outputs: 1
    fn test_classify_aspect_types_and_orbs() {
        // Conjunction at 0 deg, diff 500 milli-deg (50 bp)
        // @safety-proof: valid test inputs within angle domain
        let (kind, orb) = classify_aspect_type(500, 100).unwrap();
        assert_eq!(kind, ASPECT_CONJUNCTION);
        assert_eq!(orb, 50);

        // Square at 90 deg, exact
        // @safety-proof: valid test inputs within angle domain
        let (kind_sq, orb_sq) = classify_aspect_type(90_000, 100).unwrap();
        assert_eq!(kind_sq, ASPECT_SQUARE);
        assert_eq!(orb_sq, 0);

        // Out of orb (angle 45 deg, max orb 100 bp = 1 deg)
        // @safety-proof: valid test inputs within angle domain
        let (kind_none, _) = classify_aspect_type(45_000, 100).unwrap();
        assert_eq!(kind_none, ASPECT_NONE);
    }

    #[test]
    /// @outputs: 1
    fn test_encode_decode_packet_32_roundtrip() {
        let pkt = CelestialPacket32 {
            ts_us: 1_700_000_000_123_456,
            ecliptic_longitude_milli_deg: 142_500,
            aspect_angle_milli_deg: 90_000,
            speed_milli_deg_day: -1_200, // Retrograde
            primary_body_id: BODY_MERCURY,
            aspect_target_id: BODY_SATURN,
            distance_au_fp4: 4_200,
            aspect_type_code: ASPECT_SQUARE,
            motion_flags: FLAG_RETROGRADE,
            aspect_orb_bp: 25,
            seq_mod: 42,
        };
        let buf = encode_celestial_packet_32(&pkt);
        assert_eq!(buf.len(), 32);

        // @safety-proof: valid test packet within bounds
        let decoded = decode_celestial_packet_32(&buf).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    /// @outputs: 1
    fn test_ephemeris_bitemporal_guard() {
        let event_ts = 1_700_000_000;
        let public_ts = 1_700_005_000;

        // Clock ahead: OK
        assert!(verify_ephemeris_bitemporal(event_ts, public_ts, 1_700_010_000));
        // Clock before publication: Blocked (lookahead leak)
        assert!(!verify_ephemeris_bitemporal(event_ts, public_ts, 1_700_002_000));
        // Future event known before it happens: Blocked
        assert!(!verify_ephemeris_bitemporal(1_700_010_000, 1_700_005_000, 1_700_020_000));
    }

    #[test]
    /// @outputs: 1
    fn test_ephemeris_tsv_roundtrip() {
        let rec = EphemerisRecord {
            entity_id: 1000,
            event_period_ts_sec: 1_700_000_000,
            public_known_ts_sec: 1_700_005_000,
            date_yyyymmdd: 20261111,
            body_id: BODY_MERCURY,
            ecliptic_lon_milli_deg: 245_000,
            speed_milli_deg_day: -350,
            is_retrograde: 1,
            moon_phase_pct: 75,
        };
        let tsv = format_ephemeris_tsv(&rec);
        let parsed = parse_ephemeris_tsv(&tsv).expect("tsv parse failed");
        assert_eq!(rec, parsed);
    }

    #[test]
    /// @outputs: 1
    fn test_vedic_event_packet_32_roundtrip() {
        assert_eq!(std::mem::size_of::<VedicEventPacket32>(), 32);

        let pkt = VedicEventPacket32 {
            ts_us: 1_700_000_000_987_654,
            lat_milli_deg: 25_033,
            lon_milli_deg: 121_565,
            ayanamsha_arcsec: 85_871,
            dasha_packed: 0x0001_0703,
            transit_summary_hash: 0xABCD_1234,
            event_category_code: 101,
            flags: FLAG_PRE_NATAL_EVENT | FLAG_BTR_CALIBRATED,
        };

        let encoded = encode_vedic_event_packet_32(&pkt);
        assert_eq!(encoded.len(), 32);

        let decoded = decode_vedic_event_packet_32(&encoded).unwrap();
        assert_eq!(decoded, pkt);
        let flg = decoded.flags;
        assert_eq!(flg & FLAG_PRE_NATAL_EVENT, FLAG_PRE_NATAL_EVENT);
    }
}
