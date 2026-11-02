# 模組名稱: [例如: 一維卡爾曼與指數移動平均濾波 (EMA & 1D Smoother)]

## 1. 目標定位
- 目標模組檔: `src/dsp_ema_smoother.rs`
- 目標測試檔: `tests/test_dsp_ema_smoother.rs`
- 功能簡述: 針對高頻感測器與音訊串流提供零外部依賴、無配置 (Zero-Allocation) 之平滑濾波與突波抑制純函式。

## 2. 09 合約介面定義
- 檔案開頭必備 `// ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 2
  /// @domain: alpha [0.0001..1.0]
  /// @domain: current [-1000000.0..1000000.0]
  /// @domain: prev [-1000000.0..1000000.0]
  pub fn ema_step_f64(current: f64, prev: f64, alpha: f64) -> Option<f64>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- `alpha < 0.0001 || alpha > 1.0` 或含 `NaN` / `Infinite` 時，第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。
- 嚴禁使用未附帶 `// @safety-proof:` 註解的 `.unwrap()`、`.expect()` 或 `panic!`。

## 4. 確定性黃金值測試要求 (Golden Value Tests)
- 基準恆等測試：`alpha = 1.0` 時輸出必須精確等於 `current`；`current == prev` 時輸出保持不變。
- 確定性黃金值驗證：`current = 100.0, prev = 0.0, alpha = 0.25` $\rightarrow$ 斷言 `(res - 25.0).abs() < 1e-6`。
- 非法邊界拒絕測試：`alpha = 0.0`、`alpha = 1.5`、`f64::NAN` 均必須斷言為 `None`。
