# 模組名稱: 三次樣條插值單段評估器 (Cubic Spline Segment Evaluator)

## 1. 目標定位
- 目標模組檔: `src/math_cubic_spline.rs`
- 目標測試檔: `tests/test_math_cubic_spline.rs`
- 功能簡述: 在無人機軌跡平滑規劃、機器人關節路徑生成與高精度曲線擬合中，給定分段端點座標與二階導數曲率 $(M_0, M_1)$，計算區間內任意點之三次樣條插值數值。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Natural cubic spline single segment evaluation
// Inputs: x, x0, x1, y0, y1, m0, m1: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: x0 < x1, x in [x0..=x1], all inputs finite
pub fn math_cubic_spline_eval(
    x: f64,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    m0: f64,
    m1: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數、`x1 <= x0`、或 `x < x0 || x > x1`（超出插值區間），立即回傳 `None`。
- 標準三次樣條展開式（令 $h = x_1 - x_0$）：
  $$S(x) = \frac{m_0 (x_1 - x)^3}{6h} + \frac{m_1 (x - x_0)^3}{6h} + \left(y_0 - \frac{m_0 h^2}{6}\right) \frac{x_1 - x}{h} + \left(y_1 - \frac{m_1 h^2}{6}\right) \frac{x - x_0}{h}$$
- 回傳 `Some(S(x))`。

## 4. 確定性黃金值測試要求
- 直線退化驗證（$m_0 = m_1 = 0$）：
  `x0=0, x1=2, y0=10, y1=20`，在 $x=1.0$ 處必須為 $15.0$。
- 二次拋物線精確驗證（$y=x^2$，二階導 $m_0=m_1=2$）：
  `x0=0, x1=2, y0=0, y1=4, m0=2, m1=2`，在 $x=1.0$ 處必須精確等於 $1.0$。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_math_cubic_spline.rs）
```golden
(math_cubic_spline_eval(1.0, 0.0, 2.0, 10.0, 20.0, 0.0, 0.0).unwrap() - 15.0).abs() < 1e-10
(math_cubic_spline_eval(1.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 1.0).abs() < 1e-10
(math_cubic_spline_eval(0.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 0.0).abs() < 1e-10
(math_cubic_spline_eval(2.0, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).unwrap() - 4.0).abs() < 1e-10
math_cubic_spline_eval(2.5, 0.0, 2.0, 0.0, 4.0, 2.0, 2.0).is_none()
math_cubic_spline_eval(1.0, 2.0, 0.0, 0.0, 4.0, 2.0, 2.0).is_none()
```
