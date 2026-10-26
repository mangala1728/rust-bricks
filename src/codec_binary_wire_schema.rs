//! # 序列化與二進位線路傳輸架構 (Binary Serialization & Flat Buffering)
//!
//! 提供 CBOR 標頭解包、MessagePack 型別分類、ASN.1 DER 長度編解碼、
//! FlatBuffers 虛擬表 (VTable) 欄位偏移尋址、BSON 型別檢驗、
//! Avro ZigZag 64位元長整數、12位元 ADC 緊湊解包與記憶體對齊微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: CBOR (RFC 8949) 主型別與附加資訊解包 (CBOR Header Unpack)
/// 回傳 (Major Type 0..7, Additional Info 0..31)
/// @outputs: 1
/// @domain: byte <= 0xFF
pub fn cbor_header_unpack(byte: u8) -> (u8, u8) {
    let major_type = (byte >> 5) & 0x07;
    let additional_info = byte & 0x1F;
    (major_type, additional_info)
}

/// ## Step: MessagePack 標頭位元組族系分類 (MessagePack Family Classifier)
/// 回傳族系代碼: 0=PositiveFixInt, 1=FixMap, 2=FixArray, 3=FixStr, 4=Nil, 5=Bool, 6=Float, 7=Int, 8=Bin, 9=Ext, 10=NegativeFixInt
/// @outputs: 1
/// @domain: byte <= 0xFF
pub fn msgpack_classify_family(byte: u8) -> u8 {
    if (byte & 0x80) == 0 {
        0 // Positive FixInt (0x00..=0x7F)
    } else if (byte & 0xF0) == 0x80 {
        1 // FixMap (0x80..=0x8F)
    } else if (byte & 0xF0) == 0x90 {
        2 // FixArray (0x90..=0x9F)
    } else if (byte & 0xE0) == 0xA0 {
        3 // FixStr (0xA0..=0xBF)
    } else if byte == 0xC0 {
        4 // Nil
    } else if byte == 0xC2 || byte == 0xC3 {
        5 // Boolean
    } else if byte == 0xCA || byte == 0xCB {
        6 // Float32 / Float64
    } else if matches!(byte, 0xCC..=0xD3) {
        7 // Int8..64 / UInt8..64
    } else if matches!(byte, 0xC4..=0xC6) {
        8 // Bin8..32
    } else if matches!(byte, 0xC7..=0xC9 | 0xD4..=0xD8) {
        9 // Ext8..32 / FixExt
    } else if (byte as i8) < 0 && (byte & 0xE0) == 0xE0 {
        10 // Negative FixInt (0xE0..=0xFF)
    } else {
        255 // 未知
    }
}

/// ## Calc: ASN.1 DER 長度欄位解碼 (ASN.1 DER Length Decode)
/// 回傳 (有效負載長度, 長度欄位所佔字節數)
/// @outputs: 1
/// @domain: bytes.len() > 0 && bytes.len() <= 64
pub fn asn1_der_length_decode(bytes: &[u8]) -> Option<(usize, usize)> {
    if bytes.is_empty() || bytes.len() > 64 {
        return None;
    }
    let lead = bytes[0];
    if (lead & 0x80) == 0 {
        // 短格式 (Short Form): 0..127
        Some((lead as usize, 1))
    } else {
        // 長格式 (Long Form): 低 7 位代表後續長度字節數
        let octets = (lead & 0x7F) as usize;
        if octets == 0 || octets > 4 || bytes.len() < (1 + octets) {
            return None;
        }
        let mut len: usize = 0;
        for &b in &bytes[1..=octets] {
            len = (len << 8) | (b as usize);
        }
        Some((len, 1 + octets))
    }
}

