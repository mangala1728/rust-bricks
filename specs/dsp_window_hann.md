# 模組名稱: 數位訊號處理之 Hann 窗函式係數生成 (DSP Hann Window)

## 1. 目標定位
- 目標模組檔: `src/dsp_window_hann.rs`
- 目標測試檔: `tests/test_dsp_window_hann.rs`
- 功能簡述: 針對音訊與 FFT 提供零外部依賴、純標準庫之 Hann 窗係數計算純函式。

## 2. 09 合約介面定義
- 檔案開頭必備 `// ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 2
  /// @domain: n [0..65535]
  /// @domain: total_len [1..65536]
  pub fn hann_window_coeff(n: usize, total_len: usize) -> Option<f64>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- `total_len == 0 || total_len > 65536 || n >= total_len` 時，第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 <= 60 行（NASA JPL 微積木原則），控制流巢狀深度 <= 2，禁止遞迴與 `static mut`。
- 計算使用 Rust 標準庫 `std::f64::consts::PI` 與 `.cos()`，禁止自造粗糙泰勒展開。

## 4. 確定性黃金值測試要求 (Golden Value Tests)
- 基準端點測試: `n = 0, total_len = 5` 時輸出必須為 `0.0` (`abs() < 1e-6`)。
- 對稱中心測試: `n = 2, total_len = 5` 時輸出必須為 `1.0` (`(val - 1.0).abs() < 1e-6`)。
- 單點窗測試: `n = 0, total_len = 1` 時輸出為 `1.0`。
- 越界測試: `n = 5, total_len = 5` 必須返回 `None`；`total_len = 0` 返回 `None`。
