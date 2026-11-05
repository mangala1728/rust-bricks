# 模組名稱: 割線法非線性方程求根器 (Secant Method Root Finder)

## 1. 目標定位
- 目標模組檔: `src/math_secant_method.rs`
- 目標測試檔: `tests/test_math_secant_method.rs`
- 功能簡述: 在不依賴解析導函數的前提下，利用前兩次迭代點之有限差分斜率逼近切線，超線性收斂（階數約 1.618）求解一維非線性方程 $f(x) = 0$ 之實根。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 1D numerical root finding using secant line approximation
// Inputs: x_prev, x_curr, f_prev, f_curr: f64, tol: f64, max_iter: u32, f: Fn(f64)->f64
// Outputs: Option<f64> or Option<(f64, u32)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x_prev, x_curr, f_prev, f_curr are finite, |f_curr - f_prev| > 1e-15
pub fn math_secant_step(
    x_prev: f64,
    x_curr: f64,
    f_prev: f64,
    f_curr: f64,
) -> Option<f64>;

/// @outputs: 2
/// @domain: tol > 0.0, max_iter >= 1, x0 != x1, x0 and x1 are finite
pub fn math_secant_solve<F>(
    x0: f64,
    x1: f64,
    tol: f64,
    max_iter: u32,
    f: F,
) -> Option<(f64, u32)>
where
    F: Fn(f64) -> f64;
```

## 3. 邊界與異常條件 (Domain Guards)
- `math_secant_step`: 若輸入含非有限數或 `(f_curr - f_prev).abs() <= 1e-15`（斜率近乎水平除以零），回傳 `None`。
  $$x_{\text{next}} = x_{\text{curr}} - f_{\text{curr}} \times \frac{x_{\text{curr}} - x_{\text{prev}}}{f_{\text{curr}} - f_{\text{prev}}}$$
- `math_secant_solve`: 若 `tol <= 0.0 || max_iter == 0 || (x1 - x0).abs() <= 1e-15 || !x0.is_finite() || !x1.is_finite()`，回傳 `None`。若在 `max_iter` 內未達標或遇到斜率為零，回傳 `None`。

## 4. 確定性黃金值測試要求
- 單步割線截距：`x_prev = 1.0, x_curr = 3.0, f_prev = -3.0, f_curr = 5.0`
  $$x = 3.0 - 5.0 \times \frac{2.0}{8.0} = 1.75$$
- 求解 $x^2 - 4.0 = 0$（初始點 $1.0, 3.0$，容差 $10^{-6}$）：
  收斂根必為 $2.0$，容差 `< 1e-5`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_math_secant_method.rs）
```golden
(math_secant_step(1.0, 3.0, -3.0, 5.0).unwrap() - 1.75).abs() < 1e-10
math_secant_step(1.0, 3.0, 5.0, 5.0).is_none()
let (root, iters) = math_secant_solve(1.0, 3.0, 1e-6, 50, |x| x * x - 4.0).unwrap();
(root - 2.0).abs() < 1e-5
iters > 0 && iters <= 20
math_secant_solve(1.0, 1.0, 1e-6, 50, |x| x * x - 4.0).is_none()
math_secant_solve(1.0, 3.0, -1e-6, 50, |x| x * x - 4.0).is_none()
```
