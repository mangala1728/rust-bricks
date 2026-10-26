//! # Lossless Compression & Variable-Length Wire Codec Micro-Bricks (無損數據壓縮與變長線路編碼微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 LEB128 / Protocol Buffers 7-bit 變長整數 (Varint) 編解碼、
//! ZigZag 64-bit 有符號數映射與解碼、感測器時序一階差分 (Delta) 編解碼步進、
//! 行程編碼 (RLE) 統計步進、LZ77 滑動視窗最長匹配前綴搜尋、
//! 非字節對齊位元流 (Bit-Packing) 抽樣、CRC-32C (Castagnoli 0x82F63B78) 步進。

/// ## Map: LEB128 / Protobuf 7-bit 變長整數無分支編碼 (Varint u64 Encode)
/// @outputs: 1
/// @domain: val [0, u64::MAX]
pub fn codec_leb128_u64_encode(mut val: u64, out: &mut [u8]) -> usize {
    if out.is_empty() {
        return 0;
    }
    let mut bytes_written = 0;
    while bytes_written < out.len() {
        let mut byte = (val & 0x7F) as u8;
        val >>= 7;
        if val != 0 {
            byte |= 0x80;
            out[bytes_written] = byte;
            bytes_written += 1;
        } else {
            out[bytes_written] = byte;
            bytes_written += 1;
            break;
        }
    }
    bytes_written
}

/// ## Decode: LEB128 變長整數解碼 (Varint u64 Decode)
/// 回傳 (解碼之 64 位元整數, 消耗之位元組數)。若資料截斷或溢位則安全返回 (0, 0)。
/// @outputs: 1
pub fn codec_leb128_u64_decode(buf: &[u8]) -> (u64, usize) {
    if buf.is_empty() {
        return (0, 0);
    }
    let mut result: u64 = 0;
    let mut shift: usize = 0;
    let mut count: usize = 0;

    for &byte in buf.iter() {
        if count >= 10 {
            return (0, 0); // 防範 64 位元溢位攻擊 (最多 10 個 bytes)
        }
        let low_7 = (byte & 0x7F) as u64;
        if shift >= 64 && low_7 != 0 {
            return (0, 0);
        }
        result |= low_7 << shift;
        count += 1;
        shift += 7;
        if (byte & 0x80) == 0 {
            return (result, count);
        }
    }
    (0, 0) // 緩衝區提前結束
}

/// ## Map: ZigZag 64-bit 有符號整數映射為正整數 (消滅負數高位元 1)
/// 公式：(n << 1) ^ (n >> 63)
/// @outputs: 1
/// @domain: n [i64::MIN, i64::MAX]
pub fn codec_zigzag_i64_encode(n: i64) -> u64 {
    ((n << 1) ^ (n >> 63)) as u64
}

/// ## Decode: ZigZag 64-bit 正整數還原為有符號整數
/// 公式：(n >> 1) ^ (-(n & 1))
/// @outputs: 1
/// @domain: n [0, u64::MAX]
pub fn codec_zigzag_i64_decode(n: u64) -> i64 {
    let half = (n >> 1) as i64;
    let mask = -((n & 1) as i64);
    half ^ mask
}

/// ## Step: 一階時序差分編碼單步 (Delta Encode Step)
/// @outputs: 1
/// @domain: current [i32::MIN, i32::MAX], previous [i32::MIN, i32::MAX]
pub fn codec_delta_encode_step(current: i32, previous: i32) -> i32 {
    current.wrapping_sub(previous)
}

/// ## Step: 一階時序差分解碼單步 (Delta Decode Step)
/// @outputs: 1
/// @domain: delta [i32::MIN, i32::MAX], previous [i32::MIN, i32::MAX]
pub fn codec_delta_decode_step(delta: i32, previous: i32) -> i32 {
    previous.wrapping_add(delta)
}

/// ## Map: 行程長度前綴計數 (Run-Length Count Prefix)
/// 回傳 (重複字元, 連續重複次數)
/// @outputs: 1
pub fn codec_rle_count_run(bytes: &[u8], max_run: usize) -> (u8, usize) {
    if bytes.is_empty() || max_run == 0 {
        return (0, 0);
    }
    let target = bytes[0];
    let limit = bytes.len().min(max_run);
    let mut count = 0;
    while count < limit && bytes[count] == target {
        count += 1;
    }
    (target, count)
}

/// ## Search: LZ77 滑動視窗最長前綴匹配搜尋 (Sliding Window Longest Match)
/// 回傳 (匹配距離 distance, 匹配長度 length)；若無匹配則回傳 (0, 0)
/// @outputs: 1
pub fn codec_lz77_find_longest_match(
    window: &[u8],
    lookahead: &[u8],
    max_len: usize,
) -> (usize, usize) {
    if window.is_empty() || lookahead.is_empty() || max_len == 0 {
        return (0, 0);
    }
    let max_match = lookahead.len().min(max_len);
    let mut best_dist = 0;
    let mut best_len = 0;

    for i in 0..window.len() {
        let mut len = 0;
        while len < max_match
            && (i + len) < window.len()
            && window[i + len] == lookahead[len]
        {
            len += 1;
        }
        if len > best_len {
            best_len = len;
            best_dist = window.len() - i;
        }
    }
    (best_dist, best_len)
}

/// ## Extract: 非對齊緊湊位元流抽取 (Bit-Packing Extract u16)
/// 從位元組切片中以指定 bit 偏移抽取 1..16 位元無符號整數
/// @outputs: 1
/// @domain: bit_offset [0, 1000000], bit_width [1, 16]
pub fn codec_bit_pack_extract_u16(data: &[u8], bit_offset: usize, bit_width: u8) -> u16 {
    if data.is_empty() || bit_width == 0 || bit_width > 16 {
        return 0;
    }
    let start_byte = bit_offset / 8;
    if start_byte >= data.len() {
        return 0;
    }
    let bit_in_byte = (bit_offset % 8) as u32;

    // 讀取最多 3 個 bytes 組成 24-bit 窗口以保證足夠覆蓋 16 bits
    let mut window = (data[start_byte] as u32) << 16;
    if start_byte + 1 < data.len() {
        window |= (data[start_byte + 1] as u32) << 8;
    }
    if start_byte + 2 < data.len() {
        window |= data[start_byte + 2] as u32;
    }

    let shift = 24 - bit_in_byte - (bit_width as u32);
    let mask = (1u32 << bit_width) - 1;
    ((window >> shift) & mask) as u16
}

/// ## Step: CRC-32C (Castagnoli 0x82F63B78) 逐位元組快速滾動步進
/// 廣泛應用於 iSCSI, Btrfs, ext4, NVMe-oF 及高效網路傳輸
/// @outputs: 1
/// @domain: crc [0, u32::MAX], byte [0, 255]
pub fn codec_crc32c_castagnoli_step(crc: u32, byte: u8) -> u32 {
    const POLY: u32 = 0x82F63B78;
    let mut c = crc ^ (byte as u32);
    for _ in 0..8 {
        if (c & 1) != 0 {
            c = (c >> 1) ^ POLY;
        } else {
            c >>= 1;
        }
    }
    c
}
