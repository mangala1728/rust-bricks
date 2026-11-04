# 規格書：梅爾濾波器組三角窗邊界計算 (Mel Filterbank Bounds)

## 模組名稱
`dsp_mel_filterbank`

## 業務目標與適用場景
在語音識別前端特徵提取 (MFCC)、聲學事件辨識 (Audio Event Classification) 中，梅爾標度 (Mel Scale) 模擬人類耳朵對低頻高解析、高頻低解析的非線性聽覺特性。
本模組提供零外部依賴、純 `std` 實作的 Hz 轉 Mel、Mel 轉 Hz 與梅爾濾波器組中心頻率計算微積木。

## 09 號合約邊界規範

### 函式 1：赫茲與梅爾標度雙向轉換 (Hz to Mel & Mel to Hz)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: hz [0.0..100000.0]
  pub fn hz_to_mel(hz: f64) -> Option<f64>
  ```
  ```rust
  /// @outputs: 1
  /// @domain: mel [0.0..10000.0]
  pub fn mel_to_hz(mel: f64) -> Option<f64>
  ```
- **數學公式 (O'Shaughnessy Standard)**：
  - $M = 2595.0 \cdot \log_{10}(1.0 + f / 700.0)$
  - $f = 700.0 \cdot (10^{M / 2595.0} - 1.0)$
- **邊界防禦**：
  - 若 `hz < 0.0` 或 `mel < 0.0` 或為 NaN/Inf，回傳 `None`。

### 函式 2：計算梅爾濾波器組中心頻率點 (Mel Filterbank Center Frequencies)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: num_filters [2..128]
  /// @domain: min_hz [0.0..10000.0]
  /// @domain: max_hz [100.0..100000.0]
  pub fn mel_filterbank_centers_hz(
      num_filters: usize,
      min_hz: f64,
      max_hz: f64,
  ) -> Option<Vec<f64>>
  ```
- **運算過程**：
  - 若 `num_filters < 2 || min_hz >= max_hz` 回傳 `None`。
  - 計算 `min_mel = hz_to_mel(min_hz)` 與 `max_mel = hz_to_mel(max_hz)`。
  - 等間距劃分 `num_filters + 2` 個點（包含左右邊界）：
    $\Delta = (max\_mel - min\_mel) / (num\_filters + 1)$
  - 轉回 Hz 並存入 `Vec<f64>`，回傳 `Some(centers)`。

## 黃金測試向量 (Golden Vectors)
- $f = 1000.0\text{ Hz}$:
  - $1 + 1000/700 = 1 + 1.4285714 = 2.4285714$
  - $\log_{10}(2.4285714) \approx 0.385350$
  - $M = 2595.0 \times 0.385350 \approx 999.985$ (約 1000 Mel)。
  - 雙向還原：`mel_to_hz(999.985) = 1000.0 Hz`。
- 容差使用 `(val - golden).abs() < 1e-2`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_mel_filterbank.rs）
```golden
// 出處：規格黃金向量 hz_to_mel(1000) ≈ 999.985（Python 驗算 999.98554，容差照規格 1e-2）
(hz_to_mel(1000.0).unwrap() - 999.985).abs() < 1e-2
// 出處：規格雙向還原 mel_to_hz(999.985) ≈ 1000（Python 驗算 999.99919）
(mel_to_hz(999.985).unwrap() - 1000.0).abs() < 1e-2
// 出處：邊界防禦 負值、NaN → None
hz_to_mel(-1.0).is_none()
mel_to_hz(-1.0).is_none()
hz_to_mel(f64::NAN).is_none()
// 出處：運算過程 num_filters < 2 或 min_hz >= max_hz → None
mel_filterbank_centers_hz(1, 0.0, 8000.0).is_none()
mel_filterbank_centers_hz(10, 8000.0, 8000.0).is_none()
// 出處：運算過程 等間距 num_filters + 2 個點（含左右邊界）→ 長度 12，首尾等於 min_hz / max_hz
let centers = mel_filterbank_centers_hz(10, 0.0, 8000.0).unwrap();
centers.len() == 12
(centers[0] - 0.0).abs() < 1e-2
(centers[11] - 8000.0).abs() < 1e-2
```
