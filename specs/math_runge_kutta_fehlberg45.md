# 模組名稱: RKF45 自適應步長常微分方程嵌入誤差估算器 (RKF45 Embedded Truncation Error Estimator)

## 1. 目標定位
- 目標模組檔: `src/math_runge_kutta_fehlberg45.rs`
- 目標測試檔: `tests/test_math_runge_kutta_fehlberg45.rs`
- 功能簡述: Runge-Kutta-Fehlberg 4(5) 階嵌入式 ODE 數值求解器之局部截斷誤差 (Truncation Error) 計算純函式。依據 Butcher 表係數之差值線性組合，為自適應步長控制提供精確誤差估計。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Runge-Kutta-Fehlberg (RKF45) embedded truncation error calculation
// Inputs: h, k1, k3, k4, k5, k6: f64
// Outputs: Option<f64> (local_truncation_error)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: h > 0.0, k1..k6 finite
pub fn math_rkf45_error_estimate(
    h: f64,
    k1: f64,
    k3: f64,
    k4: f64,
    k5: f64,
    k6: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數值，立即回傳 `None`。
- 若 `h <= 0.0`，立即回傳 `None`。
- 標準 Fehlberg 4(5) 誤差係數差值：
  - $e_1 = 1.0 / 360.0$
  - $e_3 = -128.0 / 4275.0$
  - $e_4 = -2197.0 / 75240.0$
  - $e_5 = 1.0 / 50.0$
  - $e_6 = 2.0 / 55.0$
- 計算：
  - `error = h * (e1 * k1 + e3 * k3 + e4 * k4 + e5 * k5 + e6 * k6)`
- 回傳 `Some(error)`。

## 4. 確定性黃金值測試要求
- 若所有 $k_i = 0$，誤差必須精確為 $0.0$。
- 若 $k_1 = 360.0, k_3..k_6 = 0, h = 1.0$，則誤差必須為 $1.0$。
- 負步長 $h \le 0$ 或含 NaN 拒絕。

```golden
let err_zero = math_rkf45_error_estimate(1.0, 0.0, 0.0, 0.0, 0.0, 0.0).unwrap();
err_zero.abs() < 1e-12
let err_unit = math_rkf45_error_estimate(1.0, 360.0, 0.0, 0.0, 0.0, 0.0).unwrap();
(err_unit - 1.0).abs() < 1e-12
math_rkf45_error_estimate(0.0, 1.0, 0.0, 0.0, 0.0, 0.0).is_none()
math_rkf45_error_estimate(-1.0, 1.0, 0.0, 0.0, 0.0, 0.0).is_none()
```
