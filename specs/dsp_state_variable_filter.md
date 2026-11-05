# 模組名稱: Chamberlin 狀態變數濾波器單步步進計算器 (Chamberlin State Variable Filter Step)

## 1. 目標定位
- 目標模組檔: `src/dsp_state_variable_filter.rs`
- 目標測試檔: `tests/test_dsp_state_variable_filter.rs`
- 功能簡述: Hal Chamberlin 經典數位狀態變數濾波器 (SVF) 單步演算法。輸入單一樣本與內部狀態，在單次運算中同時產出低通 (LP)、帶通 (BP)、高通 (HP)、陷波 (Notch) 四組輸出與更新後的濾波狀態。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Chamberlin Digital State Variable Filter single-step computation
// Inputs: input: f64, s1: f64, s2: f64, f: f64, q: f64
// Outputs: Option<(f64, f64, f64, f64, f64, f64)> (lp, bp, hp, notch, next_s1, next_s2)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 6
/// @domain: input in [-1e6..=1e6], s1 in [-1e6..=1e6], s2 in [-1e6..=1e6], f in [0.0001..=0.8], q in [0.0001..=2.0]
pub fn dsp_svf_step(
    input: f64,
    s1: f64,
    s2: f64,
    f: f64,
    q: f64,
) -> Option<(f64, f64, f64, f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `f < 0.0001 || f > 0.8` (頻率調諧係數，Chamberlin 結構在 f > 0.8 時會接近不穩定)，立即回傳 `None`。
- 若 `q < 0.0001 || q > 2.0` (阻尼係數 1/Q)，立即回傳 `None`。
- 計算：
  - `hp = input - s1 - q * s2`
  - `bp = s2 + f * hp`
  - `lp = s1 + f * bp`
  - `notch = hp + lp`
  - `next_s1 = lp`
  - `next_s2 = bp`
- 回傳 `Some((lp, bp, hp, notch, next_s1, next_s2))`。

## 4. 確定性黃金值測試要求
- `input = 1.0, s1 = 0.0, s2 = 0.0, f = 0.5, q = 1.0`:
  - `hp = 1.0 - 0.0 - 0.0 = 1.0`
  - `bp = 0.0 + 0.5 * 1.0 = 0.5`
  - `lp = 0.0 + 0.5 * 0.5 = 0.25`
  - `notch = 1.0 + 0.25 = 1.25`
  - `next_s1 = 0.25, next_s2 = 0.5`
- 越界 `f` 或無效 `q` 拒絕。

```golden
let res = dsp_svf_step(1.0, 0.0, 0.0, 0.5, 1.0).unwrap();
(res.0 - 0.25).abs() < 1e-12
(res.1 - 0.50).abs() < 1e-12
(res.2 - 1.00).abs() < 1e-12
(res.3 - 1.25).abs() < 1e-12
(res.4 - 0.25).abs() < 1e-12
(res.5 - 0.50).abs() < 1e-12
dsp_svf_step(1.0, 0.0, 0.0, 0.9, 1.0).is_none()
dsp_svf_step(1.0, 0.0, 0.0, 0.5, 0.0).is_none()
```
