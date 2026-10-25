// ## Map
// Role: core/audio_vad — 音訊 PCM16 方均根能量 (RMS)、靜音門檻判定與字幕每秒字速 (CPS) 計算。
// Inputs: PCM16 取樣切片、靜音閾值、字幕字元數與毫秒持續時間。
// Outputs: RMS 能量值 (u32)、靜音布林判斷、每秒字速浮點數。
// Invariants: 零全域可變狀態、無遞迴、除數防零守衛、有界切片長度（最大 192,000 取樣點）。

/// @outputs: 2
pub fn pcm16_rms_energy(samples: &[i16]) -> u32 {
    if samples.is_empty() || samples.len() > 192_000 {
        return 0;
    }
    let mut sum_sq: u64 = 0;
    for &sample in samples {
        let val = sample as i64;
        sum_sq += (val * val) as u64;
    }
    let mean_sq = sum_sq / (samples.len() as u64);
    (mean_sq as f64).sqrt() as u32
}

/// @outputs: 2
/// @domain: rms [0..32768]
/// @domain: threshold [0..32768]
pub fn is_silence_frame(rms: u32, threshold: u32) -> bool {
    if rms > 32_768 || threshold > 32_768 {
        return true;
    }
    rms <= threshold
}

/// @outputs: 2
/// @domain: char_count [0..1000]
/// @domain: duration_ms [1..3600000]
pub fn cps_chars_per_second(char_count: u32, duration_ms: u32) -> Option<f64> {
    if char_count > 1000 || duration_ms == 0 || duration_ms > 3_600_000 {
        return None;
    }
    let sec = (duration_ms as f64) / 1000.0;
    Some((char_count as f64) / sec)
}
