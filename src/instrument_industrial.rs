// ## Map
// Role: inst/industrial — 工控儀器通訊 (Keysight/Siglent DMM, CANopen, Modbus RTU/ASCII, DSLogic, LiteVNA) 協議解析與訊框構造微積木。
// Inputs: SCPI 科學計數法位元組串、CANopen 節點與索引、Modbus 暫存器、邏輯分析儀取樣率、VNA 掃頻參數。
// Outputs: 浮點讀值、CANopen COB-ID 與 SDO 訊框、Modbus RTU 查詢訊框、取樣時間 (ms)、VNA 掃頻頻率 (Hz)。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
pub fn scpi_parse_scientific_float(ascii_bytes: &[u8]) -> Option<f64> {
    if ascii_bytes.is_empty() {
        return None;
    }
    let (sign, mut idx) = if ascii_bytes[0] == b'-' { (-1.0, 1) } else if ascii_bytes[0] == b'+' { (1.0, 1) } else { (1.0, 0) };
    let (mut int_part, mut frac_part, mut div) = (0.0, 0.0, 1.0);
    let (mut exp_sign, mut exp_val, mut in_frac, mut in_exp, mut has_dig) = (1.0, 0.0, false, false, false);

    while idx < ascii_bytes.len() {
        let b = ascii_bytes[idx];
        if b == b'\r' || b == b'\n' || b == b' ' {
            break;
        }
        if b.is_ascii_digit() {
            has_dig = true;
            let d = (b - b'0') as f64;
            if in_exp {
                exp_val = exp_val * 10.0 + d;
            } else if in_frac {
                div *= 10.0;
                frac_part += d / div;
            } else {
                int_part = int_part * 10.0 + d;
            }
        } else if b == b'.' && !in_frac && !in_exp {
            in_frac = true;
        } else if (b == b'e' || b == b'E') && !in_exp {
            in_exp = true;
            if idx + 1 < ascii_bytes.len() && (ascii_bytes[idx + 1] == b'+' || ascii_bytes[idx + 1] == b'-') {
                if ascii_bytes[idx + 1] == b'-' {
                    exp_sign = -1.0;
                }
                idx += 1;
            }
        } else {
            return None;
        }
        idx += 1;
    }
    if !has_dig {
        return None;
    }
    Some(sign * (int_part + frac_part) * 10.0f64.powf(exp_sign * exp_val))
}

/// @outputs: 1
/// @domain: range_v [0.001..1000.0]
/// @domain: resolution_v [1.0e-7..1.0]
pub fn scpi_format_dmm_voltage_range_cmd(
    range_v: f64,
    resolution_v: f64,
    out_buf: &mut [u8],
) -> usize {
    if range_v <= 0.0 || resolution_v <= 0.0 || !range_v.is_finite() || !resolution_v.is_finite() {
        return 0;
    }
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(out_buf);
    if write!(cursor, "CONF:VOLT:DC {},{}\n", range_v, resolution_v).is_ok() {
        cursor.position() as usize
    } else {
        0
    }
}

/// @outputs: 1
/// @domain: node_id [1..127]
pub fn canopen_sdo_download_expedited_req(
    node_id: u8,
    index: u16,
    subindex: u8,
    data: &[u8],
) -> Option<(u16, [u8; 8])> {
    if node_id == 0 || node_id > 127 || data.is_empty() || data.len() > 4 {
        return None;
    }
    let cob_id = 0x600 + (node_id as u16);
    let num_empty_bytes = 4 - data.len();
    // CS = 0x23 (4 bytes), 0x27 (3 bytes), 0x2B (2 bytes), 0x2F (1 byte)
    let cs = 0x23 | ((num_empty_bytes as u8) << 2);
    let mut payload = [0u8; 8];
    payload[0] = cs;
    payload[1] = (index & 0xFF) as u8;
    payload[2] = ((index >> 8) & 0xFF) as u8;
    payload[3] = subindex;
    for (i, &b) in data.iter().enumerate() {
        payload[4 + i] = b;
    }
    Some((cob_id, payload))
}

/// @outputs: 1
pub fn canopen_sdo_parse_download_response(frame_data: &[u8; 8]) -> bool {
    // SDO 下載回應命令指示碼 0x60 代表成功
    frame_data[0] == 0x60
}

/// @outputs: 1
/// @domain: node_id [1..127]
pub fn canopen_heartbeat_producer_frame(node_id: u8, nmt_state: u8) -> (u16, [u8; 1]) {
    let cob_id = 0x700 + ((node_id & 0x7F) as u16);
    (cob_id, [nmt_state])
}

/// @outputs: 1
pub fn modbus_ascii_compute_lrc(frame_chars: &[u8]) -> u8 {
    let mut sum: u8 = 0;
    for &b in frame_chars {
        sum = sum.wrapping_add(b);
    }
    (!sum).wrapping_add(1)
}

/// @outputs: 1
/// @domain: slave_addr [1..247]
/// @domain: reg_count [1..125]
pub fn modbus_rtu_build_read_holding_registers(
    slave_addr: u8,
    start_reg: u16,
    reg_count: u16,
) -> [u8; 8] {
    let mut frame = [0u8; 8];
    frame[0] = slave_addr;
    frame[1] = 0x03; // Function Code: Read Holding Registers
    frame[2] = ((start_reg >> 8) & 0xFF) as u8;
    frame[3] = (start_reg & 0xFF) as u8;
    frame[4] = ((reg_count >> 8) & 0xFF) as u8;
    frame[5] = (reg_count & 0xFF) as u8;

    // 計算 Modbus RTU CRC16 (poly 0xA001)
    let mut crc: u16 = 0xFFFF;
    for i in 0..6 {
        crc ^= frame[i] as u16;
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    frame[6] = (crc & 0xFF) as u8;         // CRC 低位元先傳
    frame[7] = ((crc >> 8) & 0xFF) as u8;  // CRC 高位元後傳
    frame
}

/// @outputs: 1
pub fn modbus_decode_float32_registers(reg_hi: u16, reg_lo: u16, swap_words: bool) -> f32 {
    let bits = if swap_words {
        ((reg_lo as u32) << 16) | (reg_hi as u32)
    } else {
        ((reg_hi as u32) << 16) | (reg_lo as u32)
    };
    f32::from_bits(bits)
}

/// @outputs: 1
/// @domain: sample_rate_hz [1..1000000000]
pub fn dslogic_sample_depth_to_time_ms(samples: u32, sample_rate_hz: u32) -> Option<f64> {
    if sample_rate_hz == 0 {
        return None;
    }
    let time_s = (samples as f64) / (sample_rate_hz as f64);
    Some(time_s * 1000.0)
}

/// @outputs: 1
/// @domain: points [2..10001]
pub fn litevna_frequency_sweep_step(
    start_hz: u64,
    stop_hz: u64,
    points: u16,
    point_index: u16,
) -> Option<u64> {
    if points < 2 || stop_hz < start_hz || point_index >= points {
        return None;
    }
    let span = stop_hz - start_hz;
    let step_val = (span as u128) * (point_index as u128) / ((points - 1) as u128);
    Some(start_hz + step_val as u64)
}
