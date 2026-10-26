//! # IoT Wire Protocol Micro-Bricks (物聯網低功耗線路通訊協議微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 MQTT 3.1.1/5.0 變長整數編解碼與標頭、RFC 7252 CoAP 封包頭與 Option 增量編碼、
//! LoRaWAN MHDR 與 16-bit 訊框計數器解纏繞、Modbus FC 0x10 封裝、RFC 1055 SLIP 串列線路協議。

/// ## Map: 編碼 MQTT 3.1.1 / 5.0 可變長度剩餘長度 (Variable Byte Integer, 1..4 bytes)
/// @domain: length [0, 268435455]
/// @outputs: 1
pub fn mqtt_remaining_length_encode(length: u32, out_buf: &mut [u8]) -> usize {
    if length > 268_435_455 || out_buf.is_empty() {
        return 0;
    }
    let mut val = length;
    let mut idx = 0;
    loop {
        if idx >= out_buf.len() || idx >= 4 {
            return 0;
        }
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val > 0 {
            byte |= 0x80;
        }
        out_buf[idx] = byte;
        idx += 1;
        if val == 0 {
            break;
        }
    }
    idx
}

/// ## Map: 解碼 MQTT 3.1.1 / 5.0 可變長度剩餘長度
/// @outputs: 1
pub fn mqtt_remaining_length_decode(buf: &[u8]) -> Option<(u32, usize)> {
    if buf.is_empty() {
        return None;
    }
    let mut multiplier: u32 = 1;
    let mut value: u32 = 0;
    let mut bytes_consumed = 0;
    for &byte in buf.iter().take(4) {
        bytes_consumed += 1;
        value = value.saturating_add(((byte & 0x7F) as u32).saturating_mul(multiplier));
        if (byte & 0x80) == 0 {
            return Some((value, bytes_consumed));
        }
        multiplier = multiplier.saturating_mul(128);
    }
    None
}

/// ## Map: 打包 MQTT PUBLISH 控制封包第 1 位元組 (Flags: DUP, QoS, RETAIN)
/// @domain: qos [0, 2]
/// @outputs: 1
pub fn mqtt_publish_header_pack(dup: bool, qos: u8, retain: bool, out_byte: &mut u8) -> bool {
    if qos > 2 {
        return false;
    }
    let mut header = 0x30u8; // Packet Type 3 (PUBLISH) << 4
    if dup {
        header |= 0x08;
    }
    header |= (qos & 0x03) << 1;
    if retain {
        header |= 0x01;
    }
    *out_byte = header;
    true
}

/// ## Map: 封裝 RFC 7252 CoAP 4-byte 封包固定頭 (Version 1, Type, Code, Message ID)
/// @domain: type_field [0, 3]
/// @domain: code [0, 255]
/// @outputs: 1
pub fn coap_header_pack(type_field: u8, code: u8, message_id: u16, out_buf: &mut [u8; 4]) -> bool {
    if type_field > 3 {
        return false;
    }
    let ver_t_tkl = (1u8 << 6) | ((type_field & 0x03) << 4); // Ver=1, TKL=0
    out_buf[0] = ver_t_tkl;
    out_buf[1] = code;
    out_buf[2] = (message_id >> 8) as u8;
    out_buf[3] = (message_id & 0xFF) as u8;
    true
}

/// ## Map: RFC 7252 CoAP Option 增量差值編碼 (Header 1..2 bytes)
/// @outputs: 1
pub fn coap_option_delta_encode(prev_opt_num: u16, curr_opt_num: u16, opt_len: usize, out_buf: &mut [u8]) -> usize {
    if curr_opt_num < prev_opt_num || opt_len > 269 || out_buf.len() < 3 {
        return 0;
    }
    let delta = (curr_opt_num - prev_opt_num) as usize;
    let (d_nibble, d_ext) = if delta < 13 {
        (delta as u8, None)
    } else if delta <= 268 {
        (13u8, Some((delta - 13) as u8))
    } else {
        return 0;
    };
    let (l_nibble, l_ext) = if opt_len < 13 {
        (opt_len as u8, None)
    } else if opt_len <= 268 {
        (13u8, Some((opt_len - 13) as u8))
    } else {
        return 0;
    };
    out_buf[0] = (d_nibble << 4) | (l_nibble & 0x0F);
    let mut idx = 1;
    if let Some(ext) = d_ext {
        out_buf[idx] = ext;
        idx += 1;
    }
    if let Some(ext) = l_ext {
        out_buf[idx] = ext;
        idx += 1;
    }
    idx
}

