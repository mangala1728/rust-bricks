//! # Analyzer Sigrok Protocol Micro-Bricks (邏輯分析儀與匯流排協議解碼微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 數位邏輯邊緣檢測 (上升沿/下降沿)、抗噪濾波器 (Glitch Filter)、UART 位元取樣中心點偏移、
//! I2C START/STOP/DATA 匯流排條件判讀、SPI 模式 (CPOL/CPHA) 取樣沿決定、
//! Dallas 1-Wire 脈衝寬度解碼、Manchester 雙相碼躍變解碼、邏輯分析儀行程編碼 (RLE) 壓縮、
//! 遮罩式硬體觸發條件比對、CAN 總線時間量子 (Time Quanta) 取樣點比例解算。

/// ## Map: 數位通道上升沿 (Rising) 與下降沿 (Falling) 邊緣檢測
/// 回傳 (是否為上升沿, 是否為下降沿)
/// @outputs: 2
/// @domain: channel_bit [0, 7]
pub fn analyzer_digital_edge_detect(
    prev_sample: u8,
    curr_sample: u8,
    channel_bit: u8,
) -> (bool, bool) {
    if channel_bit > 7 {
        return (false, false);
    }
    let mask = 1u8 << channel_bit;
    let prev_bit = (prev_sample & mask) != 0;
    let curr_bit = (curr_sample & mask) != 0;
    let rising = !prev_bit && curr_bit;
    let falling = prev_bit && !curr_bit;
    (rising, falling)
}

/// ## Filter: 數位毛刺抑制濾波器步進 (多數決抗噪去抖動)
/// @outputs: 1
/// @domain: threshold_samples [1, 16]
pub fn analyzer_glitch_filter_sample(
    raw_bit: bool,
    filter_counter: &mut u8,
    threshold_samples: u8,
) -> bool {
    if threshold_samples == 0 {
        return raw_bit;
    }
    if raw_bit {
        if *filter_counter < threshold_samples {
            *filter_counter = filter_counter.saturating_add(1);
        }
    } else {
        if *filter_counter > 0 {
            *filter_counter = filter_counter.saturating_sub(1);
        }
    }
    *filter_counter >= (threshold_samples / 2 + 1)
}

/// ## Offset: UART 串列傳輸位元中心取樣點換算
/// 回傳 (每個位元取樣數, 半位元起始採樣中心點偏移)
/// @outputs: 2
/// @domain: baud_rate [300, 10000000]
/// @domain: sample_rate_hz [1000, 100000000]
pub fn analyzer_uart_sample_point_offset(
    baud_rate: u32,
    sample_rate_hz: u32,
) -> Option<(u32, u32)> {
    if baud_rate == 0 || sample_rate_hz == 0 || sample_rate_hz < baud_rate {
        return None;
    }
    let samples_per_bit = sample_rate_hz / baud_rate;
    if samples_per_bit < 2 {
        return None;
    }
    let center_offset = samples_per_bit / 2;
    Some((samples_per_bit, center_offset))
}

/// ## Parse: I2C 總線條件判定 (START, STOP, DATA, NONE)
/// 0 = NONE, 1 = START (SCL高電平時SDA下降), 2 = STOP (SCL高電平時SDA上升), 3 = DATA valid (SCL上升沿)
/// @outputs: 1
pub fn analyzer_i2c_bus_condition(
    scl_prev: bool,
    scl_curr: bool,
    sda_prev: bool,
    sda_curr: bool,
) -> u8 {
    // START: SCL 為高電平，且 SDA 從高變低
    if scl_prev && scl_curr && sda_prev && !sda_curr {
        return 1;
    }
    // STOP: SCL 為高電平，且 SDA 從低變高
    if scl_prev && scl_curr && !sda_prev && sda_curr {
        return 2;
    }
    // DATA valid: SCL 從低變高 (上升沿取樣資料)
    if !scl_prev && scl_curr {
        return 3;
    }
    0
}

