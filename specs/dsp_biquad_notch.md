# 規格書：IIR Biquad 陷波濾波器 (IIR Biquad Notch Filter)

## 模組名稱
`dsp_biquad_notch`

## 業務目標與適用場景
在生理訊號量測 (ECG/EMG) 或精密高頻聲學採樣中，市電工頻（50Hz 或 60Hz）的電磁耦合會產生顯著的弦波雜訊。
本模組提供零外部依賴、純 `std` 實作的雙二階 (Biquad) 陷波濾波器（Notch Filter），支援高 Q 值窄帶衰減與 Transposed Direct Form II 步進濾波。

## 09 號合約邊界規範

### 函式 1：計算陷波濾波器係數
- **函式簽名**：
  ```rust
  /// @outputs: 5
  /// @domain: sample_rate_hz [1.0..1000000.0]
  /// @domain: center_freq_hz [0.1..sample_rate_hz / 2.0]
  /// @domain: q_factor [0.01..1000.0]
  pub fn biquad_notch_coefficients(
      sample_rate_hz: f64,
      center_freq_hz: f64,
      q_factor: f64,
  ) -> Option<[f64; 5]>
  ```
- **演算法公式 (Audio EQ Cookbook)**：
  - $\omega_0 = 2\pi \cdot \frac{f_0}{f_s}$
  - $\alpha = \frac{\sin(\omega_0)}{2Q}$
  - $b_0 = 1.0, \quad b_1 = -2.0 \cos(\omega_0), \quad b_2 = 1.0$
  - $a_0 = 1.0 + \alpha, \quad a_1 = -2.0 \cos(\omega_0), \quad a_2 = 1.0 - \alpha$
  - 歸一化回傳：$[b_0/a_0, \; b_1/a_0, \; b_2/a_0, \; a_1/a_0, \; a_2/a_0]$
- **邊界防禦**：
  - 若 $f_0 \ge f_s / 2$、或任意參數小於等於 0 或為 NaN/Inf，第一行立即回傳 `None`。

### 函式 2：單步狀態更新濾波 (Direct Form II Transposed)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: input [-1.0e9..1.0e9]
  pub fn biquad_notch_step(
      input: f64,
      state: &mut [f64; 2],
      coeffs: &[f64; 5],
  ) -> Option<f64>
  ```
- **運算過程**：
  - $y = \text{coeffs}[0] \cdot x + \text{state}[0]$
  - $\text{state}[0] = \text{coeffs}[1] \cdot x - \text{coeffs}[3] \cdot y + \text{state}[1]$
  - $\text{state}[1] = \text{coeffs}[2] \cdot x - \text{coeffs}[4] \cdot y$
  - 回傳 `Some(y)`。若包含 NaN/Inf 則返回 `None`。

## 黃金測試向量 (Golden Vectors)
- 採樣率 $1000.0\text{ Hz}$，陷波中心頻率 $250.0\text{ Hz}$ ($f_0 / f_s = 0.25$，$\omega_0 = \pi/2$)，$Q = 0.7071$：
  - $\sin(\pi/2) = 1.0, \quad \cos(\pi/2) = 0.0$
  - $\alpha = 1.0 / (2 \times 0.7071) \approx 0.707113562$
  - $b_0 = 1.0, b_1 = 0.0, b_2 = 1.0$
  - $a_0 = 1.0 + \alpha \approx 1.707113562, a_1 = 0.0, a_2 = 1.0 - \alpha \approx 0.292886438$
  - 歸一化：$b_0/a_0 \approx 0.585786, b_1/a_0 = 0.0, b_2/a_0 \approx 0.585786, a_1/a_0 = 0.0, a_2/a_0 \approx 0.171573$
- 測試斷言使用 `(val - golden).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_biquad_notch.rs）
```golden
// 出處：規格黃金向量 fs=1000, f0=250, Q=0.7071 → [0.585786, 0, 0.585786, 0, 0.171573]（Python 驗算 0.5857841 / 0.1715682，差 < 1e-5，容差照規格 1e-4）
let c = biquad_notch_coefficients(1000.0, 250.0, 0.7071).unwrap();
(c[0] - 0.585786).abs() < 1e-4
(c[1] - 0.0).abs() < 1e-4
(c[2] - 0.585786).abs() < 1e-4
(c[3] - 0.0).abs() < 1e-4
(c[4] - 0.171573).abs() < 1e-4
// 出處：邊界防禦 f0 >= fs/2、參數 <= 0、NaN/Inf → None
biquad_notch_coefficients(1000.0, 500.0, 0.7071).is_none()
biquad_notch_coefficients(1000.0, 250.0, -1.0).is_none()
biquad_notch_coefficients(f64::INFINITY, 250.0, 0.7071).is_none()
```