/// ## Calc: 數值編碼為 ASN.1 DER 長度字節 (ASN.1 DER Length Encode)
/// 回傳寫入 out_buf 之字節數
/// @outputs: 1
/// @domain: out_buf.len() >= 5
pub fn asn1_der_length_encode(length: usize, out_buf: &mut [u8]) -> Option<usize> {
    if out_buf.is_empty() {
        return None;
    }
    if length <= 127 {
        out_buf[0] = length as u8;
        Some(1)
    } else if length <= 0xFF && out_buf.len() >= 2 {
        out_buf[0] = 0x81;
        out_buf[1] = length as u8;
        Some(2)
    } else if length <= 0xFFFF && out_buf.len() >= 3 {
        out_buf[0] = 0x82;
        out_buf[1] = (length >> 8) as u8;
        out_buf[2] = (length & 0xFF) as u8;
        Some(3)
    } else if length <= 0x00FF_FFFF && out_buf.len() >= 4 {
        out_buf[0] = 0x83;
        out_buf[1] = (length >> 16) as u8;
        out_buf[2] = ((length >> 8) & 0xFF) as u8;
        out_buf[3] = (length & 0xFF) as u8;
        Some(4)
    } else if out_buf.len() >= 5 {
        out_buf[0] = 0x84;
        out_buf[1] = (length >> 24) as u8;
        out_buf[2] = ((length >> 16) & 0xFF) as u8;
        out_buf[3] = ((length >> 8) & 0xFF) as u8;
        out_buf[4] = (length & 0xFF) as u8;
        Some(5)
    } else {
        None
    }
}

/// ## Calc: Google FlatBuffers 虛擬表 (VTable) 欄位偏移尋址 (FlatBuffers VTable Field Offset)
/// @outputs: 1
/// @domain: vtable_bytes.len() >= 4 && vtable_bytes.len() <= 1024
pub fn flatbuffers_vtable_field_offset(vtable_bytes: &[u8], field_index: usize) -> Option<u16> {
    if vtable_bytes.len() < 4 || vtable_bytes.len() > 1024 {
        return None;
    }
    let vtable_size = u16::from_le_bytes([vtable_bytes[0], vtable_bytes[1]]) as usize;
    let entry_offset = 4 + field_index * 2;
    if entry_offset + 2 > vtable_size || entry_offset + 2 > vtable_bytes.len() {
        return Some(0); // 欄位不存在 (使用預設值)
    }
    let field_offset = u16::from_le_bytes([vtable_bytes[entry_offset], vtable_bytes[entry_offset + 1]]);
    Some(field_offset)
}

/// ## Step: BSON 元素型別識別碼合法性檢驗 (BSON Element Type Valid)
/// @outputs: 1
/// @domain: type_byte <= 0xFF
pub fn bson_element_type_valid(type_byte: u8) -> bool {
    matches!(
        type_byte,
        0x01 | 0x02 | 0x03 | 0x04 | 0x05 | 0x07 | 0x08 | 0x09 | 0x0A | 0x10 | 0x12
    )
}

/// ## Calc: Apache Avro 64 位元 ZigZag 解碼 (Avro ZigZag Long Decode)
/// @outputs: 1
/// @domain: zigzag_val <= u64::MAX
pub fn avro_zigzag_long_decode(zigzag_val: u64) -> i64 {
    ((zigzag_val >> 1) as i64) ^ (-((zigzag_val & 1) as i64))
}

/// ## Calc: 緊湊 12 位元 ADC 取樣值解包 (12-bit ADC Bitfield Unpack)
/// 從 3 位元組中解出第 1 個 (0..12) 或第 2 個 (12..24) 12-bit 採樣值
/// @outputs: 1
/// @domain: packed.len() == 3
pub fn bitfield_extract_12bit(packed: &[u8; 3], is_second: bool) -> u16 {
    if !is_second {
        // 第一個 sample: packed[0] 與 packed[1] 的低 4-bit
        ((packed[0] as u16) << 4) | ((packed[1] as u16) & 0x0F)
    } else {
        // 第二個 sample: packed[1] 的高 4-bit 與 packed[2]
        (((packed[1] as u16) & 0xF0) << 4) | (packed[2] as u16)
    }
}

/// ## Calc: 結構體記憶體邊界對齊填充需求字節 (Struct Alignment Padding Needed)
/// @outputs: 1
/// @domain: alignment 必須為 1, 2, 4, 8, 16
pub fn struct_alignment_padding_needed(current_offset: usize, alignment: usize) -> usize {
    if !matches!(alignment, 1 | 2 | 4 | 8 | 16) {
        return 0;
    }
    let rem = current_offset % alignment;
    if rem == 0 { 0 } else { alignment - rem }
}

/// ## Calc: 32 位元整數網路位元組序 (Big-Endian) 原地轉主機序 (Endian Swap U32)
/// @outputs: 1
/// @domain: val <= u32::MAX
pub fn endian_swap_u32_in_place(val: u32) -> u32 {
    val.swap_bytes()
}
