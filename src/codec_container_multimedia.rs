//! # 容器格式、封裝流與多媒體元數據解析 (Container Formats & Multimedia Packaging Streams)
//!
//! 提供 MP4/ISOBMFF Box 標頭、WebM/MKV EBML VINT、MPEG-TS 188位元組訊框與 PCR 時間戳、
//! FLV Tag 封裝、ID3v2 28位元 Syncsafe 整數編解碼與 OggS 頁頭 CRC32 微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: ISOBMFF / MP4 Atom Box 標頭解析 (Box Header Parse)
/// 回傳 (Box 大小, 4位元組 FourCC 標識, 標頭長度 8 或 16)
/// @outputs: 1
/// @domain: bytes.len() >= 8 && bytes.len() <= 4096
pub fn isobmff_box_header_parse(bytes: &[u8]) -> Option<(u32, [u8; 4], usize)> {
    if bytes.len() < 8 || bytes.len() > 4096 {
        return None;
    }
    let size32 = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    let fourcc = [bytes[4], bytes[5], bytes[6], bytes[7]];
    if size32 == 1 {
        // 64-bit LargeSize 標頭 (需要額外 8 位元組)
        if bytes.len() < 16 {
            return None;
        }
        let size64 = u64::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        let capped_size = if size64 > u32::MAX as u64 { u32::MAX } else { size64 as u32 };
        Some((capped_size, fourcc, 16))
    } else {
        Some((size32, fourcc, 8))
    }
}

/// ## Calc: WebM / Matroska (MKV) EBML 可變長度整數解碼 (EBML VINT Decode)
/// 回傳 (解碼數值, 消耗位元組數 1..=8)
/// @outputs: 1
/// @domain: bytes.len() > 0 && bytes.len() <= 64
pub fn ebml_vint_decode(bytes: &[u8]) -> Option<(u64, usize)> {
    if bytes.is_empty() || bytes.len() > 64 {
        return None;
    }
    let lead = bytes[0];
    if lead == 0 {
        return None;
    }
    let len = lead.leading_zeros() as usize + 1;
    if len > 8 || bytes.len() < len {
        return None;
    }
    let mask = 0xFF >> len;
    let mut val = (lead & mask) as u64;
    for &b in &bytes[1..len] {
        val = (val << 8) | (b as u64);
    }
    Some((val, len))
}

/// ## Step: MPEG-TS 188 位元組封包同步與標頭欄位驗證 (MPEG-TS Packet Validate)
/// 回傳 (PID 13-bit, PayloadUnitStart, AdaptationFieldCtrl 2-bit, ContinuityCounter 4-bit)
/// @outputs: 1
/// @domain: packet.len() == 188
pub fn mpegts_sync_packet_validate(packet: &[u8]) -> Option<(u16, bool, u8, u8)> {
    if packet.len() != 188 {
        return None;
    }
    // 同步字節 0x47
    if packet[0] != 0x47 {
        return None;
    }
    let tei = (packet[1] & 0x80) != 0;
    if tei {
        return None; // 傳輸錯誤指標
    }
    let pusi = (packet[1] & 0x40) != 0;
    let pid = (((packet[1] & 0x1F) as u16) << 8) | (packet[2] as u16);
    let afc = (packet[3] >> 4) & 0x03;
    let cc = packet[3] & 0x0F;
    Some((pid, pusi, afc, cc))
}

/// ## Calc: MPEG-TS PCR (節目時鐘參考) 27MHz 轉納秒時間戳 (PCR to Nanoseconds)
/// @outputs: 1
/// @domain: pcr_base <= 0x1_FFFF_FFFF && pcr_ext < 300
pub fn mpegts_pcr_to_timestamp_ns(pcr_base: u64, pcr_ext: u16) -> Option<u64> {
    if pcr_base > 0x1_FFFF_FFFF || pcr_ext >= 300 {
        return None;
    }
    // PCR = pcr_base * 300 + pcr_ext (單位: 1/27,000,000 秒)
    let pcr_27mhz = pcr_base.checked_mul(300)?.checked_add(pcr_ext as u64)?;
    // 奈秒 = pcr_27mhz * 1_000_000_000 / 27_000_000 = pcr_27mhz * 1000 / 27
    let ns = (pcr_27mhz as u128).checked_mul(1000)? / 27;
    if ns > u64::MAX as u128 {
        None
    } else {
        Some(ns as u64)
    }
}

