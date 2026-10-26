//! # 智慧電表、樓宇自動化與儀表計量通訊協議 (Smart Meter & Building Automation Bus)
//!
//! 提供 BACnet MS/TP, M-Bus, DLMS/COSEM, DNP3, HART, KNX TP1 等
//! 智慧電表、水表、氣表與智慧樓宇物聯網協定的訊框校驗、長度解算與能量換算純微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: BACnet MS/TP 訊框標頭 CRC-8 計算
/// @outputs: 1
/// @domain: header_bytes.len() [1..=64]
pub fn bacnet_mstp_crc8(header_bytes: &[u8]) -> u8 {
    if header_bytes.is_empty() || header_bytes.len() > 64 {
        return 0;
    }
    let mut crc: u8 = 0xFF;
    for &b in header_bytes {
        let mut data = crc ^ b;
        for _ in 0..8 {
            if (data & 0x01) != 0 {
                data = (data >> 1) ^ 0x92;
            } else {
                data >>= 1;
            }
        }
        crc = data;
    }
    !crc
}

/// ## Calc: BACnet MS/TP 資料欄位 CRC-16 計算
/// @outputs: 1
/// @domain: data.len() [0..=1024]
pub fn bacnet_mstp_crc16(data: &[u8]) -> u16 {
    if data.len() > 1024 {
        return 0xFFFF;
    }
    let mut crc: u16 = 0xFFFF;
    for &b in data {
        let mut temp = (crc ^ (b as u16)) & 0x00FF;
        for _ in 0..8 {
            if (temp & 0x0001) != 0 {
                temp = (temp >> 1) ^ 0x8408;
            } else {
                temp >>= 1;
            }
        }
        crc = (crc >> 8) ^ temp;
    }
    !crc
}

/// ## Calc: M-Bus (EN 13757-2) 訊框累加校驗和 (Checksum)
/// @outputs: 1
/// @domain: payload.len() [1..=256]
pub fn mbus_frame_checksum(payload: &[u8]) -> Option<u8> {
    if payload.is_empty() || payload.len() > 256 {
        return None;
    }
    let mut sum: u32 = 0;
    for &b in payload {
        sum = sum.wrapping_add(b as u32);
    }
    Some((sum & 0xFF) as u8)
}

/// ## Step: M-Bus DIF (Data Information Field) 資料長度解碼
/// @outputs: 1
/// @domain: dif [0x00..=0xFF]
pub fn mbus_dif_data_len(dif: u8) -> usize {
    match dif & 0x0F {
        0x00 => 0, // 無資料 / 特殊功能
        0x01 => 1, // 8-bit 整數
        0x02 => 2, // 16-bit 整數
        0x03 => 3, // 24-bit 整數
        0x04 => 4, // 32-bit 整數 / 32-bit 浮點數
        0x05 => 4, // 32-bit 實數 (Real)
        0x06 => 6, // 48-bit 整數
        0x07 => 8, // 64-bit 整數
        _ => 0,
    }
}

/// ## Calc: DLMS/COSEM HDLC 16 位元 FCS (Frame Check Sequence)
/// @outputs: 1
/// @domain: bytes.len() [0..=512]
pub fn dlms_hdlc_fcs16(bytes: &[u8]) -> u16 {
    if bytes.len() > 512 {
        return 0xFFFF;
    }
    let mut fcs: u16 = 0xFFFF;
    for &b in bytes {
        let mut d = (fcs ^ (b as u16)) & 0x00FF;
        for _ in 0..8 {
            if (d & 0x0001) != 0 {
                d = (d >> 1) ^ 0x8408;
            } else {
                d >>= 1;
            }
        }
        fcs = (fcs >> 8) ^ d;
    }
    !fcs
}

/// ## Step: DLMS/COSEM OBIS 識別碼有效性檢驗
/// @outputs: 1
/// @domain: obis.len() == 6
pub fn dlms_obis_is_valid(obis: &[u8; 6]) -> bool {
    let medium_a = obis[0];
    let valid_a = matches!(medium_a, 0 | 1 | 4 | 5 | 6 | 7 | 8 | 9 | 15);
    valid_a
}

/// ## Calc: IEEE 1815 DNP3 協定 16 位元 CRC 計算
/// @outputs: 1
/// @domain: bytes.len() [0..=260]
pub fn dnp3_crc16(bytes: &[u8]) -> u16 {
    if bytes.len() > 260 {
        return 0;
    }
    let mut crc: u16 = 0x0000;
    for &b in bytes {
        let mut temp = (crc ^ (b as u16)) & 0x00FF;
        for _ in 0..8 {
            if (temp & 0x0001) != 0 {
                temp = (temp >> 1) ^ 0xA6BC;
            } else {
                temp >>= 1;
            }
        }
        crc = (crc >> 8) ^ temp;
    }
    !crc
}

/// ## Calc: HART 儀表通訊協定縱向冗餘檢查 (LRC)
/// @outputs: 1
/// @domain: bytes.len() [0..=256]
pub fn hart_lrc_checksum(bytes: &[u8]) -> u8 {
    if bytes.len() > 256 {
        return 0;
    }
    let mut lrc: u8 = 0;
    for &b in bytes {
        lrc ^= b;
    }
    lrc
}

/// ## Calc: KNX TP1 雙絞線訊框同位檢查和
/// @outputs: 1
/// @domain: bytes.len() [1..=64]
pub fn knx_tp1_checksum(bytes: &[u8]) -> u8 {
    if bytes.is_empty() || bytes.len() > 64 {
        return 0xFF;
    }
    let mut xor_acc: u8 = 0;
    for &b in bytes {
        xor_acc ^= b;
    }
    !xor_acc
}

/// ## Calc: 智慧電表脈衝轉累計電能 (kWh)
/// @outputs: 1
/// @domain: pulses_per_kwh [1..=100_000]
pub fn smart_meter_pulse_to_kwh(pulse_count: u32, pulses_per_kwh: u32) -> Option<f64> {
    if pulses_per_kwh == 0 || pulses_per_kwh > 100_000 {
        return None;
    }
    Some(pulse_count as f64 / pulses_per_kwh as f64)
}
