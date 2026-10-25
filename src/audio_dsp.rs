// ## Map
// Role: core/audio_dsp — 數位訊號處理：正規化聲學功率、dBFS 能量轉換、信噪比 (SNR) 扣除運算、零交越率 (ZCR)、雙二階 Butterworth 高通濾波係數與目標 SNR 雜訊縮放因數。
// Inputs: PCM16 取樣切片、正規化線性功率、噪聲功率、目標信噪比 dB、截止頻率與取樣率。
// Outputs: 0.0..=1.0 功率、-100.0..=0.0 dBFS 能量、0.0..=100.0 SNR dB、0.0..=1.0 零交越率、Biquad 係數陣列 [b0, b1, b2, a1, a2]、雜訊縮放乘數。
// Invariants: 零全域可變狀態、無遞迴、除數防零守衛、有界取樣切片（最大 192,000 取樣點）、純標準庫 100% 零依賴。

const SILENCE_DBFS: f64 = -100.0;
const MAX_SNR_DB: f64 = 100.0;
const FULL_SCALE_SQUARED: f64 = 32_768.0 * 32_768.0;
const MIN_SIGNAL_RATIO: f64 = 1.0e-3;
const DECIBEL_FACTOR: f64 = 10.0;
const MAX_AUDIO_SAMPLES: usize = 192_000;

/// @outputs: 2
pub fn pcm16_frame_power(samples: &[i16]) -> f64 {
    if samples.is_empty() || samples.len() > MAX_AUDIO_SAMPLES {
        return 0.0;
    }
    let mut sum_sq = 0.0f64;
    for &sample in samples {
        let v = sample as f64;
        sum_sq += v * v;
    }
    let power = sum_sq / (samples.len() as f64) / FULL_SCALE_SQUARED;
    if power < 0.0 {
        0.0
    } else if power > 1.0 {
        1.0
    } else {
        power
    }
}

/// @outputs: 2
/// @domain: power [0.0..1.0]
pub fn energy_dbfs_from_power(power: f64) -> f64 {
    if power <= 0.0 || power.is_nan() {
        return SILENCE_DBFS;
    }
    let p = if power > 1.0 { 1.0 } else { power };
    let db = DECIBEL_FACTOR * p.log10();
    if db < SILENCE_DBFS {
        SILENCE_DBFS
    } else if db > 0.0 {
        0.0
    } else {
        db
    }
}

/// @outputs: 2
/// @domain: speech_power [0.0..1.0]
/// @domain: quiet_power [0.0..1.0]
pub fn snr_subtracted_db(speech_power: f64, quiet_power: f64) -> f64 {
    if speech_power.is_nan() || quiet_power.is_nan() {
        return 0.0;
    }
    let s = if speech_power < 0.0 { 0.0 } else if speech_power > 1.0 { 1.0 } else { speech_power };
    let q = if quiet_power < 0.0 { 0.0 } else if quiet_power > 1.0 { 1.0 } else { quiet_power };

    if q <= 0.0 {
        return MAX_SNR_DB;
    }
    let min_signal = q * MIN_SIGNAL_RATIO;
    let raw_diff = s - q;
    let signal = if raw_diff > min_signal { raw_diff } else { min_signal };
    let snr = DECIBEL_FACTOR * (signal / q).log10();
    if snr > MAX_SNR_DB {
        MAX_SNR_DB
    } else if snr < 0.0 {
        0.0
    } else {
        snr
    }
}

/// @outputs: 2
pub fn pcm16_zero_crossing_rate(samples: &[i16]) -> f64 {
    if samples.len() < 2 || samples.len() > MAX_AUDIO_SAMPLES {
        return 0.0;
    }
    let mut crossings = 0usize;
    for i in 1..samples.len() {
        if (samples[i] < 0) != (samples[i - 1] < 0) {
            crossings += 1;
        }
    }
    crossings as f64 / (samples.len() - 1) as f64
}

/// @outputs: 2
/// @domain: cutoff_hz [1..23999]
/// @domain: sample_rate [8000..48000]
pub fn biquad_butterworth_highpass_coeffs(cutoff_hz: u32, sample_rate: u32) -> Option<[f64; 5]> {
    if cutoff_hz == 0 || sample_rate < 8000 || sample_rate > 48000 {
        return None;
    }
    if cutoff_hz >= sample_rate / 2 {
        return None;
    }
    let q = 1.0 / (2.0f64).sqrt();
    let omega = 2.0 * std::f64::consts::PI * (cutoff_hz as f64) / (sample_rate as f64);
    let cosine = omega.cos();
    let alpha = omega.sin() / (2.0 * q);
    let a0 = 1.0 + alpha;

    let b0 = (1.0 + cosine) / 2.0 / a0;
    let b1 = -(1.0 + cosine) / a0;
    let b2 = (1.0 + cosine) / 2.0 / a0;
    let a1 = -2.0 * cosine / a0;
    let a2 = (1.0 - alpha) / a0;

    Some([b0, b1, b2, a1, a2])
}

/// @outputs: 2
/// @domain: speech_power [0.0..1.0]
/// @domain: noise_power [0.0..1.0]
/// @domain: target_snr_db [-50.0..100.0]
pub fn mix_noise_scale_factor(speech_power: f64, noise_power: f64, target_snr_db: f64) -> f64 {
    if speech_power <= 0.0 || noise_power <= 0.0 || target_snr_db < -50.0 || target_snr_db > 100.0 {
        return 0.0;
    }
    if speech_power.is_nan() || noise_power.is_nan() || target_snr_db.is_nan() {
        return 0.0;
    }
    let ratio = 10.0f64.powf(target_snr_db / 10.0);
    let target_noise_power = speech_power / ratio;
    (target_noise_power / noise_power).sqrt()
}

/// @outputs: 2
/// @domain: alpha [0.0..1.0]
pub fn pre_emphasis_filter(samples: &[i16], alpha: f32) -> Vec<i16> {
    if alpha < 0.0 || alpha > 1.0 || samples.is_empty() || samples.len() > MAX_AUDIO_SAMPLES {
        return samples.to_vec();
    }
    let mut out = Vec::with_capacity(samples.len());
    out.push(samples[0]);
    for i in 1..samples.len() {
        let cur = samples[i] as f32;
        let prev = samples[i - 1] as f32;
        let val = cur - alpha * prev;
        let clamped = if val > 32767.0 {
            32767
        } else if val < -32768.0 {
            -32768
        } else {
            val as i16
        };
        out.push(clamped);
    }
    out
}

/// @outputs: 2
/// @domain: threshold [1000..32767]
pub fn soft_clip_i16(sample: i32, threshold: i32) -> i16 {
    if threshold < 1000 || threshold > 32767 {
        return if sample > 32767 { 32767 } else if sample < -32768 { -32768 } else { sample as i16 };
    }
    if sample.abs() <= threshold {
        return sample as i16;
    }
    let sign = if sample < 0 { -1 } else { 1 };
    let abs_val = sample.abs();
    let excess = abs_val - threshold;
    let compressed = threshold + (excess as f32 * 0.5) as i32;
    let final_val = if compressed > 32767 { 32767 } else { compressed };
    (sign * final_val) as i16
}

