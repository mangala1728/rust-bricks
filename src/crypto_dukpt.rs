// ## Map
// Role: core/crypto_dukpt — ANSI X9.24 金融終端 DUKPT (每筆交易唯一衍生金鑰) 序列號與變體派生微積木。
// Inputs: 10 位元組 KSN (Key Serial Number)、16 位元組當前衍生密鑰。
// Outputs: 21 位元交易計數器 u32、KSN 累加布林結果 bool、PIN/Data 變體金鑰陣列 [u8; 16]。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、計數器上限 0x1FFFFF (2,097,151) 溢出保護。

const MAX_COUNTER_21BIT: u32 = 0x1F_FFFF;

/// @outputs: 1
/// @domain: ksn_bytes [10..10]
pub fn dukpt_ksn_parse_counter(ksn: &[u8; 10]) -> u32 {
    // KSN 最後 3 字節包含 21-bit 計數器 (低 21 位)
    let b7 = ksn[7] as u32;
    let b8 = ksn[8] as u32;
    let b9 = ksn[9] as u32;
    let raw = (b7 << 16) | (b8 << 8) | b9;
    raw & MAX_COUNTER_21BIT
}

/// @outputs: 1
/// @domain: ksn_bytes [10..10]
pub fn dukpt_ksn_increment_counter(ksn: &mut [u8; 10]) -> bool {
    let curr = dukpt_ksn_parse_counter(ksn);
    if curr >= MAX_COUNTER_21BIT {
        return false;
    }
    let next = curr + 1;
    // 保留 ksn[7] 的高 3 位元
    let high_mask = (ksn[7] as u32) & 0xE0;
    ksn[7] = (high_mask | ((next >> 16) & 0x1F)) as u8;
    ksn[8] = ((next >> 8) & 0xFF) as u8;
    ksn[9] = (next & 0xFF) as u8;
    true
}

/// @outputs: 1
/// @domain: key_len [16..16]
pub fn dukpt_derive_pin_variant(current_key: &[u8; 16]) -> [u8; 16] {
    // ANSI X9.24 PIN 變體遮罩: byte 7 與 byte 15 異或 0xFF
    let mut variant = *current_key;
    variant[7] ^= 0xFF;
    variant[15] ^= 0xFF;
    variant
}

/// @outputs: 1
/// @domain: key_len [16..16]
pub fn dukpt_derive_data_variant(current_key: &[u8; 16]) -> [u8; 16] {
    // ANSI X9.24 Data 變體遮罩: byte 5 與 byte 13 異或 0xFF
    let mut variant = *current_key;
    variant[5] ^= 0xFF;
    variant[13] ^= 0xFF;
    variant
}