/// ## Step: FLV (Flash Video) 11位元組 Tag 標頭解碼 (FLV Tag Header Parse)
/// 回傳 (Tag類型, 數據大小, 毫秒時間戳)
/// @outputs: 1
/// @domain: header.len() == 11
pub fn flv_tag_header_parse(header: &[u8; 11]) -> Option<(u8, u32, u32)> {
    let tag_type = header[0];
    // 8: Audio, 9: Video, 18: Script
    if !matches!(tag_type, 8 | 9 | 18) {
        return None;
    }
    let data_size = ((header[1] as u32) << 16) | ((header[2] as u32) << 8) | (header[3] as u32);
    // 時間戳: [4..6] 為低 24-bit，[7] 為高 8-bit (擴展位元組)
    let ts_low = ((header[4] as u32) << 16) | ((header[5] as u32) << 8) | (header[6] as u32);
    let ts_high = (header[7] as u32) << 24;
    let timestamp_ms = ts_high | ts_low;
    Some((tag_type, data_size, timestamp_ms))
}

/// ## Calc: ID3v2 28 位元 Syncsafe 整數解碼 (ID3v2 Syncsafe Decode)
/// @outputs: 1
/// @domain: 每個 byte 最高位 MSB 必須為 0
pub fn id3v2_syncsafe_to_u32(bytes: [u8; 4]) -> Option<u32> {
    if (bytes[0] & 0x80) != 0 || (bytes[1] & 0x80) != 0 || (bytes[2] & 0x80) != 0 || (bytes[3] & 0x80) != 0 {
        return None;
    }
    let val = ((bytes[0] as u32) << 21)
        | ((bytes[1] as u32) << 14)
        | ((bytes[2] as u32) << 7)
        | (bytes[3] as u32);
    Some(val)
}

/// ## Calc: 32 位元整數轉 ID3v2 28 位元 Syncsafe 編碼 (ID3v2 Syncsafe Encode)
/// @outputs: 1
/// @domain: val <= 0x0FFF_FFFF
pub fn id3v2_u32_to_syncsafe(val: u32) -> Option<[u8; 4]> {
    if val > 0x0FFF_FFFF {
        return None;
    }
    let b0 = ((val >> 21) & 0x7F) as u8;
    let b1 = ((val >> 14) & 0x7F) as u8;
    let b2 = ((val >> 7) & 0x7F) as u8;
    let b3 = (val & 0x7F) as u8;
    Some([b0, b1, b2, b3])
}

/// ## Step: Ogg 封裝容器頁面標頭校驗 (OggS Page Header Validate)
/// 回傳 (標頭類型標誌, 顆粒位置 GranulePos, 序列號, 頁序號)
/// @outputs: 1
/// @domain: bytes.len() >= 27 && bytes.len() <= 512
pub fn ogg_page_header_validate(bytes: &[u8]) -> Option<(u8, u64, u32, u32)> {
    if bytes.len() < 27 || bytes.len() > 512 {
        return None;
    }
    // "OggS" 捕獲圖元: 0x4F, 0x67, 0x67, 0x53
    if bytes[0] != 0x4F || bytes[1] != 0x67 || bytes[2] != 0x67 || bytes[3] != 0x53 {
        return None;
    }
    // 結構版本必須為 0
    if bytes[4] != 0x00 {
        return None;
    }
    let header_type = bytes[5];
    let granule = u64::from_le_bytes([
        bytes[6], bytes[7], bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13],
    ]);
    let serial = u32::from_le_bytes([bytes[14], bytes[15], bytes[16], bytes[17]]);
    let seq_no = u32::from_le_bytes([bytes[18], bytes[19], bytes[20], bytes[21]]);
    Some((header_type, granule, serial, seq_no))
}

/// ## Calc: Ogg 頁面 CRC-32 單字節累加步進 (Ogg CRC-32 Step)
/// @outputs: 1
/// @domain: crc <= 0xFFFF_FFFF
pub fn ogg_crc32_step(crc: u32, byte: u8) -> u32 {
    let mut reg = crc ^ ((byte as u32) << 24);
    for _ in 0..8 {
        if (reg & 0x8000_0000) != 0 {
            reg = (reg << 1) ^ 0x04C1_1DB7;
        } else {
            reg <<= 1;
        }
    }
    reg
}

/// ## Calc: 音訊取樣數換算毫秒時長 (Audio Samples to Duration in Milliseconds)
/// @outputs: 1
/// @domain: sample_rate_hz >= 1000 && sample_rate_hz <= 384_000
pub fn audio_samples_to_duration_ms(sample_count: u64, sample_rate_hz: u32) -> Option<u64> {
    if sample_rate_hz < 1000 || sample_rate_hz > 384_000 {
        return None;
    }
    let ms = (sample_count as u128).checked_mul(1000)? / (sample_rate_hz as u128);
    if ms > u64::MAX as u128 {
        None
    } else {
        Some(ms as u64)
    }
}
