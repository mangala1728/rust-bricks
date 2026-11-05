// ## Map
// Role: dsp/dsp_stft_ola_denoiser — 零依賴 STFT 重疊相加 (OLA) 即時頻譜減法音訊淨化微積木。
// Inputs: input_pcm16: &[i16; 256], overlap_buffer: &mut [f32; 256], noise_profile: &[f32; 256], oversubtraction: f32, spectral_floor: f32
// Outputs: 寫入 output_pcm16: &mut [i16; 256] 並回傳處理取樣數 usize (256 或 0)
// Invariants: 零外部依賴、零堆分配 (Zero Heap Allocation)、函式行數 <= 60、無遞迴、零全域可變狀態。

use crate::dsp_fft_radix2::{fft_radix2_f32_inplace, ifft_radix2_f32_inplace};

/// @outputs: 1
/// @domain: oversubtraction [1.0..10.0]
/// @domain: spectral_floor [0.001..0.5]
pub fn spectral_subtraction_gain_single(
    frame_pwr: f32,
    noise_pwr: f32,
    oversubtraction: f32,
    spectral_floor: f32,
) -> f32 {
    if oversubtraction < 1.0
        || oversubtraction > 10.0
        || spectral_floor < 0.001
        || spectral_floor > 0.5
        || !oversubtraction.is_finite()
        || !spectral_floor.is_finite()
        || !frame_pwr.is_finite()
        || !noise_pwr.is_finite()
    {
        return 1.0;
    }
    if noise_pwr <= 1e-12 {
        return 1.0;
    }
    if frame_pwr <= 1e-12 {
        return spectral_floor;
    }
    let sub = 1.0 - oversubtraction * (noise_pwr / frame_pwr);
    sub.max(spectral_floor).min(1.0)
}

/// @outputs: 1
/// @domain: noise_pcm16.len() [256..65536]
pub fn estimate_noise_power_profile(
    noise_pcm16: &[i16],
    noise_profile_out: &mut [f32; 256],
) -> usize {
    if noise_pcm16.len() < 256 || noise_pcm16.len() > 65536 {
        return 0;
    }
    for p in noise_profile_out.iter_mut() {
        *p = 0.0;
    }
    let chunks = noise_pcm16.len() / 256;
    let mut real = [0.0f32; 512];
    let mut imag = [0.0f32; 512];

    for c in 0..chunks {
        let offset = c * 256;
        for i in 0..256 {
            real[i] = noise_pcm16[offset + i] as f32;
            real[i + 256] = 0.0;
            imag[i] = 0.0;
            imag[i + 256] = 0.0;
        }
        if !fft_radix2_f32_inplace(&mut real, &mut imag) {
            return 0;
        }
        for k in 0..256 {
            noise_profile_out[k] += real[k] * real[k] + imag[k] * imag[k];
        }
    }
    let scale = 1.0 / (chunks as f32);
    for p in noise_profile_out.iter_mut() {
        *p *= scale;
    }
    chunks * 256
}

/// @outputs: 1
/// @domain: oversubtraction [1.0..10.0]
/// @domain: spectral_floor [0.001..0.5]
pub fn stft_spectral_denoise_frame(
    input_pcm16: &[i16; 256],
    overlap_buffer: &mut [f32; 256],
    noise_profile: &[f32; 256],
    oversubtraction: f32,
    spectral_floor: f32,
    output_pcm16: &mut [i16; 256],
) -> usize {
    if oversubtraction < 1.0
        || oversubtraction > 10.0
        || spectral_floor < 0.001
        || spectral_floor > 0.5
        || !oversubtraction.is_finite()
        || !spectral_floor.is_finite()
    {
        return 0;
    }
    let mut real = [0.0f32; 512];
    let mut imag = [0.0f32; 512];
    for i in 0..256 {
        real[i] = input_pcm16[i] as f32;
    }
    if !fft_radix2_f32_inplace(&mut real, &mut imag) {
        return 0;
    }
    for k in 0..256 {
        let p_frame = real[k] * real[k] + imag[k] * imag[k];
        let p_noise = noise_profile[k];
        let g = spectral_subtraction_gain_single(p_frame, p_noise, oversubtraction, spectral_floor);
        real[k] *= g;
        imag[k] *= g;
        if k > 0 {
            real[512 - k] *= g;
            imag[512 - k] *= g;
        }
    }
    let p_nyq = real[256] * real[256] + imag[256] * imag[256];
    let g_nyq = spectral_subtraction_gain_single(p_nyq, noise_profile[255], oversubtraction, spectral_floor);
    real[256] *= g_nyq;
    imag[256] *= g_nyq;
    if !ifft_radix2_f32_inplace(&mut real, &mut imag) {
        return 0;
    }
    for i in 0..256 {
        let s = real[i] + overlap_buffer[i];
        output_pcm16[i] = s.clamp(-32767.0, 32767.0).round() as i16;
        overlap_buffer[i] = real[i + 256];
    }
    256
}
