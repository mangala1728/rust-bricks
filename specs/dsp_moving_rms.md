# 規格書：滑動視窗均方根 (Moving RMS Power Detector)

## 模組名稱
`dsp_moving_rms`

## 業務目標與適用場景
在電力監測、音訊電平量測（True RMS / Loudness）、聲學感測與振動特徵分析中，均方根（RMS）是表徵交流訊號有效能量的核心指標。
本模組提供零外部依賴、純 `std` 實作的 $O(1)$ 遞推滑動視窗平方和更新與整段切片 RMS 計算微積木。

## 09 號合約邊界規範

### 函式 1：O(1) 遞推滑動視窗 RMS 更新 (Moving RMS Step)
- **函式簽名**：
  ```rust
  /// @outputs: 2
  /// @domain: window_len >= 1, sum_sq >= 0.0
  pub fn dsp_moving_rms_step(x_new: f64, x_old: f64, sum_sq: f64, window_len: usize) -> Option<(f64, f64)>
  ```
- **演算法規範**：
  - 邊界檢查：`if window_len == 0 || !x_new.is_finite() || !x_old.is_finite() || !sum_sq.is_finite() || sum_sq < 0.0 { return None; }`
  - 遞推平方和更新（嚴防浮點微小負值，強制保證物理非負性）：
    ```rust
    let new_sum_sq = (sum_sq + (x_new * x_new) - (x_old * x_old)).max(0.0);
    let rms = (new_sum_sq / (window_len as f64)).sqrt();
    ```
  - 回傳 `Some((new_sum_sq, rms))`。

### 函式 2：單切片批次 RMS 計算 (Batch Slice RMS)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: signal.len() >= 1
  pub fn dsp_rms_slice(signal: &[f64]) -> Option<f64>
  ```
- **演算法規範**：
  - 邊界檢查：`if signal.is_empty() || signal.iter().any(|&x| !x.is_finite()) { return None; }`
  - 計算平均平方和之平方根：
    ```rust
    let sum_sq: f64 = signal.iter().map(|&x| x * x).sum();
    let mean_sq = sum_sq / (signal.len() as f64);
    let rms = mean_sq.sqrt();
    ```
  - 回傳 `Some(rms)`。

## 黃金測試向量 (Golden Vectors)
- 向量 1 (弦波峰值為 1.0 的有效值):
  - 正弦波 $A \sin(\omega t)$ 之 RMS 理論值為 $A / \sqrt{2} \approx 0.70710678$。
  - 對直流常數訊號 `[3.0, 3.0, 3.0, 3.0]` -> RMS 必須為 `3.0`。
- 向量 2 (遞推滑動更新):
  - `sum_sq = 9.0`, `x_new = 4.0`, `x_old = 3.0`, `window_len = 4`
  - `new_sum_sq = 9.0 + 16.0 - 9.0 = 16.0`
  - `rms = sqrt(16.0 / 4) = 2.0`
  - 回傳 `Some((16.0, 2.0))`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_moving_rms.rs）
```golden
// 出處：向量 1 直流常數 [3,3,3,3] → RMS = 3（手算）
(dsp_rms_slice(&[3.0, 3.0, 3.0, 3.0]).unwrap() - 3.0).abs() < 1e-6
// 出處：向量 1 峰值 1.0 正弦 RMS = 1/sqrt(2)；取整週期 8 點取樣（離散整週期取樣時恆等式精確成立，Python 驗算 0.70710678）
let sine: Vec<f64> = (0..8).map(|n| (2.0 * std::f64::consts::PI * n as f64 / 8.0).sin()).collect();
(dsp_rms_slice(&sine).unwrap() - 0.70710678).abs() < 1e-6
// 出處：向量 2 遞推 sum_sq=9, x_new=4, x_old=3, window_len=4 → (16, 2)（手算）
let (s, r) = dsp_moving_rms_step(4.0, 3.0, 9.0, 4).unwrap();
(s - 16.0).abs() < 1e-6
(r - 2.0).abs() < 1e-6
// 出處：邊界檢查 window_len == 0、sum_sq < 0、非有限值、空切片 → None
dsp_moving_rms_step(4.0, 3.0, 9.0, 0).is_none()
dsp_moving_rms_step(4.0, 3.0, -1.0, 4).is_none()
dsp_moving_rms_step(f64::NAN, 3.0, 9.0, 4).is_none()
dsp_rms_slice(&[]).is_none()
dsp_rms_slice(&[1.0, f64::INFINITY]).is_none()
```
