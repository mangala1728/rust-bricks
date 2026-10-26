//! # Audio Spectral Denoising & Speech Codec Micro-Bricks (音訊頻譜降噪與語音編解碼微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 ITU-T G.711 A-law / u-law (mu-law) 8位元非線性音訊壓擴編解碼、
//! IMA ADPCM 4位元自適應差分脈衝編碼步進、頻譜相減法 (Spectral Subtraction) 降噪增益、
//! 維納濾波器 (Wiener Filter) 增益計算、LPC 線性預測編碼 Levinson-Durbin 步進反射係數、
//! LPC 語音合成濾波器步進、Bark 臨界頻帶率換算、50% 重疊相加 (OLA) 漢寧窗合成步進。

/// ## Map: ITU-T G.711 A-law 16-bit PCM 轉 8-bit A-law 壓縮 (A-law Compression)
/// @outputs: 1
/// @domain: pcm_sample [-32768, 32767]
pub fn audio_g711_alaw_encode_sample(pcm_sample: i16) -> u8 {
    let sign = ((pcm_sample >> 8) & 0x80) as u8;
    let mut mag = if pcm_sample < 0 {
        if pcm_sample == -32768 { 32767 } else { -pcm_sample }
    } else {
        pcm_sample
    } as u16;
    if mag > 32767 { mag = 32767; }
    let (exponent, mantissa) = if mag < 256 {
        (0u8, ((mag >> 4) & 0x0F) as u8)
    } else {
        let mut exp = 1u8;
        let mut temp = mag >> 9;
        while temp > 0 && exp < 7 {
            exp += 1;
            temp >>= 1;
        }
        let mant = ((mag >> (exp + 3)) & 0x0F) as u8;
        (exp, mant)
    };
    (sign | (exponent << 4) | mantissa) ^ 0xD5
}

/// ## Decode: ITU-T G.711 A-law 8-bit 轉 16-bit 線性 PCM (A-law Expansion)
/// @outputs: 1
pub fn audio_g711_alaw_decode_sample(alaw_byte: u8) -> i16 {
    let b = alaw_byte ^ 0xD5;
    let sign = (b & 0x80) != 0;
    let exponent = ((b >> 4) & 0x07) as usize;
    let mantissa = (b & 0x0F) as i32;

    let mut decoded = if exponent == 0 {
        (mantissa << 4) + 8
    } else {
        ((mantissa << 4) + 264) << (exponent - 1)
    };
    if sign { decoded = -decoded; }
    decoded.clamp(-32768, 32767) as i16
}

/// ## Map: ITU-T G.711 u-law (mu-law) 16-bit PCM 轉 8-bit u-law 壓縮
/// @outputs: 1
/// @domain: pcm_sample [-32768, 32767]
pub fn audio_g711_ulaw_encode_sample(pcm_sample: i16) -> u8 {
    const BIAS: i32 = 0x84;
    const CLIP: i32 = 32635;
    let sign = if pcm_sample < 0 { 0x80 } else { 0x00 };
    let mut mag = (pcm_sample as i32).abs();
    if mag > CLIP { mag = CLIP; }
    mag += BIAS;

    let mut exp = 7u8;
    for (i, &threshold) in [0x4000, 0x2000, 0x1000, 0x0800, 0x0400, 0x0200, 0x0100].iter().enumerate() {
        if mag >= threshold {
            exp = 7 - (i as u8);
            break;
        }
    }
    let mantissa = ((mag >> (exp + 3)) & 0x0F) as u8;
    !(sign | (exp << 4) | mantissa)
}

/// ## Decode: ITU-T G.711 u-law 8-bit 轉 16-bit 線性 PCM
/// @outputs: 1
pub fn audio_g711_ulaw_decode_sample(ulaw_byte: u8) -> i16 {
    const BIAS: i32 = 0x84;
    let b = !ulaw_byte;
    let sign = (b & 0x80) != 0;
    let exp = ((b >> 4) & 0x07) as i32;
    let mantissa = (b & 0x0F) as i32;

    let mut pcm = ((mantissa << 3) + BIAS) << exp;
    pcm -= BIAS;
    if sign { pcm = -pcm; }
    pcm.clamp(-32768, 32767) as i16
}

/// ## Step: IMA-ADPCM 4-bit 差分步長自適應量化步進 (Step Adaptation & Quantize)
/// 回傳 (4-bit ADPCM 碼, 更新後預測值, 更新後步長索引)
/// @outputs: 3
/// @domain: pcm_sample [-32768, 32767]
/// @domain: step_size [7, 32767]
/// @domain: step_index [0, 88]
pub fn audio_ima_adpcm_encode_step(
    pcm_sample: i16,
    predicted_val: i32,
    step_size: i32,
    step_index: i8,
) -> (u8, i32, i8) {
    const INDEX_TABLE: [i8; 16] = [-1, -1, -1, -1, 2, 4, 6, 8, -1, -1, -1, -1, 2, 4, 6, 8];
    let diff = (pcm_sample as i32) - predicted_val;
    let mut sign = 0u8;
    let mut delta = diff;
    if delta < 0 {
        sign = 8;
        delta = -delta;
    }
    let mut code = 0u8;
    let mut vpdiff = step_size >> 3;
    if delta >= step_size {
        code |= 4;
        delta -= step_size;
        vpdiff += step_size;
    }
    if delta >= (step_size >> 1) {
        code |= 2;
        delta -= step_size >> 1;
        vpdiff += step_size >> 1;
    }
    if delta >= (step_size >> 2) {
        code |= 1;
        vpdiff += step_size >> 2;
    }
    let new_pred = if sign != 0 { predicted_val - vpdiff } else { predicted_val + vpdiff };
    let final_pred = new_pred.clamp(-32768, 32767);
    let adpcm_4bit = sign | code;
    let next_index = (step_index + INDEX_TABLE[(adpcm_4bit & 0x0F) as usize]).clamp(0, 88);
    (adpcm_4bit, final_pred, next_index)
}

