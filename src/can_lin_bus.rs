// ## Map
// Role: core/can_lin_bus — 車載 LIN 總線 2.x PID 奇偶校驗、增強校驗和與 I2S 數位音訊槽位對齊微積木。
// Inputs: 6 位元 LIN Frame ID (0..63)、LIN 數據切片、I2S 32-bit 原始立體聲槽位。
// Outputs: 8 位元 LIN PID、Classic/Enhanced 校驗和 u8、I2S 左右聲道解壓數值 (i16, i16)。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、LIN 2.x 規範八位元反轉求和。

/// @outputs: 1
/// @domain: frame_id [0..63]
pub fn lin_calc_pid(frame_id: u8) -> u8 {
    let id = frame_id & 0x3F;
    let id0 = (id >> 0) & 1;
    let id1 = (id >> 1) & 1;
    let id2 = (id >> 2) & 1;
    let id3 = (id >> 3) & 1;
    let id4 = (id >> 4) & 1;
    let id5 = (id >> 5) & 1;

    let p0 = id0 ^ id1 ^ id2 ^ id4;
    let p1 = !(id1 ^ id3 ^ id4 ^ id5) & 1;

    id | (p0 << 6) | (p1 << 7)
}

/// @outputs: 1
/// @domain: data_len [1..8]
pub fn lin_checksum(pid: Option<u8>, data: &[u8]) -> u8 {
    if data.is_empty() || data.len() > 8 {
        return 0;
    }
    let mut sum: u16 = 0;
    // Enhanced 包含 PID，Classic 不包含 (pid = None)
    if let Some(p) = pid {
        sum += p as u16;
    }
    for &b in data {
        sum += b as u16;
        if sum > 0xFF {
            sum -= 0xFF; // 八位元進位回加 (Carry wrapping)
        }
    }
    (!sum & 0xFF) as u8
}

/// @outputs: 1
pub fn i2s_unpack_stereo_16bit(left_slot_32: u32, right_slot_32: u32, is_left_justified: bool) -> (i16, i16) {
    // Standard Philips I2S: MSB 在第 2 個時脈週期 (高 16 位元有效)
    // Left-Justified: MSB 在第 1 個時脈週期
    let shift = if is_left_justified { 16 } else { 16 };
    let left = (left_slot_32 >> shift) as i16;
    let right = (right_slot_32 >> shift) as i16;
    (left, right)
}