/// ## Mode: SPI 模式 (CPOL/CPHA 0..3) 採樣邊緣與移位邊緣判定
/// 回傳 (是否在上升沿採樣, 是否在上升沿移位)
/// @outputs: 2
/// @domain: cpol [0, 1]
/// @domain: cpha [0, 1]
pub fn analyzer_spi_sample_edge(cpol: u8, cpha: u8) -> (bool, bool) {
    let pol = cpol & 1;
    let pha = cpha & 1;
    // 當 (pol == 0 && pha == 0) 或 (pol == 1 && pha == 1) 時在第一個沿(上升沿)採樣
    let sample_on_rising = (pol ^ pha) == 0;
    let shift_on_rising = !sample_on_rising;
    (sample_on_rising, shift_on_rising)
}

/// ## Decode: Dallas 1-Wire 低電平脈衝寬度解碼 (單位: 微秒 us)
/// 低電平 1..15 us 判定為 1, 60..120 us 判定為 0, >480 us 判定為 RESET 脈衝
/// @outputs: 1
/// @domain: low_pulse_us [0.1, 10000.0]
pub fn analyzer_onewire_bit_decode(low_pulse_us: f32) -> Option<bool> {
    if !low_pulse_us.is_finite() || low_pulse_us <= 0.0 {
        return None;
    }
    if low_pulse_us >= 1.0 && low_pulse_us <= 15.0 {
        Some(true) // Bit 1
    } else if low_pulse_us >= 50.0 && low_pulse_us <= 130.0 {
        Some(false) // Bit 0
    } else {
        None // 無效或 Reset 脈衝
    }
}

/// ## Decode: Manchester 雙相碼 (Biphase) 週期中段跳變解碼
/// @outputs: 1
pub fn analyzer_manchester_decode_transition(
    prev_level: bool,
    curr_level: bool,
    is_ieee_802_3: bool,
) -> Option<bool> {
    if prev_level == curr_level {
        return None; // 無躍變
    }
    if is_ieee_802_3 {
        // IEEE 802.3 (以太網): Low -> High 為 '1', High -> Low 為 '0'
        Some(!prev_level && curr_level)
    } else {
        // G.E. Thomas: High -> Low 為 '1', Low -> High 為 '0'
        Some(prev_level && !curr_level)
    }
}

/// ## Compress: 邏輯分析儀取樣流行程長度編碼 (RLE) 壓縮步進
/// @outputs: 1
/// @domain: max_run [1, 65535]
pub fn analyzer_rle_compress_step(
    current_sample: u8,
    last_sample: &mut u8,
    run_length: &mut u32,
    max_run: u32,
) -> Option<(u8, u32)> {
    if max_run == 0 {
        return None;
    }
    if *run_length == 0 {
        *last_sample = current_sample;
        *run_length = 1;
        return None;
    }
    if current_sample == *last_sample && *run_length < max_run {
        *run_length += 1;
        None
    } else {
        let emit = (*last_sample, *run_length);
        *last_sample = current_sample;
        *run_length = 1;
        Some(emit)
    }
}

/// ## Trigger: 邏輯分析儀遮罩式樣式比對觸發 (Pattern Match Trigger)
/// (sample & mask) == (level & mask)
/// @outputs: 1
pub fn analyzer_pattern_trigger_match(sample: u8, pattern_mask: u8, pattern_level: u8) -> bool {
    (sample & pattern_mask) == (pattern_level & pattern_mask)
}

/// ## Calc: CAN 總線位元時間量子 (Time Quanta, Tq) 總數與採樣點比例
/// 回傳 (總 Tq 數, 採樣點百分比 [0.0, 1.0])
/// @outputs: 2
/// @domain: prop_seg [1, 8]
/// @domain: phase_seg1 [1, 8]
/// @domain: phase_seg2 [1, 8]
pub fn analyzer_can_bit_timing_sample_point(
    prop_seg: u8,
    phase_seg1: u8,
    phase_seg2: u8,
) -> (u8, f32) {
    let p = prop_seg.clamp(1, 8);
    let s1 = phase_seg1.clamp(1, 8);
    let s2 = phase_seg2.clamp(1, 8);
    // CAN bit = Sync_Seg(1) + Prop_Seg + Phase_Seg1 + Phase_Seg2
    let total_tq = 1 + p + s1 + s2;
    let sample_point_tq = 1 + p + s1;
    let ratio = (sample_point_tq as f32) / (total_tq as f32);
    (total_tq, ratio)
}
