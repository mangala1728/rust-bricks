// ## Map
// Role: core/packet_checksum — 零外部依賴之純標準庫 CRC32 (IEEE 802.3) 與 Adler-32 校驗微積木（含 8-bit 與 16-bit 異變轉譯）。
// Inputs: 8-bit 二進位位元組切片、16-bit PCM/感測器整數取樣切片、初始校驗狀態。
// Outputs: 32 位元無號整數校驗碼 (u32)。
// Invariants: 零全域可變狀態、無遞迴、純標準庫 100% 零依賴、有界迴圈處理。

const fn generate_crc32_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut crc = i as u32;
        let mut j = 0;
        while j < 8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
            j += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

const CRC32_TABLE: [u32; 256] = generate_crc32_table();
const MOD_ADLER: u32 = 65521;
const MAX_CHUNK: usize = 5552;

/// @outputs: 1
pub fn crc32_ieee_update(prev_crc: u32, data: &[u8]) -> u32 {
    let mut crc = !prev_crc;
    for &byte in data {
        let idx = ((crc ^ (byte as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx];
    }
    !crc
}

/// @outputs: 1
pub fn crc32_ieee(data: &[u8]) -> u32 {
    crc32_ieee_update(0, data)
}

/// @outputs: 1
pub fn crc32_pcm16_le(samples: &[i16]) -> u32 {
    let mut crc = !0u32;
    for &sample in samples {
        let u = sample as u16;
        let b0 = (u & 0xFF) as u8;
        let b1 = ((u >> 8) & 0xFF) as u8;
        let idx0 = ((crc ^ (b0 as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx0];
        let idx1 = ((crc ^ (b1 as u32)) & 0xFF) as usize;
        crc = (crc >> 8) ^ CRC32_TABLE[idx1];
    }
    !crc
}

/// @outputs: 1
pub fn adler32_update(prev_adler: u32, data: &[u8]) -> u32 {
    let mut s1 = prev_adler & 0xFFFF;
    let mut s2 = (prev_adler >> 16) & 0xFFFF;

    for chunk in data.chunks(MAX_CHUNK) {
        for &byte in chunk {
            s1 += byte as u32;
            s2 += s1;
        }
        s1 %= MOD_ADLER;
        s2 %= MOD_ADLER;
    }
    (s2 << 16) | s1
}

/// @outputs: 1
pub fn adler32(data: &[u8]) -> u32 {
    adler32_update(1, data)
}

/// @outputs: 1
pub fn adler32_pcm16_le(samples: &[i16]) -> u32 {
    let mut s1 = 1u32;
    let mut s2 = 0u32;

    for chunk in samples.chunks(MAX_CHUNK / 2) {
        for &sample in chunk {
            let u = sample as u16;
            let b0 = (u & 0xFF) as u32;
            let b1 = ((u >> 8) & 0xFF) as u32;
            s1 += b0;
            s2 += s1;
            s1 += b1;
            s2 += s1;
        }
        s1 %= MOD_ADLER;
        s2 %= MOD_ADLER;
    }
    (s2 << 16) | s1
}
