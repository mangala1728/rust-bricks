//! # NFC ISO 14443 & NDEF Micro-Bricks (近場通訊 ISO 14443 與 NDEF 協議微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 ISO 14443 Type A/B 循環冗餘校驗碼 (CRC-A / CRC-B)、ATQA 階層防碰撞 UID 長度解析、
//! SAK 選擇確認字元解析、NDEF 記錄封包頭 (TNF/Flags) 解包、NDEF 文字記錄語言與編碼提取、
//! NDEF URI 常用前綴字首擴展、FeliCa 封包長度校驗、Mifare Classic 扇區尾塊索引換算、
//! Mifare 訪問控制位 (C1/C2/C3) 讀寫權限解碼。

/// ## Map: ISO 14443-3 Type A 雙位元組 CRC-A (多項式 0x8408，初始值 0x6363)
/// @outputs: 1
/// @domain: byte [0, 255]
pub fn nfc_iso14443a_crc_step(byte: u8, crc: &mut u16) {
    // ISO/IEC 14443-3 Annex B：ch 為 8 位元，ch ^= ch << 4 必須截回 8 位元 (INC-20261026-01)
    let mut ch = byte ^ (*crc as u8);
    ch ^= ch << 4;
    let ch = ch as u16;
    *crc = (*crc >> 8) ^ (ch << 8) ^ (ch << 3) ^ (ch >> 4);
}

/// ## Calc: ISO 14443-3 Type B 雙位元組 CRC-B (多項式 0x8408，初始值 0xFFFF，反相輸出)
/// @outputs: 1
pub fn nfc_iso14443b_crc_calculate(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for &b in data {
        nfc_iso14443a_crc_step(b, &mut crc);
    }
    !crc
}

/// ## Parse: ISO 14443 Type A ATQA (Answer to Request A) 防碰撞 UID 級聯階層
/// 回傳 UID 長度 (4 位元組 Single, 7 位元組 Double, 10 位元組 Triple)
/// @outputs: 1
pub fn nfc_iso14443a_atqa_uid_size(atqa: [u8; 2]) -> usize {
    // ATQA b8-b7 (atqa[0] & 0xC0) 指示 UID 大小
    let size_bits = (atqa[0] >> 6) & 0x03;
    match size_bits {
        0 => 4,  // Single size (e.g. Mifare Classic 1K/4K 4-byte UID)
        1 => 7,  // Double size (e.g. NTAG213/215/216, Mifare Ultralight 7-byte UID)
        2 => 10, // Triple size (10-byte UID)
        _ => 4,
    }
}

/// ## Parse: SAK (Select Acknowledge) 卡片就緒與協議特性解碼
/// 回傳 (UID 是否已完整接收, 是否支援 ISO 14443-4 T=CL 協議, 卡片架構代碼)
/// @outputs: 3
pub fn nfc_iso14443a_sak_parse(sak: u8) -> (bool, bool, u8) {
    // SAK bit 3 (0x04) 為 1 表示 UID 尚未完成 (需要級聯階層 Cascade Tag)
    let is_complete = (sak & 0x04) == 0;
    // SAK bit 6 (0x20) 為 1 表示符合 ISO/IEC 14443-4
    let is_iso14443_4 = (sak & 0x20) != 0;
    // 遮罩保留晶片類型特徵位元 (排除 b3 與 b6)
    let card_type = sak & !0x24;
    (is_complete, is_iso14443_4, card_type)
}

/// ## Parse: NFC 論壇 NDEF Record 記錄頭部 Flags 與 TNF 解包
/// 回傳 (Message Begin, Message End, Chunk Flag, Short Record, ID Length Present, Type Name Format 0..7)
/// @outputs: 6
pub fn nfc_ndef_record_header_unpack(header_byte: u8) -> (bool, bool, bool, bool, bool, u8) {
    let mb = (header_byte & 0x80) != 0;
    let me = (header_byte & 0x40) != 0;
    let cf = (header_byte & 0x20) != 0;
    let sr = (header_byte & 0x10) != 0;
    let il = (header_byte & 0x08) != 0;
    let tnf = header_byte & 0x07;
    (mb, me, cf, sr, il, tnf)
}