/// ## Map: LoRaWAN 1.0.x / 1.1 實體層 MHDR (MAC Header) 解碼
/// @outputs: 1
pub fn lorawan_mhdr_unpack(mhdr_byte: u8) -> (u8, u8) {
    let mtype = (mhdr_byte >> 5) & 0x07;
    let major = mhdr_byte & 0x03;
    (mtype, major)
}

/// ## Map: LoRaWAN 16-bit 訊框計數器解纏繞至 32-bit (Roll-over Unroll)
/// @outputs: 1
pub fn lorawan_frame_counter_unroll(short_fcnt16: u16, last_fcnt32: u32) -> u32 {
    let last_short = (last_fcnt32 & 0xFFFF) as u16;
    let diff = short_fcnt16.wrapping_sub(last_short);
    if diff <= 0x7FFF {
        last_fcnt32.saturating_add(diff as u32)
    } else {
        last_fcnt32
    }
}

/// ## Map: 構建 Modbus FC 0x10 (Write Multiple Registers) 請求訊框
/// @domain: unit_id [0, 247]
/// @outputs: 1
pub fn modbus_fc10_request_pack(unit_id: u8, start_addr: u16, reg_values: &[u16], out_buf: &mut [u8]) -> usize {
    if unit_id > 247 || reg_values.is_empty() || reg_values.len() > 123 {
        return 0;
    }
    let count = reg_values.len();
    if out_buf.len() < 7 + count * 2 {
        return 0;
    }
    out_buf[0] = unit_id;
    out_buf[1] = 0x10; // FC 0x10
    out_buf[2] = (start_addr >> 8) as u8;
    out_buf[3] = (start_addr & 0xFF) as u8;
    out_buf[4] = (count >> 8) as u8;
    out_buf[5] = (count & 0xFF) as u8;
    out_buf[6] = (count * 2) as u8;
    let mut offset = 7;
    for &reg in reg_values {
        out_buf[offset] = (reg >> 8) as u8;
        out_buf[offset + 1] = (reg & 0xFF) as u8;
        offset += 2;
    }
    offset
}

/// ## Map: RFC 1055 SLIP 串列封包封裝 (0xC0 定界與 0xDB 轉義)
/// @outputs: 1
pub fn slip_encode_packet(data: &[u8], out_buf: &mut [u8]) -> usize {
    if data.is_empty() || out_buf.len() < data.len() + 2 {
        return 0;
    }
    let mut out_idx = 0;
    out_buf[out_idx] = 0xC0; // END
    out_idx += 1;
    for &b in data {
        if b == 0xC0 {
            if out_idx + 2 >= out_buf.len() { return 0; }
            out_buf[out_idx] = 0xDB;
            out_buf[out_idx + 1] = 0xDC;
            out_idx += 2;
        } else if b == 0xDB {
            if out_idx + 2 >= out_buf.len() { return 0; }
            out_buf[out_idx] = 0xDB;
            out_buf[out_idx + 1] = 0xDD;
            out_idx += 2;
        } else {
            if out_idx + 1 >= out_buf.len() { return 0; }
            out_buf[out_idx] = b;
            out_idx += 1;
        }
    }
    out_buf[out_idx] = 0xC0;
    out_idx + 1
}

/// ## Map: RFC 1055 SLIP 串列封包解碼 (還原轉義序列與剝離定界符)
/// @outputs: 1
pub fn slip_decode_packet(slip_frame: &[u8], out_buf: &mut [u8]) -> Option<usize> {
    if slip_frame.len() < 2 {
        return None;
    }
    let mut out_idx = 0;
    let mut in_esc = false;
    for &b in slip_frame {
        if b == 0xC0 {
            if in_esc { return None; }
            continue;
        }
        if in_esc {
            if out_idx >= out_buf.len() { return None; }
            if b == 0xDC {
                out_buf[out_idx] = 0xC0;
            } else if b == 0xDD {
                out_buf[out_idx] = 0xDB;
            } else {
                return None; // 違規轉義
            }
            out_idx += 1;
            in_esc = false;
        } else if b == 0xDB {
            in_esc = true;
        } else {
            if out_idx >= out_buf.len() { return None; }
            out_buf[out_idx] = b;
            out_idx += 1;
        }
    }
    if in_esc { None } else { Some(out_idx) }
}
