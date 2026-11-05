// ## Map
// Role: msa/canfd_crc — CAN-FD CRC-17 的量測系統分析（MSA）零件：參考實作、公開標準件、挑戰件（已知錯誤寫法）、盲測比對。
// Inputs: 位元組資料。
// Outputs: 參考 CRC、各挑戰件的 CRC、盲測不一致筆數。
// Invariants: 獨立於 src/canfd_crc.rs（不得 use rust_bricks::canfd_crc）；參考實作必須先通過公開標準件，才可當量具的基準。
//
// @msa-standard-source: RevEng CRC Catalogue「CRC-17/CAN-FD」width=17 poly=0x1685b init=0x00000 refin=false refout=false xorout=0x00000 check=0x04f03 residue=0x00000
// @msa-blind: canfd_crc17_slice
// @msa-risk: 通訊互通（CRC 算錯就無法和真實 CAN-FD 設備交換資料）
// @msa-escape: customer
// @msa-station-cost: high
// 風險判斷：使用本積木的系統測試多半沒有真實 CAN-FD 設備，錯誤要到現場互通才會發現（流到客戶端），故收緊。
// ISO 11898 是產業標準而非 CE／FCC 類法規絕對值，所以不標 @msa-regulatory。

/// 多項式 x^17+x^16+x^14+x^13+x^11+x^6+x^4+x^3+x+1（最高次項隱含）
const POLY: u32 = 0x1685B;
const MASK: u32 = 0x1FFFF;
/// 規格的資料長度上限（CAN-FD 負載 > 16 位元組改用 CRC-21）
const MAX_LEN: usize = 16;

/// 公開標準件：(輸入, 公認正確值, 出處)。量具與參考實作都必須先過這些
pub const STANDARD_PARTS: &[(&[u8], u32, &str)] = &[(b"123456789", 0x04F03, "RevEng CRC-17/CAN-FD check")];

/// 一個位元的移位暫存器步進。augment=true 為 ISO 11898-1 寫法（等於 M(x)·x^17 mod G）；false 為把資料推進最低位（只算 M(x) mod G，不是 CRC）
fn step(crc: u32, bit: u8, augment: bool, poly: u32) -> Option<u32> {
    if crc > MASK || bit > 1 {
        return None;
    }
    let msb = (crc >> 16) & 1;
    if augment {
        let feedback = msb ^ bit as u32;
        let shifted = (crc << 1) & MASK;
        return Some(if feedback != 0 { shifted ^ poly } else { shifted });
    }
    let shifted = ((crc << 1) & MASK) | bit as u32;
    Some(if msb != 0 { shifted ^ poly } else { shifted })
}

/// 逐位元組計算；lsb_first=true 時每個位元組從最低位開始（錯誤的位元順序）
fn run(data: &[u8], init: u32, max_len: usize, lsb_first: bool, bit_fn: fn(u32, u8) -> Option<u32>) -> Option<u32> {
    if data.len() > max_len {
        return None;
    }
    let mut crc = init;
    for &byte in data {
        for i in 0..8u8 {
            let shift = if lsb_first { i } else { 7 - i };
            crc = bit_fn(crc, (byte >> shift) & 1)?;
        }
    }
    Some(crc)
}

/// 參考實作（標準）：RevEng CRC-17/CAN-FD 參數，MSB 先、init 0、資料 ≤ 16 位元組
pub mod standard {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, true, super::POLY)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0, super::MAX_LEN, false, canfd_crc17_bit)
    }
}

/// 挑戰件 1：原規格的寫法（資料推進最低位、沒乘 x^17）。2026-11-05 實際入庫的錯誤版本
pub mod challenge_no_augment {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, false, super::POLY)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0, super::MAX_LEN, false, canfd_crc17_bit)
    }
}

/// 挑戰件 2：用 ISO 11898-1:2015 訊框層的初值 0x10000（本積木規格採 RevEng 參數 init 0）
pub mod challenge_iso_frame_init {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, true, super::POLY)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0x10000, super::MAX_LEN, false, canfd_crc17_bit)
    }
}

/// 挑戰件 3：多項式少了 +1 項
pub mod challenge_wrong_poly {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, true, super::POLY ^ 1)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0, super::MAX_LEN, false, canfd_crc17_bit)
    }
}

/// 挑戰件 4：位元組內從最低位開始送（位元順序顛倒）
pub mod challenge_lsb_first {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, true, super::POLY)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0, super::MAX_LEN, true, canfd_crc17_bit)
    }
}

/// 挑戰件 5：長度上限寫成 64（CRC-17 只適用 ≤ 16 位元組）
pub mod challenge_len_64 {
    pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32> {
        super::step(crc, bit, true, super::POLY)
    }
    pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32> {
        super::run(data, 0, 64, false, canfd_crc17_bit)
    }
}

/// 參考實作先過公開標準件，並滿足「附上 CRC 後重算餘數為 0」（RevEng residue=0）
pub fn standard_parts_ok() -> bool {
    let checks = STANDARD_PARTS.iter().all(|(input, expected, _)| standard::canfd_crc17_slice(input) == Some(*expected));
    checks && residue_is_zero(b"123456789")
}

fn residue_is_zero(message: &[u8]) -> bool {
    let crc = match standard::canfd_crc17_slice(message) {
        Some(c) => c,
        None => return false,
    };
    // 訊息位元接著 CRC 的 17 個位元一起送進暫存器，結果應歸零
    let bits = message.iter().flat_map(|&b| (0..8).rev().map(move |s| (b >> s) & 1));
    let crc_bits = (0..17).rev().map(|i| ((crc >> i) & 1) as u8);
    let mut reg = 0u32;
    for bit in bits.chain(crc_bits) {
        reg = match standard::canfd_crc17_bit(reg, bit) {
            Some(r) => r,
            None => return false,
        };
    }
    reg == 0
}

/// 盲測：以規格中沒有的隨機輸入（長度 0..=16），比對受測實作與參考實作；回傳 (不一致筆數, 總筆數)
pub fn blind_check(candidate: fn(&[u8]) -> Option<u32>, n: usize) -> (usize, usize) {
    let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut mismatches = 0usize;
    for _ in 0..n {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let len = (state % 17) as usize;
        let data: Vec<u8> = (0..len).map(|k| (state.rotate_left(k as u32 * 8) & 0xFF) as u8).collect();
        if candidate(&data) != standard::canfd_crc17_slice(&data) {
            mismatches += 1;
        }
    }
    (mismatches, n)
}
