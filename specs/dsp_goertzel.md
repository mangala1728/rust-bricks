# 規格書：Goertzel 單頻率 DFT 檢波演算法 (Goertzel Single-Tone Detector)

## 模組名稱
`dsp_goertzel`

## 業務目標與適用場景
在雙音多頻電話撥號 (DTMF 訊號解碼)、音頻觸發開關、工業超音波特定諧波能量檢測中，若僅需偵測少數特定頻率點（如 697Hz, 770Hz, 852Hz, 941Hz），計算完整的 FFT ($O(N \log N)$) 運算代價過於昂貴。
Goertzel 演算法為二階 IIR 濾波器結構，以 $O(N)$ 複雜度、僅需一次實數乘法/樣本即可求出單一目標頻率的能量。

## 09 號合約邊界規範

### 函式 1：計算 Goertzel 係數
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: target_freq_hz [1.0..500000.0]
  /// @domain: sample_rate_hz [2.0..1000000.0]
  pub fn goertzel_coeff(target_freq_hz: f64, sample_rate_hz: f64) -> Option<f64>
  ```
- **數學公式**：
  - 若 `target_freq_hz <= 0.0 || sample_rate_hz <= 0.0 || target_freq_hz >= sample_rate_hz / 2.0`，回傳 `None`。
  - $\omega = 2\pi \cdot \frac{f_0}{f_s}$
  - 回傳 `Some(2.0 * \cos(\omega))`。

### 函式 2：執行 Goertzel 區塊檢波計算能量 (Goertzel Power & Magnitude)
- **函式簽名**：
  ```rust
  /// @outputs: 2
  pub fn goertzel_detect(samples: &[f64], coeff: f64) -> Option<(f64, f64)>
  ```
- **運算過程**：
  - 若 `samples.is_empty()` 或 `coeff.abs() > 2.0` 回傳 `None`。
  - 令狀態變數 `s1 = 0.0`, `s2 = 0.0`。
  - 對 `x` in `samples`：
    - $s_0 = x + coeff \cdot s_1 - s_2$
    - $s_2 = s_1$
    - $s_1 = s_0$
  - 計算功率：$P = s_1^2 + s_2^2 - coeff \cdot s_1 \cdot s_2$
  - 若 $P < 0.0$，夾止為 0.0。
  - 回傳 `Some((P, P.sqrt()))` (功率與振幅)。

## 黃金測試向量 (Golden Vectors)
- 採樣率 $f_s = 8000.0\text{ Hz}$，目標頻率 $f_0 = 1000.0\text{ Hz}$，$\omega = \pi/4$：
  - 係數 $\text{coeff} = 2 \cos(\pi/4) = \sqrt{2} \approx 1.41421356$。
  - 測試訊號：長度 $N = 8$，純正弦波 $x[n] = \sin(2\pi \cdot 1000 \cdot n / 8000)$。
  - 理論功率 $P = 16.0$。
  - 理論振幅 $M = 4.0$。
- 斷言 `(coeff - 1.41421356).abs() < 1e-4`。
- 斷言 `(p - 16.0).abs() < 1e-4`。
- 斷言 `(mag - 4.0).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_goertzel.rs）
```golden
// 出處：規格黃金向量 fs=8000, f0=1000 → coeff = 2cos(pi/4) = sqrt(2)（手算）
let coeff = goertzel_coeff(1000.0, 8000.0).unwrap();
(coeff - 1.41421356).abs() < 1e-4
// 出處：規格黃金向量 N=8 純正弦 → P=16, M=4（Python 獨立 DFT 驗算 |X[1]|^2 = 16）
let x: Vec<f64> = (0..8).map(|n| (2.0 * std::f64::consts::PI * 1000.0 * n as f64 / 8000.0).sin()).collect();
let (p, mag) = goertzel_detect(&x, coeff).unwrap();
(p - 16.0).abs() < 1e-4
(mag - 4.0).abs() < 1e-4
// 出處：邊界規則 target >= fs/2、空樣本、|coeff| > 2 → None
goertzel_coeff(4000.0, 8000.0).is_none()
goertzel_detect(&[], 1.0).is_none()
goertzel_detect(&[1.0, 2.0], 2.5).is_none()
```
