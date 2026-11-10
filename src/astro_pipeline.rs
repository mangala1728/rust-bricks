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
    event_ts <= public_ts && public_ts <= sim_ts
}
