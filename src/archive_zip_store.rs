// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Zero-dependency IEEE 802.3 CRC-32, ZIP (STORED method 0) archive builder & uncompressed PNG encoder (`core` brick)
//! - Inputs: Named byte payloads (`ZipEntry`) or RGBA image dimensions and pixels
//! - Outputs: Valid PKZIP archive bytes or PNG file bytes
//! - Invariants:
//!   1. Zero external crates (`std` only) and zero `static mut` (`R09-2`).
//!   2. Zero recursion (`R09-7`), bounded loops (`R09-8`), nesting depth <= 2 (`R09-3`).
//!   3. Explicit `@domain` bounds and runtime guards on all numeric parameters (`R09-4`).

const MAX_ZIP_ENTRIES: usize = 4096;
const MAX_PNG_DIM: u32 = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// Computes IEEE 802.3 CRC-32 checksum over `data` without lookup tables (`static mut = 0`).
// @outputs: 1
pub fn zip_crc32_ieee(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320u32 & mask);
        }
    }
    !crc
}

/// Computes Adler-32 checksum used by zlib streams inside PNG `IDAT`.
// @outputs: 1
pub fn zip_adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

/// Appends a single STORED local file header + payload and records central-directory metadata.
// @outputs: 1
// @domain: local_offset [0..2147483647]
fn write_zip_local_entry(
    entry: &ZipEntry,
    local_offset: u32,
    out: &mut Vec<u8>,
    central: &mut Vec<u8>,
) {
    if local_offset > 2_147_483_647 {
        return;
    }
    let crc = zip_crc32_ieee(&entry.data);
    let size = entry.data.len() as u32;
    let name_bytes = entry.name.as_bytes();
    let nlen = name_bytes.len().min(65535) as u16;

    out.extend_from_slice(&0x0403_4B50u32.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&size.to_le_bytes());
    out.extend_from_slice(&nlen.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&name_bytes[..nlen as usize]);
    out.extend_from_slice(&entry.data);

    central.extend_from_slice(&0x0201_4B50u32.to_le_bytes());
    central.extend_from_slice(&20u16.to_le_bytes());
    central.extend_from_slice(&20u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&crc.to_le_bytes());
    central.extend_from_slice(&size.to_le_bytes());
    central.extend_from_slice(&size.to_le_bytes());
    central.extend_from_slice(&nlen.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u16.to_le_bytes());
    central.extend_from_slice(&0u32.to_le_bytes());
    central.extend_from_slice(&local_offset.to_le_bytes());
    central.extend_from_slice(&name_bytes[..nlen as usize]);
}

/// Builds a valid PKZIP archive (STORED method 0) from `entries`.
// @outputs: 1
pub fn build_zip_store(entries: &[ZipEntry]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut central = Vec::new();
    let count = entries.len().min(MAX_ZIP_ENTRIES);
    for entry in &entries[..count] {
        let offset = out.len() as u32;
        write_zip_local_entry(entry, offset, &mut out, &mut central);
    }
    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4B50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(count as u16).to_le_bytes());
    out.extend_from_slice(&(count as u16).to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

/// Appends one PNG chunk (`length + tag + data + crc32`) to `out`.
// @outputs: 1
fn push_png_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let crc_start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let crc = zip_crc32_ieee(&out[crc_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// Encodes an RGBA pixel buffer into a valid uncompressed PNG byte vector.
// @domain: width [1..4096]
// @domain: height [1..4096]
// @outputs: 2
pub fn encode_minimal_png_rgba(width: u32, height: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    if width == 0 || width > MAX_PNG_DIM {
        return Err("PNG width out of range".to_string());
    }
    if height == 0 || height > MAX_PNG_DIM {
        return Err("PNG height out of range".to_string());
    }
    let expected = (width as usize) * (height as usize) * 4;
    if rgba.len() != expected {
        return Err("RGBA buffer length mismatch".to_string());
    }

    let row_bytes = (width as usize) * 4;
    let mut raw = Vec::with_capacity((row_bytes + 1) * (height as usize));
    for y in 0..(height as usize) {
        raw.push(0u8);
        let start = y * row_bytes;
        raw.extend_from_slice(&rgba[start..start + row_bytes]);
    }

    let mut zlib = Vec::with_capacity(raw.len() + 16);
    zlib.push(0x78);
    zlib.push(0x01);
    let mut off = 0usize;
    while off < raw.len() {
        let chunk_len = (raw.len() - off).min(65535);
        let is_final = if off + chunk_len >= raw.len() { 1u8 } else { 0u8 };
        zlib.push(is_final);
        let len_u16 = chunk_len as u16;
        zlib.extend_from_slice(&len_u16.to_le_bytes());
        zlib.extend_from_slice(&(!len_u16).to_le_bytes());
        zlib.extend_from_slice(&raw[off..off + chunk_len]);
        off += chunk_len;
    }
    zlib.extend_from_slice(&zip_adler32(&raw).to_be_bytes());

    let mut png = Vec::with_capacity(zlib.len() + 64);
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    push_png_chunk(&mut png, b"IHDR", &ihdr);
    push_png_chunk(&mut png, b"IDAT", &zlib);
    push_png_chunk(&mut png, b"IEND", &[]);
    Ok(png)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // @outputs: 1
    fn test_crc32_ieee_known_vector() {
        assert_eq!(zip_crc32_ieee(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    // @outputs: 1
    fn test_zip_and_png_headers() {
        let zip = build_zip_store(&[ZipEntry {
            name: "hello.txt".to_string(),
            data: b"world".to_vec(),
        }]);
        assert!(zip.starts_with(b"PK\x03\x04"));
        assert!(zip.windows(4).any(|w| w == b"PK\x05\x06"));

        // @safety-proof: valid 1x1 RGBA buffer
        let png = encode_minimal_png_rgba(1, 1, &[255, 0, 0, 255]).unwrap();
        assert!(png.starts_with(&[137, 80, 78, 71, 13, 10, 26, 10]));
        assert!(png.windows(4).any(|w| w == b"IEND"));
    }
}
