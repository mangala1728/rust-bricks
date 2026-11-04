# 規格書：PPG 雙波長光學血氧飽和度估算器 (PPG SpO2 Estimator)

## 模組名稱
`sensor_ppg_spo2`

## 業務目標與適用場景
在穿戴手環、指夾血氧計中，光體積變化描記圖 (PPG) 利用紅光 (Red, ~660nm) 與紅外光 (IR, ~940nm) 的吸光度比值估計動脈血氧飽和度 ($SpO_2$)。
本模組提供零外部依賴、純 `std` 實作的雙波長 AC/DC 比值轉化與標準經驗校準曲線計算微積木。

## 09 號合約邊界規範

### 函式 1：計算雙波長吸收比值 R (Ratio of Ratios)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: ac_red [0.0..1.0e6]
  /// @domain: dc_red [1.0..1.0e8]
  /// @domain: ac_ir [0.0..1.0e6]
  /// @domain: dc_ir [1.0..1.0e8]
  pub fn ppg_ratio_of_ratios(
      ac_red: f64,
      dc_red: f64,
      ac_ir: f64,
      dc_ir: f64,
  ) -> Option<f64>
  ```
- **數學公式**：
  - $R = \frac{\text{ac\_red} / \text{dc\_red}}{\text{ac\_ir} / \text{dc\_ir}}$
  - 若 $dc\_red \le 0$ 或 $dc\_ir \le 0$ 或 $ac\_ir \le 1e-12$，返回 `None`。

### 函式 2：利用標準經驗二項式估算 SpO2 百分比
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: r [0.1..3.0]
  pub fn ppg_estimate_spo2(r: f64) -> Option<f64>
  ```
- **經驗曲線 (Maxim / TI 標準校準多項式)**：
  - $SpO_2 = -45.060 \cdot R^2 + 30.354 \cdot R + 94.845$
  - 結果夾止於 $[0.0, 100.0]$。

## 黃金測試向量 (Golden Vectors)
- $R = 0.5$:
  - $SpO_2 = -45.060 \times 0.25 + 30.354 \times 0.5 + 94.845 = -11.265 + 15.177 + 94.845 = 98.757$。
- $R = 1.0$:
  - $SpO_2 = -45.060 \times 1.0 + 30.354 \times 1.0 + 94.845 = 80.139$。
- 容差使用 `(val - golden).abs() < 1e-3`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_sensor_ppg_spo2.rs）
```golden
// 出處：規格向量 R = 0.5 → 98.757、R = 1.0 → 80.139，容差 1e-3（Python 獨立驗算）
(ppg_estimate_spo2(0.5).unwrap() - 98.757).abs() < 1e-3
(ppg_estimate_spo2(1.0).unwrap() - 80.139).abs() < 1e-3
// 出處：經驗曲線＋夾止規則手算：R = 3.0 → -45.060×9 + 30.354×3 + 94.845 = -219.633，夾止為 0.0
(ppg_estimate_spo2(3.0).unwrap() - 0.0).abs() < 1e-3
// 出處：函式 1 拒絕條件：dc_red <= 0、dc_ir <= 0、ac_ir <= 1e-12 回傳 None
ppg_ratio_of_ratios(0.02, 0.0, 0.04, 1.0).is_none()
ppg_ratio_of_ratios(0.02, 1.0, 0.04, 0.0).is_none()
ppg_ratio_of_ratios(0.02, 1.0, 0.0, 1.0).is_none()
```
