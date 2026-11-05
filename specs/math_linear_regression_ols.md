# 模組名稱: 普通最小平方法 OLS 一維線性回歸與 R2 計算器 (Ordinary Least Squares 1D Linear Regression & R-squared)

## 1. 目標定位
- 目標模組檔: `src/math_linear_regression_ols.rs`
- 目標測試檔: `tests/test_math_linear_regression_ols.rs`
- 功能簡述: 統計迴歸分析與趨勢估算之普通最小平方法 (OLS) 純函式。輸入成對數據點 $(x_i, y_i)$，無配置計算最佳擬合斜率 $m$、截距 $b$ 與判定係數 $R^2$。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Ordinary Least Squares (OLS) 1D linear regression with slope, intercept, and R-squared
// Inputs: x: &[f64], y: &[f64]
// Outputs: Option<(f64, f64, f64)> (slope, intercept, r_squared)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: x.len() >= 2, x.len() == y.len(), x and y finite, variance of x > 0
pub fn math_linear_regression_ols(
    x: &[f64],
    y: &[f64],
) -> Option<(f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `x.len() < 2 || x.len() != y.len()`，立即回傳 `None`。
- 檢查所有數據皆為有限數值，若含 `NaN` 立即回傳 `None`。
- 計算平均值：
  - `let n = x.len() as f64;`
  - `let mean_x = sum(x) / n; let mean_y = sum(y) / n;`
  - 累加變異量：
    - `ss_xx = sum((x_i - mean_x)^2)`
    - `ss_yy = sum((y_i - mean_y)^2)`
    - `ss_xy = sum((x_i - mean_x) * (y_i - mean_y))`
- 若 `ss_xx <= 1e-14` (所有 $x$ 座標重疊退化)，立即回傳 `None`。
- 計算：
  - 斜率：`slope = ss_xy / ss_xx`
  - 截距：`intercept = mean_y - slope * mean_x`
  - 判定係數：若 `ss_yy <= 1e-14` 則 `r2 = 1.0`，否則 `r2 = (ss_xy * ss_xy) / (ss_xx * ss_yy)`。
- 回傳 `Some((slope, intercept, r2))`。

## 4. 確定性黃金值測試要求
- 精確直線 $y = 2x + 1$：
  - `x = [1.0, 2.0, 3.0, 4.0], y = [3.0, 5.0, 7.0, 9.0]`
  - `slope = 2.0, intercept = 1.0, r2 = 1.0`
- 長度不匹配或 $x$ 退化為常數時回傳 `None`。

```golden
let x = [1.0, 2.0, 3.0, 4.0];
let y = [3.0, 5.0, 7.0, 9.0];
let (m, b, r2) = math_linear_regression_ols(&x, &y).unwrap();
(m - 2.0).abs() < 1e-12
(b - 1.0).abs() < 1e-12
(r2 - 1.0).abs() < 1e-12
math_linear_regression_ols(&[1.0, 1.0], &[2.0, 3.0]).is_none()
math_linear_regression_ols(&[1.0], &[2.0]).is_none()
math_linear_regression_ols(&x, &[1.0, 2.0]).is_none()
```
