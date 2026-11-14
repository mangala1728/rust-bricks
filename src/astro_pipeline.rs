// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Pure Rust Canonical Domain Pipe & 32-Byte Packet Stream Protocol Engine
//! - Inputs:
//!   - High-speed stream packets (VedicEventPacket32, CelestialPacket32)
//!   - Microsecond / second timestamps (supports signed pre-1970 UTC epochs)
//! - Outputs:
//!   - Fixed 32-Byte little-endian binary buffers
//!   - Validated domain packet structs
//! - Invariants:
//!   1. Exactly 32 bytes wire layout (std::mem::size_of::<T>() == 32).
//!   2. Little-endian serialization, zero dynamic allocation, zero pointers.
//!   3. Bitemporal causality & anti-lookahead invariant enforcement.

/// Event flags for VedicEventPacket32
pub const FLAG_SOLAR_RETURN: u16 = 0x0001;
pub const FLAG_PRE_NATAL_EVENT: u16 = 0x0002;
pub const FLAG_BTR_CALIBRATED: u16 = 0x0004;
pub const FLAG_LUNAR_RETURN: u16 = 0x0008;

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
/// Validates domain boundaries on all subfields.
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

/// Validates Bus 2 Bitemporal causality with signed epoch support.
/// Invariant: event_period_ts <= public_known_ts <= sim_clock_ts.
/// @outputs: 1
/// @domain: event_ts [-62135596800..=253402300799]
/// @domain: public_ts [-62135596800..=253402300799]
/// @domain: sim_ts [-62135596800..=253402300799]
pub fn verify_signed_bitemporal(event_ts: i64, public_ts: i64, sim_ts: i64) -> bool {
    if event_ts < -62_135_596_800 || event_ts > 253_402_300_799 {
        return false;
    }
    if public_ts < -62_135_596_800 || public_ts > 253_402_300_799 {
        return false;
    }
    if sim_ts < -62_135_596_800 || sim_ts > 253_402_300_799 {
        return false;
    }
    event_ts <= public_ts && public_ts <= sim_ts
}

/// Bus 1 Canonical 32-Byte Celestial Event Packet (`<QIIihhHBBHH`).
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

/// Decodes a 32-byte fixed binary slice into a `CelestialPacket32` with domain guards.
/// @outputs: 2
pub fn decode_celestial_packet_32(buf: &[u8; 32]) -> Option<CelestialPacket32> {
    let mut ts_b = [0u8; 8];
    ts_b.copy_from_slice(&buf[0..8]);
    let ts_us = u64::from_le_bytes(ts_b);

    let mut lon_b = [0u8; 4];
    lon_b.copy_from_slice(&buf[8..12]);
    let lon = u32::from_le_bytes(lon_b);
    if lon > 360_000 {
        return None;
    }

    let mut ang_b = [0u8; 4];
    ang_b.copy_from_slice(&buf[12..16]);
    let ang = u32::from_le_bytes(ang_b);
    if ang > 360_000 {
        return None;
    }

    let mut spd_b = [0u8; 4];
    spd_b.copy_from_slice(&buf[16..20]);
    let speed = i32::from_le_bytes(spd_b);

    let mut b1_b = [0u8; 2];
    b1_b.copy_from_slice(&buf[20..22]);
    let primary = u16::from_le_bytes(b1_b);

    let mut b2_b = [0u8; 2];
    b2_b.copy_from_slice(&buf[22..24]);
    let target = u16::from_le_bytes(b2_b);

    let mut dist_b = [0u8; 2];
    dist_b.copy_from_slice(&buf[24..26]);
    let dist = u16::from_le_bytes(dist_b);

    let mut orb_b = [0u8; 2];
    orb_b.copy_from_slice(&buf[28..30]);
    let orb = u16::from_le_bytes(orb_b);

    let mut seq_b = [0u8; 2];
    seq_b.copy_from_slice(&buf[30..32]);
    let seq = u16::from_le_bytes(seq_b);

    Some(CelestialPacket32 {
        ts_us,
        ecliptic_longitude_milli_deg: lon,
        aspect_angle_milli_deg: ang,
        speed_milli_deg_day: speed,
        primary_body_id: primary,
        aspect_target_id: target,
        distance_au_fp4: dist,
        aspect_type_code: buf[26],
        motion_flags: buf[27],
        aspect_orb_bp: orb,
        seq_mod: seq,
    })
}

/// Validates the schema keys of `02_UNITS_AND_PRECISION_SPEC.json` (parity with `test_spec_json_validity` in `test_astro_pipeline.py`).
/// @outputs: 2
pub fn validate_units_precision_spec_json(json_text: &str) -> bool {
    if json_text.is_empty() || json_text.len() > 1_000_000 {
        return false;
    }
    let has_title = json_text.contains("\"UniversalDomainUnitAndPrecisionSpecification\"");
    let has_spatial = json_text.contains("\"spatial_and_angular\"") && json_text.contains("\"angle_milli_deg\"");
    let has_vedic = json_text.contains("\"vedic_identities\"") && json_text.contains("\"vedic_tithi_id\"");
    let has_frames = json_text.contains("\"spatiotemporal_frames\"") && json_text.contains("\"frame_id\"");
    has_title && has_spatial && has_vedic && has_frames
}

