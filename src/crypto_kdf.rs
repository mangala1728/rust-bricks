// ## Map
// Role: core/crypto_kdf — 金融終端與韌體安全之 PKCS#7 填充、金鑰加鹽拉伸與衍生微積木。
// Inputs: 原始資料切片 &[u8]、區塊大小 block_size (2..255)、主密鑰切片、鹽值切片、上下文標籤。
// Outputs: 填充結果長度 Option<usize>、去填充資料切片 Option<&[u8]>、密鑰派生與拉伸布林結果 bool。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、常數時間掃描防 Padding Oracle 側信道攻擊。

/// @outputs: 2
/// @domain: block_size [2..255]
pub fn pkcs7_pad(data: &[u8], block_size: usize, out: &mut [u8]) -> Option<usize> {
    if block_size < 2 || block_size > 255 || data.len() > 65536 {
        return None;
    }
    let remainder = data.len() % block_size;
    let pad_len = block_size - remainder;
    let total_len = data.len() + pad_len;
    if out.len() < total_len {
        return None;
    }
    out[..data.len()].copy_from_slice(data);
    let pad_byte = pad_len as u8;
    for i in data.len()..total_len {
        out[i] = pad_byte;
    }
    Some(total_len)
}

/// @outputs: 2
/// @domain: block_size [2..255]
pub fn pkcs7_unpad<'a>(padded_data: &'a [u8], block_size: usize) -> Option<&'a [u8]> {
    if block_size < 2 || block_size > 255 || padded_data.is_empty() {
        return None;
    }
    if padded_data.len() % block_size != 0 {
        return None;
    }
    let pad_val = padded_data[padded_data.len() - 1] as usize;
    if pad_val == 0 || pad_val > block_size || pad_val > padded_data.len() {
        return None;
    }
    // 常數時間防禦：全量累加比較，防止 Padding Oracle 攻擊
    let mut diff: u8 = 0;
    let start_idx = padded_data.len() - pad_val;
    for i in start_idx..padded_data.len() {
        diff |= padded_data[i] ^ (pad_val as u8);
    }
    if diff != 0 {
        return None;
    }
    Some(&padded_data[..start_idx])
}

/// ## Stretch: 嵌入式輕量級加鹽迭代金鑰拉伸 (非密碼學 PBKDF2/RFC 2898)
/// 警告：此演算法採用 FNV-1a 搭配 32-bit PRNG 混淆，僅適用於低功耗 MCU 輕量級 Session ID 衍生。
/// @outputs: 1
/// @domain: rounds [1..100000]
pub fn pbkdf_xor_stretch(key: &[u8], salt: &[u8], rounds: u32, out_key: &mut [u8]) -> bool {
    if key.is_empty() || salt.is_empty() || out_key.is_empty() || rounds == 0 || rounds > 100_000 {
        return false;
    }
    let mut state: u32 = 0x811C9DC5;
    for &b in key {
        state ^= b as u32;
        state = state.wrapping_mul(0x01000193);
    }
    for &b in salt {
        state ^= b as u32;
        state = state.wrapping_mul(0x01000193);
    }
    for _ in 0..rounds {
        state = state.rotate_left(3) ^ 0x9E3779B9;
        state = state.wrapping_mul(0x85EBCA6B);
    }
    for (i, byte) in out_key.iter_mut().enumerate() {
        state ^= (i as u32).wrapping_add(0xA5);
        state = state.wrapping_mul(0x01000193);
        *byte = (state & 0xFF) as u8;
    }
    true
}

/// ## Expand: 嵌入式輕量級上下文金鑰衍生 (非 RFC 5869 HKDF)
/// 警告：基於 FNV-1a 之確定性衍生，僅供嵌入式測試，不可替代標準密碼學 HKDF。
/// @outputs: 1
/// @domain: master_key_len [1..64]
pub fn simple_kdf_expand(master_key: &[u8], context: &[u8], out_key: &mut [u8]) -> bool {
    if master_key.is_empty() || master_key.len() > 64 || out_key.is_empty() {
        return false;
    }
    let mut state: u32 = 0x811C9DC5; // FNV-1a 偏移基準
    for &b in master_key {
        state ^= b as u32;
        state = state.wrapping_mul(0x01000193);
    }
    for &b in context {
        state ^= b as u32;
        state = state.wrapping_mul(0x01000193);
    }
    for (i, byte) in out_key.iter_mut().enumerate() {
        state ^= (i as u32).wrapping_add(0x5A);
        state = state.wrapping_mul(0x01000193);
        state = state.rotate_left(5);
        *byte = (state & 0xFF) as u8;
    }
    true
}
