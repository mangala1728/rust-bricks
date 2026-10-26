// ## Map
// Role: dsp/radar_biomed_dsp — 毫米波雷達生醫呼吸信號解算、MIMO 正交相位編碼 (CDMA/TDMA)、FMCW 測距與多位元量化 (Q7/Q31) 微積木。
// Inputs: 頻寬 (Hz)、拍頻 (Hz)、掃頻時間 (s)、相位 (rad)、IIR 濾波器狀態、浮點數與定點數採樣。
// Outputs: 距離解析度 (m)、目標距離 (m)、解纏繞相位、生醫呼吸位移、Q7/Q31 定點與浮點量化數值。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: tx_index [0..64]
/// @domain: chip_index [0..64]
/// @domain: code_len [1..64]
pub fn radar_hadamard_cdma_code(tx_index: usize, chip_index: usize, code_len: usize) -> Option<i8> {
    if code_len == 0 || code_len > 64 || (code_len & (code_len - 1)) != 0
        || tx_index >= code_len || chip_index >= code_len {
        return None;
    }
    // Sylvester Walsh-Hadamard：H(i, j) = (-1)^(popcount(i AND j)) (INC-20261026-02)
    let sum_bits = (tx_index & chip_index).count_ones();
    if sum_bits % 2 == 0 {
        Some(1)
    } else {
        Some(-1)
    }
}

/// @outputs: 1
/// @domain: total_tx [1..16]
pub fn radar_tdma_tx_burst_active(tx_id: usize, total_tx: usize, chirp_seq: usize) -> bool {
    if total_tx == 0 || tx_id >= total_tx {
        return false;
    }
    (chirp_seq % total_tx) == tx_id
}

/// @outputs: 1
/// @domain: bandwidth_hz [1.0e6..10.0e9]
pub fn radar_fmcw_range_resolution_meters(bandwidth_hz: f64) -> Option<f64> {
    if bandwidth_hz <= 0.0 || !bandwidth_hz.is_finite() {
        return None;
    }
    let c = 299_792_458.0; // 光速 m/s
    Some(c / (2.0 * bandwidth_hz))
}

/// @outputs: 1
/// @domain: beat_freq_hz [0.0..50.0e6]
/// @domain: chirp_time_s [1.0e-6..1.0]
/// @domain: bandwidth_hz [1.0e6..10.0e9]
pub fn radar_fmcw_beat_frequency_to_range(
    beat_freq_hz: f64,
    chirp_time_s: f64,
    bandwidth_hz: f64,
) -> Option<f64> {
    if beat_freq_hz < 0.0 || chirp_time_s <= 0.0 || bandwidth_hz <= 0.0
        || !beat_freq_hz.is_finite() || !chirp_time_s.is_finite() || !bandwidth_hz.is_finite() {
        return None;
    }
    let c = 299_792_458.0;
    let range = (c * chirp_time_s * beat_freq_hz) / (2.0 * bandwidth_hz);
    Some(range)
}

/// @outputs: 1
pub fn radar_phase_unwrap_step(current_phase_rad: f32, prev_phase_rad: f32) -> f32 {
    if !current_phase_rad.is_finite() || !prev_phase_rad.is_finite() {
        return 0.0;
    }
    let pi = std::f32::consts::PI;
    let two_pi = 2.0 * pi;
    let mut diff = current_phase_rad - prev_phase_rad;
    while diff > pi {
        diff -= two_pi;
    }
    while diff < -pi {
        diff += two_pi;
    }
    prev_phase_rad + diff
}

/// @outputs: 1
/// @domain: alpha_hp [0.0..1.0]
/// @domain: alpha_lp [0.0..1.0]
pub fn radar_respiration_bandpass_step(
    sample: f32,
    state: &mut [f32; 2],
    alpha_hp: f32,
    alpha_lp: f32,
) -> f32 {
    if !sample.is_finite() || alpha_hp < 0.0 || alpha_hp > 1.0 || alpha_lp < 0.0 || alpha_lp > 1.0 {
        return 0.0;
    }
    // state[0]: 高通濾波前一刻輸入，state[1]: 低通濾波累積器
    state[0] += (1.0 - alpha_hp) * (sample - state[0]);
    let hp_out = sample - state[0];
    state[1] += alpha_lp * (hp_out - state[1]);
    state[1]
}

/// @outputs: 1
pub fn dsp_quantize_f32_to_q7(sample: f32) -> i8 {
    if !sample.is_finite() {
        return 0;
    }
    let scaled = (sample * 127.0).round();
    scaled.clamp(-128.0, 127.0) as i8
}

/// @outputs: 1
pub fn dsp_dequantize_q7_to_f32(sample: i8) -> f32 {
    (sample as f32) / 127.0
}

/// @outputs: 1
pub fn dsp_quantize_f64_to_q31(sample: f64) -> i32 {
    if !sample.is_finite() {
        return 0;
    }
    let scaled = (sample * 2147483647.0).round();
    scaled.clamp(-2147483648.0, 2147483647.0) as i32
}

/// @outputs: 1
pub fn dsp_dequantize_q31_to_f64(sample: i32) -> f64 {
    (sample as f64) / 2147483647.0
}
