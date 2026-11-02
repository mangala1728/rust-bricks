# 規格書：IIR Biquad 帶通濾波器 (IIR Biquad Bandpass Filter)

## 模組名稱
`dsp_biquad_bandpass`

## 業務目標與適用場景
在聲學特徵提取、超音波回波偵測與生物信號特定頻帶（如腦電 EEG $\alpha$ 波、$\beta$ 波）隔離中，帶通濾波器是基礎組件。
本模組提供零外部依賴、純 `std` 實作的雙二階 (Biquad) 固定裙帶增益帶通濾波器 (Constant 0 dB Peak Gain Bandpass Filter)。

## 09 號合約邊界規範

### 函式 1：計算帶通濾波器係數
- **函式簽名**：
  ```rust
  /// @outputs: 5
  /// @domain: sample_rate_hz [1.0..1000000.0]
  /// @domain: center_freq_hz [0.1..sample_rate_hz / 2.0]
  /// @domain: q_factor [0.01..1000.0]
  pub fn biquad_bandpass_coefficients(
      sample_rate_hz: f64,
      center_freq_hz: f64,
      q_factor: f64,
  ) -> Option<[f64; 5]>
  ```
- **演算法公式 (Audio EQ Cookbook - Constant 0 dB Peak Gain)**：
  - $\omega_0 = 2\pi \cdot \frac{f_0}{f_s}$
  - $\alpha = \frac{\sin(\omega_0)}{2Q}$
  - $b_0 = \alpha, \quad b_1 = 0.0, \quad b_2 = -\alpha$
  - $a_0 = 1.0 + \alpha, \quad a_1 = -2.0 \cos(\omega_0), \quad a_2 = 1.0 - \alpha$
  - 歸一化回傳：$[b_0/a_0, \; b_1/a_0, \; b_2/a_0, \; a_1/a_0, \; a_2/a_0]$
- **邊界防禦**：
  - 若 $f_0 \ge f_s / 2$、或任意參數非正或為 NaN/Inf，第一行立即回傳 `None`。

### 函式 2：單步狀態更新濾波 (Direct Form II Transposed)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: input [-1.0e9..1.0e9]
  pub fn biquad_bandpass_step(
      input: f64,
      state: &mut [f64; 2],
      coeffs: &[f64; 5],
  ) -> Option<f64>
  ```
- **運算過程**：
  - $y = \text{coeffs}[0] \cdot x + \text{state}[0]$
  - $\text{state}[0] = \text{coeffs}[1] \cdot x - \text{coeffs}[3] \cdot y + \text{state}[1]$
  - $\text{state}[1] = \text{coeffs}[2] \cdot x - \text{coeffs}[4] \cdot y$
  - 回傳 `Some(y)`。

## 黃金測試向量 (Golden Vectors)
- 採樣率 $1000.0\text{ Hz}$，中心頻率 $250.0\text{ Hz}$ ($f_0 / f_s = 0.25$，$\omega_0 = \pi/2$)，$Q = 1.0$：
  - $\sin(\pi/2) = 1.0, \quad \cos(\pi/2) = 0.0$
  - $\alpha = 1.0 / (2 \times 1.0) = 0.5$
  - $b_0 = 0.5, b_1 = 0.0, b_2 = -0.5$
  - $a_0 = 1.5, a_1 = 0.0, a_2 = 0.5$
  - 歸一化：$b_0/a_0 = 0.333333, b_1/a_0 = 0.0, b_2/a_0 = -0.333333, a_1/a_0 = 0.0, a_2/a_0 = 0.333333$
- 容差使用 `(val - golden).abs() < 1e-4`。
