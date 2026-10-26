// ## Map
// Role: core/ble_adv_parser — 低功耗藍芽 (BLE 4.2 / 5.0) 廣播封包 AD Structure 零拷貝解析微積木。
// Inputs: 原始廣播資料切片 &[u8]、目標 AD Type 代碼。
// Outputs: 零拷貝借用欄位切片 Option<&[u8]>、發射功率 Option<i8>、設備名稱 Option<&str>、廠商自定義數據 Option<(u16, &[u8])>。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、畸形長度防越界安全守衛。

/// @outputs: 2
/// @domain: adv_len [1..255]
pub fn ble_adv_find_ad_type<'a>(raw_packet: &'a [u8], target_ad_type: u8) -> Option<&'a [u8]> {
    if raw_packet.is_empty() || raw_packet.len() > 255 {
        return None;
    }
    let mut cursor = 0usize;
    while cursor < raw_packet.len() {
        let block_len = raw_packet[cursor] as usize;
        if block_len == 0 {
            break;
        }
        let total_block = block_len + 1;
        if cursor + total_block > raw_packet.len() {
            break;
        }
        let ad_type = raw_packet[cursor + 1];
        let val_slice = &raw_packet[cursor + 2..cursor + total_block];
        if ad_type == target_ad_type {
            return Some(val_slice);
        }
        cursor += total_block;
    }
    None
}

/// @outputs: 2
/// @domain: adv_len [1..255]
pub fn ble_adv_parse_tx_power(raw_packet: &[u8]) -> Option<i8> {
    if raw_packet.is_empty() || raw_packet.len() > 255 {
        return None;
    }
    // AD Type 0x0A: TX Power Level (1 byte signed integer)
    let slice = ble_adv_find_ad_type(raw_packet, 0x0A)?;
    if slice.len() != 1 {
        return None;
    }
    Some(slice[0] as i8)
}

/// @outputs: 2
/// @domain: adv_len [1..255]
pub fn ble_adv_parse_device_name<'a>(raw_packet: &'a [u8]) -> Option<&'a str> {
    if raw_packet.is_empty() || raw_packet.len() > 255 {
        return None;
    }
    // 0x09: Complete Local Name, 0x08: Shortened Local Name
    let name_bytes = ble_adv_find_ad_type(raw_packet, 0x09)
        .or_else(|| ble_adv_find_ad_type(raw_packet, 0x08))?;
    std::str::from_utf8(name_bytes).ok()
}

/// @outputs: 2
/// @domain: adv_len [1..255]
pub fn ble_adv_parse_manufacturer_data<'a>(raw_packet: &'a [u8]) -> Option<(u16, &'a [u8])> {
    if raw_packet.is_empty() || raw_packet.len() > 255 {
        return None;
    }
    // AD Type 0xFF: Manufacturer Specific Data
    let slice = ble_adv_find_ad_type(raw_packet, 0xFF)?;
    if slice.len() < 2 {
        return None;
    }
    let company_id = u16::from_le_bytes([slice[0], slice[1]]);
    let payload = &slice[2..];
    Some((company_id, payload))
}
