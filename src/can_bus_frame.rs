// ## Map
// Role: core/can_bus_frame — 車載總線 (CAN/CAN-FD)、ISO-TP 車載診斷與工控 MCU SDK 微積木。
// Inputs: CAN ID 原始碼、DLC 長度碼、Modbus 資料訊框、ISO-TP 負載、系統時鐘頻率。
// Outputs: 擴展幀判定與清淨 ID、CAN-FD 位元組長度 (0..64)、Modbus CRC16 校驗碼、ISO-TP 單幀、定時器預分頻與重載值。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、100% 零 unsafe、防除以零溢出、控制流巢狀深度 <= 2。

const CAN_EFF_FLAG: u32 = 0x8000_0000;

/// @outputs: 2
/// @domain: raw_id [0..4294967295]
pub fn can_id_unpack(raw_id: u32) -> (bool, u32) {
    let is_extended = (raw_id & CAN_EFF_FLAG) != 0;
    let clean_id = if is_extended {
        raw_id & 0x1FFF_FFFF
    } else {
        raw_id & 0x0000_07FF
    };
    (is_extended, clean_id)
}

/// @outputs: 1
/// @domain: dlc [0..15]
pub fn can_fd_dlc_to_len(dlc: u8) -> u8 {
    if dlc > 15 {
        return 0;
    }
    match dlc {
        0..=8 => dlc,
        9 => 12,
        10 => 16,
        11 => 20,
        12 => 24,
        13 => 32,
        14 => 48,
        15 => 64,
        _ => 0,
    }
}

/// @outputs: 1
/// @domain: data_len [0..4096]
pub fn crc16_modbus(data: &[u8]) -> u16 {
    let len = data.len();
    if len > 4096 {
        return 0;
    }
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

/// @outputs: 2
/// @domain: payload_len [1..7]
pub fn iso_tp_single_frame_encode(payload: &[u8], out_frame: &mut [u8; 8]) -> bool {
    let len = payload.len();
    if len == 0 || len > 7 {
        return false;
    }
    out_frame[0] = len as u8; // SF_DL (Single Frame Data Length)
    for i in 0..len {
        out_frame[i + 1] = payload[i];
    }
    for i in (len + 1)..8 {
        out_frame[i] = 0xCC; // 標準填充字元 (Padding)
    }
    true
}

/// @outputs: 2
/// @domain: sys_clock_hz [1..1000000000]
/// @domain: target_freq_hz [1..100000000]
pub fn timer_calc_prescaler_and_period(sys_clock_hz: u32, target_freq_hz: u32) -> Option<(u16, u16)> {
    if sys_clock_hz == 0 || target_freq_hz == 0 || target_freq_hz > sys_clock_hz {
        return None;
    }
    let total_div = sys_clock_hz / target_freq_hz;
    if total_div == 0 {
        return None;
    }
    let psc = (total_div / 65536) as u32;
    if psc > 65535 {
        return None;
    }
    let arr = total_div / (psc + 1);
    if arr == 0 || arr > 65536 {
        return None;
    }
    let final_psc = psc as u16;
    let final_arr = (arr - 1) as u16;
    Some((final_psc, final_arr))
}
