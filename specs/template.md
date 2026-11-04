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

## 5. 機器可驗證的黃金向量（```golden 區塊，新規格必填）
雲端農夫會把下面這個區塊**自動轉成 `tests/golden_<模組名>.rs`**，模型看不到也改不了這個測試檔；預期值只能由規格作者修改。
- 每行一個 Rust 布林運算式，產生成 `assert!(運算式, "spec golden vector #N")`；以 `let` 開頭或分號結尾的行是準備敘述。
- 預期值必須有出處（手算推導、RFC／標準文件附錄、或獨立參考實作），不可先跑程式再把輸出填回來。
- 若不同模型都卡在同一個向量，農夫會判定「規格可疑」並開 issue 交給人類複核；修改規格後，舊的失敗紀錄自動失效。

```golden
// 確定性黃金值：100 × 0.25 + 0 × 0.75 = 25（手算）
(ema_step_f64(100.0, 0.0, 0.25).unwrap() - 25.0).abs() < 1e-6
// alpha = 1 時輸出等於 current
(ema_step_f64(42.0, 7.0, 1.0).unwrap() - 42.0).abs() < 1e-12
// current == prev 時輸出不變
(ema_step_f64(5.0, 5.0, 0.3).unwrap() - 5.0).abs() < 1e-12
// 非法邊界一律拒絕
ema_step_f64(1.0, 0.0, 0.0).is_none()
ema_step_f64(1.0, 0.0, 1.5).is_none()
ema_step_f64(f64::NAN, 0.0, 0.5).is_none()
```
