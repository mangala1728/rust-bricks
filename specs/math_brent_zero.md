# 模組名稱: Brent 根求解法單步候選點運算器 (Brent Root Finder Step Candidate)

## 1. 目標定位
- 目標模組檔: `src/math_brent_zero.rs`
- 目標測試檔: `tests/test_math_brent_zero.rs`
- 功能簡述: Brent-Dekker 數值求根演算法核心單步候選點生成純函式。依據前三個點與函數值，動態判定並執行反二次插值 (Inverse Quadratic Interpolation) 或割線插值 (Secant)，並在插值偏離時自動退回安全二分點 (Bisection)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Brent-Dekker numerical root-finder candidate step computation
// Inputs: a, b, c, fa, fb, fc: f64
// Outputs: Option<f64> (next_candidate_x)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: a, b, c, fa, fb, fc finite, fa * fb <= 0.0 or brackets root, tol > 0.0
pub fn math_brent_candidate_step(
    a: f64,
    b: f64,
    c: f64,
    fa: f64,
    fb: f64,
    fc: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數值，立即回傳 `None`。
- 若 `(fa - fb).abs() < 1e-15`，立即回傳 `None` (避免除以零)。
- 運算法則：
  - 判定是否可使用反二次插值 (三個函數值皆不相同)：
    - 若 `(fa - fc).abs() > 1e-14 && (fb - fc).abs() > 1e-14`：
      - `term1 = (a * fb * fc) / ((fa - fb) * (fa - fc))`
      - `term2 = (b * fa * fc) / ((fb - fa) * (fb - fc))`
      - `term3 = (c * fa * fb) / ((fc - fa) * (fc - fb))`
      - `s = term1 + term2 + term3`
    - 否則 (割線插值)：
      - `s = b - fb * (b - a) / (fb - fa)`
  - 安全檢查：檢查 $s$ 是否介於 $(3a + b)/4$ 與 $b$ 之間：
    - `let min_bound = (3.0 * a + b) / 4.0;`
    - `let (low, high) = if min_bound < b { (min_bound, b) } else { (b, min_bound) };`
    - 若 `s < low || s > high`：
      - 退回二分點：`s = (a + b) / 2.0;`
- 回傳 `Some(s)`。

## 4. 確定性黃金值測試要求
- 割線退化情境：`c == a` (則 `fa == fc`)
  - `a = 0.0, b = 2.0, c = 0.0, fa = -2.0, fb = 2.0, fc = -2.0`
  - 割線法：`s = 2.0 - 2.0 * (2.0 - 0.0) / (2.0 - (-2.0)) = 2.0 - 1.0 = 1.0`
  - 區間 $(3a+b)/4 = 0.5$ 至 $b=2.0$，候選點 $1.0$ 在區間內，輸出 $1.0$。
- 函數值相等除以零時回傳 `None`。

```golden
let s = math_brent_candidate_step(0.0, 2.0, 0.0, -2.0, 2.0, -2.0).unwrap();
(s - 1.0).abs() < 1e-12
math_brent_candidate_step(0.0, 2.0, 1.0, 1.0, 1.0, 2.0).is_none()
```
