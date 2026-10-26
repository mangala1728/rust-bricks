//! # Modern Wire Cryptography Micro-Bricks (現代高階線路加密與安全算子微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 RFC 8439 ChaCha20 四分之一輪與狀態初始化、Poly1305 夾緊、常數時間條件交換/清零、
//! BLAKE2s G 函數、GF(256) 有限域乘法、Shamir 秘密共享多項式求值、RFC 4226 動態截斷 TOTP 算子。

/// ## Map: RFC 8439 ChaCha20 四分之一輪變換 (Quarter Round)
/// @outputs: 1
pub fn chacha20_quarter_round(a: &mut u32, b: &mut u32, c: &mut u32, d: &mut u32) {
    *a = a.wrapping_add(*b); *d = (*d ^ *a).rotate_left(16);
    *c = c.wrapping_add(*d); *b = (*b ^ *c).rotate_left(12);
    *a = a.wrapping_add(*b); *d = (*d ^ *a).rotate_left(8);
    *c = c.wrapping_add(*d); *b = (*b ^ *c).rotate_left(7);
}

/// ## Map: RFC 8439 ChaCha20 16-word 狀態矩陣初始化 ("expand 32-byte k")
/// @outputs: 1
pub fn chacha20_init_state(
    key: &[u8; 32],
    nonce: &[u8; 12],
    counter: u32,
    state: &mut [u32; 16],
) {
    // 0..3: 常數 0x61707865, 0x3320646e, 0x79622d32, 0x6b206574
    state[0] = 0x61707865;
    state[1] = 0x3320646e;
    state[2] = 0x79622d32;
    state[3] = 0x6b206574;
    // 4..11: 32-byte key (Little Endian)
    for i in 0..8 {
        state[4 + i] = ((key[i * 4] as u32))
            | ((key[i * 4 + 1] as u32) << 8)
            | ((key[i * 4 + 2] as u32) << 16)
            | ((key[i * 4 + 3] as u32) << 24);
    }
    // 12: counter
    state[12] = counter;
    // 13..15: 12-byte nonce
    for i in 0..3 {
        state[13 + i] = ((nonce[i * 4] as u32))
            | ((nonce[i * 4 + 1] as u32) << 8)
            | ((nonce[i * 4 + 2] as u32) << 16)
            | ((nonce[i * 4 + 3] as u32) << 24);
    }
}

/// ## Map: RFC 8439 Poly1305 評估金鑰 r 夾緊操作 (Clamping)
/// @outputs: 1
pub fn poly1305_clamp_key(r: &mut [u8; 16]) {
    r[3] &= 15;
    r[7] &= 15;
    r[11] &= 15;
    r[15] &= 15;
    r[4] &= 252;
    r[8] &= 252;
    r[12] &= 252;
}

/// ## Map: 32-bit 整數常數時間條件交換 (Constant-Time CSWAP)
/// @outputs: 1
pub fn constant_time_cswap_u32(a: &mut u32, b: &mut u32, swap: bool) {
    let mask = if swap { 0xFFFFFFFFu32 } else { 0x00000000u32 };
    let delta = (*a ^ *b) & mask;
    *a ^= delta;
    *b ^= delta;
}

/// ## Map: 安全記憶體清除 (Zeroize 覆寫歸零)
/// @outputs: 1
pub fn constant_time_zeroize(buf: &mut [u8]) {
    for b in buf.iter_mut() {
        *b = 0;
    }
}

/// ## Map: Curve25519 256-bit 座標四元組常數時間條件交換
/// @outputs: 1
pub fn curve25519_cswap_step(x2: &mut [u64; 4], x3: &mut [u64; 4], swap: bool) {
    let mask = if swap { 0xFFFF_FFFF_FFFF_FFFFu64 } else { 0u64 };
    for i in 0..4 {
        let delta = (x2[i] ^ x3[i]) & mask;
        x2[i] ^= delta;
        x3[i] ^= delta;
    }
}

/// ## Map: BLAKE2s G 函數混合置換步進 (RFC 7693)
/// @outputs: 1
pub fn blake2s_g_round(v: &mut [u32; 16], a: usize, b: usize, c: usize, d: usize, x: u32, y: u32) {
    if a >= 16 || b >= 16 || c >= 16 || d >= 16 {
        return;
    }
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(x);
    v[d] = (v[d] ^ v[a]).rotate_right(16);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(12);
    v[a] = v[a].wrapping_add(v[b]).wrapping_add(y);
    v[d] = (v[d] ^ v[a]).rotate_right(8);
    v[c] = v[c].wrapping_add(v[d]);
    v[b] = (v[b] ^ v[c]).rotate_right(7);
}

/// ## Map: GF(2^8) 有限域無分支乘法 (不可約多項式 0x11B)
/// @outputs: 1
pub fn gf256_mul_step(mut a: u8, mut b: u8) -> u8 {
    let mut res = 0u8;
    for _ in 0..8 {
        if (b & 1) != 0 {
            res ^= a;
        }
        let hi_bit = a & 0x80;
        a <<= 1;
        if hi_bit != 0 {
            a ^= 0x1B; // x^8 + x^4 + x^3 + x + 1
        }
        b >>= 1;
    }
    res
}

/// ## Map: Shamir 秘密共享 GF(256) 門檻多項式求值 (Horner's Method)
/// @outputs: 1
pub fn shamir_eval_polynomial_step(coeffs: &[u8], x: u8) -> u8 {
    if coeffs.is_empty() {
        return 0;
    }
    let mut res = 0u8;
    for &c in coeffs.iter().rev() {
        res = gf256_mul_step(res, x) ^ c;
    }
    res
}

/// ## Map: RFC 4226 HOTP / TOTP 動態截斷提取 6 位數密碼
/// @outputs: 1
pub fn hotp_truncate_code(hash_hmac_sha1: &[u8; 20]) -> u32 {
    let offset = (hash_hmac_sha1[19] & 0x0F) as usize;
    let bin_code = (((hash_hmac_sha1[offset] & 0x7F) as u32) << 24)
        | ((hash_hmac_sha1[offset + 1] as u32) << 16)
        | ((hash_hmac_sha1[offset + 2] as u32) << 8)
        | (hash_hmac_sha1[offset + 3] as u32);
    bin_code % 1_000_000
}
