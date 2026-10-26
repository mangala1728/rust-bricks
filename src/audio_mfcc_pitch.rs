//! # Audio MFCC & Pitch DSP Micro-Bricks (音訊語音特徵工程與基頻檢測微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 Hz/Mel 刻度互轉、三角梅爾濾波權重、DCT-II 倒頻譜係數、自相關基頻估算、
//! YIN 平方差分與歸一化、DRC 軟拐點壓縮增益、Q15 預加重、頻譜平坦度 (SFM)。

/// ## Map: 赫茲轉梅爾刻度 (Mel = 2595 * log10(1 + f / 700))
/// @outputs: 1
pub fn audio_hz_to_mel(hz: f32) -> f32 {
    if hz <= 0.0 || hz.is_nan() {
        return 0.0;
    }
    let ratio = 1.0 + hz / 700.0;
    2595.0 * ratio.log10()
}

/// ## Map: 梅爾刻度轉赫茲 (Hz = 700 * (10^(mel / 2595) - 1))
/// @outputs: 1
pub fn audio_mel_to_hz(mel: f32) -> f32 {
    if mel <= 0.0 || mel.is_nan() {
        return 0.0;
    }
    let exp_val = 10.0f32.powf(mel / 2595.0);
    700.0 * (exp_val - 1.0).max(0.0)
}

/// ## Map: 三角梅爾濾波器組響應權重計算
/// @outputs: 1
pub fn audio_mel_filterbank_weights(center_mel: f32, left_mel: f32, right_mel: f32, sample_mel: f32) -> f32 {
    if left_mel >= center_mel || center_mel >= right_mel || sample_mel.is_nan() {
        return 0.0;
    }
    if sample_mel < left_mel || sample_mel > right_mel {
        0.0
    } else if sample_mel <= center_mel {
        (sample_mel - left_mel) / (center_mel - left_mel)
    } else {
        (right_mel - sample_mel) / (right_mel - center_mel)
    }
}

/// ## Map: DCT-II 離散餘弦轉換單階倒頻譜係數求解
/// @outputs: 1
pub fn audio_dct_type2_step(energies: &[f32], k_coeff: usize, total_coeffs: usize) -> Option<f32> {
    let n_banks = energies.len();
    if n_banks == 0 || total_coeffs == 0 || k_coeff >= total_coeffs {
        return None;
    }
    let pi = 3.141592653589793f32;
    let factor = (pi * (k_coeff as f32)) / (n_banks as f32);
    let mut sum = 0.0f32;
    for (n, &e) in energies.iter().enumerate() {
        if e.is_nan() { continue; }
        let angle = factor * ((n as f32) + 0.5);
        let cos_val = angle.cos();
        sum += e * cos_val;
    }
    Some(sum)
}

