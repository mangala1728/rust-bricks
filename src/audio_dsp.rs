// ## Map
// Role: core/audio_dsp — 數位訊號處理：正規化聲學功率、dBFS 能量轉換、信噪比 (SNR) 扣除運算、零交越率 (ZCR)、雙二階 Butterworth 高通濾波係數、Biquad 濾波執行與目標 SNR 雜訊縮放因數。
// Inputs: PCM16 取樣切片、正規化線性功率、噪聲功率、目標信噪比 dB、截止頻率與取樣率、Biquad 係數 [b0, b1, b2, a1, a2]。
// Outputs: 0.0..=1.0 功率、-100.0..=0.0 dBFS 能量、0.0..=100.0 SNR dB、0.0..=1.0 零交越率、Biquad 係數陣列 [b0, b1, b2, a1, a2]、濾波後 PCM16 切片、雜訊縮放乘數。
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
pub fn biquad_filter_apply(samples: &[i16], coeffs: [f64; 5]) -> Vec<i16> {
    if samples.is_empty() || samples.len() > MAX_AUDIO_SAMPLES {
        return Vec::new();
    }
    let [b0, b1, b2, a1, a2] = coeffs;
    if b0.is_nan() || b1.is_nan() || b2.is_nan() || a1.is_nan() || a2.is_nan() {
        return samples.to_vec();
    }

    let mut out = Vec::with_capacity(samples.len());
    let mut x1 = 0.0f64;
    let mut x2 = 0.0f64;
    let mut y1 = 0.0f64;
    let mut y2 = 0.0f64;

    for &s in samples {
        let x0 = s as f64;
        let y0 = b0 * x0 + b1 * x1 + b2 * x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = x0;
        y2 = y1;
        y1 = y0;

        let clamped = if y0 > 32767.0 {
            32767
        } else if y0 < -32768.0 {
            -32768
        } else {
            y0.round() as i16
        };
        out.push(clamped);
    }
    out
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



/// @outputs: 1
/// @domain: alpha [0.0..1.0]
pub fn update_noise_spectrum(noise_profile: &mut [f32; 256], frame_mag: &[f32; 256], alpha: f32) {
    if alpha < 0.0 || alpha > 1.0 || !alpha.is_finite() {
        return;
    }
    let one_minus_alpha = 1.0 - alpha;
    for i in 0..256 {
        if frame_mag[i].is_finite() && frame_mag[i] >= 0.0 {
            noise_profile[i] = alpha * noise_profile[i] + one_minus_alpha * frame_mag[i];
        }
    }
}

/// @outputs: 1
/// @domain: oversubtraction [1.0..10.0]
/// @domain: spectral_floor [0.001..0.5]
pub fn compute_spectral_subtraction_gain(
    frame_mag: &[f32; 256],
    noise_profile: &[f32; 256],
    oversubtraction: f32,
    spectral_floor: f32,
    gain_out: &mut [f32; 256],
) {
    if oversubtraction < 1.0 || spectral_floor <= 0.0 || spectral_floor > 0.5 || !oversubtraction.is_finite() {
        for g in gain_out.iter_mut() {
            *g = 1.0;
        }
        return;
    }
    for i in 0..256 {
        let f = frame_mag[i];
        let n = noise_profile[i];
        if f <= 1e-9 || !f.is_finite() || !n.is_finite() {
            gain_out[i] = spectral_floor;
        } else {
            let snr_sub = 1.0 - oversubtraction * (n / f);
            gain_out[i] = snr_sub.max(spectral_floor).min(1.0);
        }
    }
}

/// @outputs: 1
pub fn downsample_3to1_pcm16(input_48k: &[i16], output_16k: &mut [i16]) -> usize {
    if input_48k.len() < 3 || output_16k.is_empty() {
        return 0;
    }
    let max_out = (input_48k.len() / 3).min(output_16k.len());
    for i in 0..max_out {
        let center = i * 3 + 1;
        let prev = input_48k[center - 1] as i32;
        let mid = input_48k[center] as i32;
        let next = if center + 1 < input_48k.len() { input_48k[center + 1] as i32 } else { mid };
        let filtered = (prev + 2 * mid + next) / 4;
        output_16k[i] = filtered.clamp(-32768, 32767) as i16;
    }
    max_out
}

/// @outputs: 1
/// @domain: target_dbfs [-60.0..0.0]
/// @domain: max_gain [1.0..100.0]
pub fn agc_apply_gain_pcm16(samples: &mut [i16], target_dbfs: f32, max_gain: f32) {
    if samples.is_empty() || target_dbfs < -60.0 || target_dbfs > 0.0 || max_gain < 1.0 || !max_gain.is_finite() {
        return;
    }
    let mut sum_sq = 0.0f64;
    for &s in samples.iter() {
        sum_sq += (s as f64) * (s as f64);
    }
    let mean_sq = sum_sq / (samples.len() as f64);
    let rms = mean_sq.sqrt();
    if rms < 1.0 {
        return;
    }
    let current_dbfs = (20.0 * (rms / 32768.0).log10()) as f32;
    if current_dbfs >= target_dbfs {
        return;
    }
    let gain_needed = 10.0f32.powf((target_dbfs - current_dbfs) / 20.0);
    let gain = gain_needed.min(max_gain);

    for s in samples.iter_mut() {
        let amplified = (*s as f32) * gain;
        *s = soft_clip_i16(amplified as i32, 30000);
    }
}