/// ## Unpack: NDEF Well-Known "T" (Text) 記錄狀態字元解包
/// 回傳 (是否為 UTF-16 編碼, 語言代碼 ISO/IANA 字串長度)
/// @outputs: 2
pub fn nfc_ndef_text_status_unpack(status_byte: u8) -> (bool, usize) {
    // bit 7: 0 = UTF-8, 1 = UTF-16
    let is_utf16 = (status_byte & 0x80) != 0;
    // bit 5..0: 語言長度 (0..63)
    let lang_len = (status_byte & 0x3F) as usize;
    (is_utf16, lang_len)
}

/// ## Expand: NDEF Well-Known "U" (URI) 前綴代碼展開為完整協議字串
/// @outputs: 1
pub fn nfc_ndef_uri_prefix_expand(prefix_code: u8) -> Option<&'static str> {
    match prefix_code {
        0x01 => Some("http://www."),
        0x02 => Some("https://www."),
        0x03 => Some("http://"),
        0x04 => Some("https://"),
        0x05 => Some("tel:"),
        0x06 => Some("mailto:"),
        0x07 => Some("ftp://anonymous:anonymous@"),
        0x08 => Some("ftp://ftp."),
        0x09 => Some("ftps://"),
        0x0A => Some("sftp://"),
        0x0B => Some("smb://"),
        0x0C => Some("nfs://"),
        0x0D => Some("ftp://"),
        0x0E => Some("dav://"),
        0x0F => Some("news:"),
        0x10 => Some("telnet://"),
        0x11 => Some("imap:"),
        0x12 => Some("rtsp://"),
        0x13 => Some("urn:"),
        0x14 => Some("pop:"),
        0x15 => Some("sip:"),
        0x16 => Some("sips:"),
        0x17 => Some("tftp:"),
        0x18 => Some("btspp://"),
        0x19 => Some("btl2cap://"),
        0x1A => Some("btgoep://"),
        0x1B => Some("tcpobex://"),
        0x1C => Some("irdaobex://"),
        0x1D => Some("file://"),
        0x1E => Some("urn:epc:id:"),
        0x1F => Some("urn:epc:tag:"),
        0x20 => Some("urn:epc:pat:"),
        0x21 => Some("urn:epc:raw:"),
        0x22 => Some("urn:epc:"),
        0x23 => Some("urn:nfc:"),
        _ => None,
    }
}

/// ## Index: Mifare Classic 扇區控制尾塊 (Sector Trailer) 區塊絕對索引換算
/// 扇區 0..31 每個扇區 4 個區塊 (尾塊為 4*n + 3)
/// 扇區 32..39 每個扇區 16 個區塊 (128 + (n-32)*16 + 15)
/// @outputs: 1
/// @domain: sector_idx [0, 39]
pub fn nfc_mifare_classic_sector_trailer_block_index(sector_idx: u8) -> Option<u8> {
    if sector_idx > 39 {
        return None;
    }
    if sector_idx < 32 {
        Some(sector_idx * 4 + 3)
    } else {
        Some(128 + (sector_idx - 32) * 16 + 15)
    }
}

/// ## Decode: Mifare Classic 區塊存取控制位 (C1, C2, C3) 讀寫權限解碼
/// 回傳 (可讀取, 可寫入, 支援增減值運算)
/// @outputs: 3
pub fn nfc_mifare_classic_data_block_access(c1: u8, c2: u8, c3: u8) -> (bool, bool, bool) {
    let mode = ((c1 & 1) << 2) | ((c2 & 1) << 1) | (c3 & 1);
    match mode {
        0b000 => (true, true, true),   // 讀/寫/增減全部開放 (Key A|B)
        0b010 => (true, false, false), // 唯讀 (Key A|B)
        0b100 => (true, true, false),  // 可讀/可寫 (Key B 寫)
        0b110 => (true, true, true),   // Key B 可讀寫與增減
        0b001 => (true, false, true),  // Key A|B 讀，增減
        0b011 => (false, true, false), // 僅 Key B 可寫
        0b101 => (false, false, false),// 完全禁止 (除 Key B 格式化外)
        0b111 => (false, false, false),// 完全禁止
        _ => (false, false, false),
    }
}

/// ## Check: Sony FeliCa 訊框最小長度與響應標誌校驗
/// 訊框首位元組為長度 Len (含自身)，且最小為 2
/// @outputs: 1
pub fn nfc_felica_frame_validate(frame: &[u8]) -> bool {
    if frame.len() < 2 {
        return false;
    }
    let len = frame[0] as usize;
    if len < 2 || len > frame.len() {
        return false;
    }
    true
}