/// ## Map: 16-bit PCM 自相關法基頻 (Pitch / F0) 估算
/// @domain: sample_rate_hz [4000, 192000]
/// @domain: min_pitch_hz [20.0, 500.0]
/// @domain: max_pitch_hz [50.0, 2000.0]
/// @outputs: 1
pub fn audio_autocorrelation_pitch_hz(
    pcm: &[i16],
    sample_rate_hz: u32,
    min_pitch_hz: f32,
    max_pitch_hz: f32,
) -> Option<f32> {
    if sample_rate_hz < 4000 || sample_rate_hz > 192000 || min_pitch_hz < 20.0 || max_pitch_hz > 2000.0 || min_pitch_hz >= max_pitch_hz {
        return None;
    }
    let fs = sample_rate_hz as f32;
    let min_lag = (fs / max_pitch_hz) as usize;
    let max_lag = (fs / min_pitch_hz) as usize;
    if pcm.len() <= max_lag || min_lag == 0 {
        return None;
    }
    let n = pcm.len() - max_lag;
    let mut best_corr = i64::MIN;
    let mut best_lag = 0usize;

    for lag in min_lag..=max_lag {
        let mut corr = 0i64;
        for i in 0..n {
            corr += (pcm[i] as i64) * (pcm[i + lag] as i64);
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }
    if best_lag == 0 || best_corr <= 0 {
        None
    } else {
        Some(fs / (best_lag as f32))
    }
}

/// ## Map: YIN 演算法一階平方差分計算 (Square Difference Function: d_t(tau))
/// @outputs: 1
pub fn audio_yin_difference_step(pcm: &[i16], tau: usize, window_len: usize) -> Option<f32> {
    if window_len == 0 || pcm.len() < window_len + tau {
        return None;
    }
    let mut sum_sq = 0.0f32;
    for j in 0..window_len {
        let diff = (pcm[j] as f32) - (pcm[j + tau] as f32);
        sum_sq += diff * diff;
    }
    Some(sum_sq)
}

/// ## Map: YIN 累積平均歸一化差分 (CMND: d'(tau) = d(tau) / ((1/tau) * sum(d(j))))
/// @outputs: 1
pub fn audio_yin_cumulative_mean_normalized_difference(d_tau: f32, sum_d: f32, tau: usize) -> f32 {
    if tau == 0 {
        return 1.0;
    }
    let denom = sum_d / (tau as f32);
    if denom <= 1e-12 || d_tau.is_nan() {
        1.0
    } else {
        d_tau / denom
    }
}

/// ## Map: 音訊動態範圍壓縮器 (DRC) 軟拐點增益特性曲線 (Soft-Knee Gain Characteristic in dB)
/// @domain: ratio [1.0, 100.0]
/// @domain: knee_width_db [0.0, 40.0]
/// @outputs: 1
pub fn audio_drc_compressor_gain_db(input_db: f32, threshold_db: f32, ratio: f32, knee_width_db: f32) -> f32 {
    if ratio < 1.0 || ratio > 100.0 || knee_width_db < 0.0 || knee_width_db > 40.0 || input_db.is_nan() {
        return input_db;
    }
    let half_knee = knee_width_db * 0.5;
    let slope = 1.0 - 1.0 / ratio;

    if input_db < threshold_db - half_knee {
        input_db
    } else if input_db <= threshold_db + half_knee && knee_width_db > 0.0 {
        let delta = input_db - threshold_db + half_knee;
        let excess = delta * delta / (2.0 * knee_width_db);
        input_db - slope * excess
    } else {
        let excess = input_db - threshold_db;
        input_db - slope * excess
    }
}

/// ## Map: 16-bit PCM 預加重高通濾波 (Q15 格式: y[n] = x[n] - (alpha * x[n-1]) >> 15)
/// @outputs: 1
pub fn audio_pre_emphasis_filter_step(curr_sample: i16, prev_sample: i16, alpha_q15: i16) -> i16 {
    let scaled_prev = ((prev_sample as i32) * (alpha_q15 as i32)) >> 15;
    let diff = (curr_sample as i32) - scaled_prev;
    diff.clamp(i16::MIN as i32, i16::MAX as i32) as i16
}

/// ## Map: 頻譜平坦度 (SFM: 幾何平均 / 算術平均，區分濁音與噪聲)
/// @outputs: 1
pub fn audio_spectral_flatness_measure(power_spectrum: &[f32]) -> Option<f32> {
    if power_spectrum.is_empty() {
        return None;
    }
    let mut n_valid = 0usize;
    let mut has_zero_bin = false;
    let mut sum_linear = 0.0f32;
    let mut sum_log = 0.0f32;

    for &p in power_spectrum {
        if p.is_nan() {
            continue;
        }
        n_valid += 1;
        if p <= 1e-12 {
            // 任一 bin 為 0 時幾何平均為 0，不可直接略過 (INC-20261026-05)
            has_zero_bin = true;
            continue;
        }
        sum_linear += p;
        sum_log += p.ln();
    }
    if n_valid == 0 {
        return None;
    }
    let n = n_valid as f32;
    let arith_mean = sum_linear / n;
    if arith_mean <= 1e-12 {
        return None;
    }
    if has_zero_bin {
        return Some(0.0);
    }
    let geom_mean = (sum_log / n).exp();
    Some((geom_mean / arith_mean).clamp(0.0, 1.0))
}


