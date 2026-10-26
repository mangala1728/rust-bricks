// ## Map
// Role: core/iso7816_smartcard — 金融終端、ATM、刷卡機與智慧晶片卡安全微積木。
// Inputs: 卡號字元序列、ISO 7816 狀態碼 SW1/SW2、明文 PIN、卡片主帳號 (PAN)、TLV 位元組流、敏感記憶體緩衝區。
// Outputs: Luhn 卡號校驗布林值、狀態碼分類等級 (0..3)、Format 0 PIN Block 構造布林值、TLV 解析元組、記憶體清零。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、100% 零 unsafe、防記憶體殘留、控制流巢狀深度 <= 2。

use std::sync::atomic::{compiler_fence, Ordering};

/// @outputs: 2
/// @domain: digits_len [1..32]
pub fn luhn_checksum_validate(digits: &[u8]) -> bool {
    let len = digits.len();
    if len == 0 || len > 32 {
        return false;
    }
    let mut sum: u32 = 0;
    let mut double_digit = false;

    let mut i = len;
    while i > 0 {
        i -= 1;
        let byte = digits[i];
        let val = if byte >= b'0' && byte <= b'9' {
            (byte - b'0') as u32
        } else if byte <= 9 {
            byte as u32
        } else {
            return false;
        };

        if double_digit {
            let doubled = val * 2;
            sum += if doubled > 9 { doubled - 9 } else { doubled };
        } else {
            sum += val;
        }
        double_digit = !double_digit;
    }
    sum % 10 == 0
}

/// @outputs: 4
/// @domain: sw1 [0..255]
/// @domain: sw2 [0..255]
pub fn iso7816_sw_status(sw1: u8, sw2: u8) -> u8 {
    if sw1 == 0x90 && sw2 == 0x00 {
        return 0; // 成功 (Success)
    }
    if sw1 == 0x61 {
        return 0; // 成功且有待讀位元組 (Success with bytes remaining)
    }
    if sw1 == 0x62 || sw1 == 0x63 {
        return 1; // 警告 (Warning: 狀態未變更/密碼剩餘次數)
    }
    if sw1 >= 0x64 && sw1 <= 0x66 {
        return 2; // 執行錯誤 (Execution error)
    }
    if sw1 >= 0x67 && sw1 <= 0x6F {
        return 3; // 檢查錯誤 (Checking error: 檔案未找到/長度錯誤/安全條件不符)
    }
    3
}

/// @outputs: 1
fn encode_pin_field(pin: &[u8], field: &mut [u8; 8]) -> bool {
    let pin_len = pin.len();
    field[0] = pin_len as u8;
    for i in 1..8 {
        field[i] = 0xFF;
    }
    let mut nibble_idx = 2usize;
    for &ch in pin {
        let digit = if ch >= b'0' && ch <= b'9' {
            ch - b'0'
        } else if ch <= 9 {
            ch
        } else {
            return false;
        };
        let byte_pos = nibble_idx / 2;
        if nibble_idx % 2 == 0 {
            field[byte_pos] = (field[byte_pos] & 0x0F) | (digit << 4);
        } else {
            field[byte_pos] = (field[byte_pos] & 0xF0) | digit;
        }
        nibble_idx += 1;
    }
    true
}

/// @outputs: 1
fn encode_pan_field(pan: &[u8], field: &mut [u8; 8]) -> bool {
    let pan_len = pan.len();
    field[0] = 0x00;
    field[1] = 0x00;
    for i in 2..8 {
        field[i] = 0x00;
    }
    let start_idx = pan_len - 13;
    let end_idx = pan_len - 1;
    let mut nibble_idx = 4usize;
    for &ch in &pan[start_idx..end_idx] {
        let digit = if ch >= b'0' && ch <= b'9' {
            ch - b'0'
        } else if ch <= 9 {
            ch
        } else {
            return false;
        };
        let byte_pos = nibble_idx / 2;
        if nibble_idx % 2 == 0 {
            field[byte_pos] = (field[byte_pos] & 0x0F) | (digit << 4);
        } else {
            field[byte_pos] = (field[byte_pos] & 0xF0) | digit;
        }
        nibble_idx += 1;
    }
    true
}

/// @outputs: 2
/// @domain: pin_len [4..12]
/// @domain: pan_len [12..19]
pub fn iso9564_pin_block_format0(pin: &[u8], pan: &[u8], out_block: &mut [u8; 8]) -> bool {
    let pin_len = pin.len();
    let pan_len = pan.len();
    if pin_len < 4 || pin_len > 12 || pan_len < 13 || pan_len > 19 {
        return false;
    }
    let mut pin_field = [0u8; 8];
    if !encode_pin_field(pin, &mut pin_field) {
        return false;
    }
    let mut pan_field = [0u8; 8];
    if !encode_pan_field(pan, &mut pan_field) {
        return false;
    }
    for i in 0..8 {
        out_block[i] = pin_field[i] ^ pan_field[i];
    }
    zeroize_slice(&mut pin_field);
    zeroize_slice(&mut pan_field);
    true
}

/// @outputs: 1
/// @domain: buf_len [0..65536]
pub fn zeroize_slice(buf: &mut [u8]) {
    if buf.is_empty() || buf.len() > 65536 {
        return;
    }
    for byte in buf.iter_mut() {
        *byte = 0;
    }
    compiler_fence(Ordering::SeqCst);
}

/// @outputs: 2
/// @domain: data_len [2..1024]
pub fn ber_tlv_parse_tag_len(data: &[u8]) -> Option<(u32, usize, usize)> {
    let len = data.len();
    if len < 2 || len > 1024 {
        return None;
    }
    let mut idx = 0;
    let b0 = data[idx] as u32;
    idx += 1;

    let tag = if (b0 & 0x1F) == 0x1F {
        if idx >= len { return None; }
        let b1 = data[idx] as u32;
        idx += 1;
        (b0 << 8) | b1
    } else {
        b0
    };

    if idx >= len { return None; }
    let len_byte = data[idx];
    idx += 1;

    let value_len = if (len_byte & 0x80) == 0 {
        len_byte as usize
    } else {
        let num_octets = (len_byte & 0x7F) as usize;
        if num_octets == 0 || num_octets > 2 || idx + num_octets > len {
            return None;
        }
        let mut v = 0usize;
        for _ in 0..num_octets {
            v = (v << 8) | (data[idx] as usize);
            idx += 1;
        }
        v
    };

    if idx + value_len > len {
        return None;
    }
    Some((tag, value_len, idx))
}

/// @outputs: 2
/// @domain: field_num [1..64]
pub fn iso8583_bitmap_is_present(bitmap: &[u8; 8], field_num: u8) -> bool {
    if field_num < 1 || field_num > 64 {
        return false;
    }
    let bit_idx = (field_num - 1) as usize;
    let byte_pos = bit_idx / 8;
    let bit_offset = 7 - (bit_idx % 8);
    (bitmap[byte_pos] & (1 << bit_offset)) != 0
}