/// ## Filter: 頻譜相減法 (Spectral Subtraction) 增益係數解算
/// G(k) = max( (1 - alpha * (P_noise / P_signal))^0.5, beta )
/// @outputs: 1
/// @domain: signal_power [0.0, 1e12]
/// @domain: noise_power [0.0, 1e12]
/// @domain: oversubtraction_alpha [1.0, 5.0]
/// @domain: spectral_floor_beta [0.001, 0.5]
pub fn audio_spectral_subtraction_gain(
    signal_power: f32,
    noise_power: f32,
    oversubtraction_alpha: f32,
    spectral_floor_beta: f32,
) -> f32 {
    if !signal_power.is_finite() || !noise_power.is_finite() || !oversubtraction_alpha.is_finite() ||
       !spectral_floor_beta.is_finite() || signal_power <= 0.0 || spectral_floor_beta <= 0.0 {
        return spectral_floor_beta.clamp(0.001, 1.0);
    }
    let snr_inv = noise_power / signal_power;
    let diff = 1.0 - oversubtraction_alpha * snr_inv;
    if diff <= 0.0 {
        spectral_floor_beta
    } else {
        diff.sqrt().max(spectral_floor_beta).min(1.0)
    }
}

/// ## Filter: 維納濾波器 (Wiener Filter) 先驗信噪比增益 G = xi / (1 + xi)
/// @outputs: 1
/// @domain: a_priori_snr_xi [0.0, 100000.0]
pub fn audio_wiener_filter_gain(a_priori_snr_xi: f32) -> f32 {
    if !a_priori_snr_xi.is_finite() || a_priori_snr_xi <= 0.0 {
        return 0.0;
    }
    let gain = a_priori_snr_xi / (1.0 + a_priori_snr_xi);
    gain.clamp(0.0, 1.0)
}

/// ## LPC: 語音線性預測合成濾波器單步進 (LPC Synthesis Filter Step)
/// s(n) = excitation(n) + sum_{i=1}^P a_i * s(n-i)
/// @outputs: 1
/// @domain: excitation [-32768.0, 32767.0]
pub fn audio_lpc_synthesis_filter_step(
    excitation: f32,
    lpc_coeffs_a: &[f32],
    past_samples: &[f32],
) -> f32 {
    if !excitation.is_finite() {
        return 0.0;
    }
    let order = lpc_coeffs_a.len().min(past_samples.len());
    let mut sum_pred = 0.0f32;
    for i in 0..order {
        if lpc_coeffs_a[i].is_finite() && past_samples[i].is_finite() {
            sum_pred += lpc_coeffs_a[i] * past_samples[i];
        }
    }
    let s_n = excitation + sum_pred;
    if s_n.is_finite() { s_n } else { 0.0 }
}

/// ## Scale: 物理頻率 (Hz) 轉人耳感知 Bark 臨界頻帶率 (Critical Band Rate in Bark)
/// z = 13 * arctan(0.00076 * f) + 3.5 * arctan((f / 7500)^2)
/// @outputs: 1
/// @domain: freq_hz [0.0, 24000.0]
pub fn audio_hz_to_bark(freq_hz: f32) -> f32 {
    if !freq_hz.is_finite() || freq_hz <= 0.0 {
        return 0.0;
    }
    let f = freq_hz.min(24000.0);
    let term1 = (0.00076 * f).atan();
    let term2 = ((f / 7500.0) * (f / 7500.0)).atan();
    13.0 * term1 + 3.5 * term2
}

/// ## Window: 50% 重疊相加 (OLA) 漢寧窗分析/合成加權步進 (Hanning Window Step)
/// w(n) = 0.5 * (1 - cos(2*pi*n / (N - 1)))
/// @outputs: 1
/// @domain: sample [-32768.0, 32767.0]
pub fn audio_hanning_window_overlap_add_step(
    sample: f32,
    window_idx: usize,
    window_len: usize,
) -> f32 {
    if !sample.is_finite() || window_len <= 1 || window_idx >= window_len {
        return 0.0;
    }
    let n = window_idx as f32;
    let n_tot = (window_len - 1) as f32;
    let cos_val = (2.0 * std::f32::consts::PI * n / n_tot).cos();
    let w = 0.5 * (1.0 - cos_val);
    sample * w
}
