# 規格書：零依賴 STFT 重疊相加 (OLA) 即時頻譜減法音訊淨化微積木 (dsp_stft_ola_denoiser)

- 目標模組檔: `src/dsp_stft_ola_denoiser.rs`
- 目標測試檔: `tests/test_dsp_stft_ola_denoiser.rs`
- 來源需求: `wishes/WISH-20261105-006-stft-overlap-add-denoiser.md`

## 模組職責
提供 0 堆分配、符合 09 合約、NASA JPL 60 行規範的端側即時時域音訊 STFT 頻譜減法降噪與 50% 重疊相加 (OLA) 平滑合成引擎。

## 函式簽章
```rust
pub fn spectral_subtraction_gain_single(frame_pwr: f32, noise_pwr: f32, oversubtraction: f32, spectral_floor: f32) -> f32;
pub fn estimate_noise_power_profile(noise_pcm16: &[i16], noise_profile_out: &mut [f32; 256]) -> usize;
pub fn stft_spectral_denoise_frame(
    input_pcm16: &[i16; 256],
    overlap_buffer: &mut [f32; 256],
    noise_profile: &[f32; 256],
    oversubtraction: f32,
    spectral_floor: f32,
    output_pcm16: &mut [i16; 256],
) -> usize;
```

## 機器可驗證黃金向量（```golden）
```golden
spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.05) == 0.8
spectral_subtraction_gain_single(10.0, 10.0, 2.0, 0.05) == 0.05
spectral_subtraction_gain_single(0.0, 10.0, 2.0, 0.05) == 0.05
spectral_subtraction_gain_single(100.0, 0.0, 2.0, 0.05) == 1.0
spectral_subtraction_gain_single(100.0, 10.0, 0.5, 0.05) == 1.0
spectral_subtraction_gain_single(100.0, 10.0, 2.0, 0.6) == 1.0
```
