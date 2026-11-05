# 零依賴 STFT 重疊相加 (OLA) 即時頻譜減法音訊淨化微積木

wish_id: WISH-20261105-006
requester: ai-interpreter-core-squad
status: PENDING
priority: HIGH
created_at: 2026-11-05

## 1. 需求背景與場景（Context）
- 使用專案：雙向即時語音口譯 Android App（`ai-interpreter`）端側音訊 DSP 引擎。
- 遇到瓶頸：
  在 Stage 2c 真實物理環境壓力測試中（SNR -10dB ~ +20dB、高鐵與捷運車廂鐵軌撞擊聲、印度德里街頭嘟嘟車喇叭），口譯機需要即時抑制強背景噪聲。
  雖然 `rust-bricks` 中已有 `dsp_fft_radix2` 與 `audio_dsp::compute_spectral_subtraction_gain`，但下游口譯機若要在 Android AudioRecord 錄音鏈路中即時調用，仍需自行拼接「Hann 窗加權 -> 512-point FFT -> 增益遮罩乘算 -> IFFT 逆變換 -> 50% 重疊相加（OLA）緩衝區管理 -> 軟削峰輸出」。
  缺乏一個一體化、零堆分配（Zero-heap allocation）、符合 09 合約規範的時域 PCM16 端到端降噪微積木。

## 2. 演算法本質（Algorithm Principles）
1. **短時分幀與 Hann 加窗**：
   輸入 256 取樣點（16kHz 下的 16ms 語音幀），結合前一幀後半段組成 512 點分析視窗並乘上 Hann 窗函式 $w(n) = 0.5 \cdot (1 - \cos(2\pi n / N))$。
2. **頻譜增益遮罩施加**：
   藉由 Radix-2 FFT 將訊號投影至頻域，計算幅度譜後與前置取樣的環境噪聲譜底（Noise Profile）進行過減計算：
   $$G(k) = \max\left(\gamma, 1.0 - \beta \cdot \frac{P_{\text{noise}}(k)}{P_{\text{frame}}(k)}\right)$$
3. **頻譜重構與重疊相加 (Overlap-Add, OLA)**：
   經 IFFT 逆變換回時域後，與 `overlap_buffer` 執行重疊相加，消除幀邊界不連續所導致的「吉布斯效應（Gibbs ringing）」與音樂噪聲（Musical noise）。
4. **Q15 軟削峰防止破音**：
   將浮點訊號限制在 $[-32767, 32767]$，防止飽和溢位。

## 3. 期望的 Rust 微積木簽章（Target Signature）
```rust
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
) -> usize;
```

## 4. 效益與驗收指標（ROI）
- **延遲指標**：單幀 256 取樣點在 ARM64（如 Snapdragon 835 舊機）上的計算延遲必須 $< 50\,\mu\text{s}$，確保即時音訊流零卡頓。
- **記憶體指標**：0 Heap Allocation，所有暫存運算均在棧上（Stack）完成。
- **降噪效果**：在低 SNR 噪聲（高鐵鐵軌撞擊、街頭喇叭）下至少降低底噪 12dB，顯著提升大模型語音聽寫命中率。
