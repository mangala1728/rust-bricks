# 模組名稱: 2D Nelder-Mead 單純形法反射與擴張運算器 (2D Nelder-Mead Simplex Operations)

## 1. 目標定位
- 目標模組檔: `src/math_nelder_mead_2d.rs`
- 目標測試檔: `tests/test_math_nelder_mead_2d.rs`
- 功能簡述: 二維無約束最佳化 Nelder-Mead 單純形法 (Simplex Method) 頂點幾何轉換純函式。包含最佳兩點質心 (Centroid $x_0$)、反射點 (Reflection $x_r$) 與擴張點 (Expansion $x_e$) 計算。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 2D Nelder-Mead simplex reflection, expansion, and contraction transforms
// Inputs: p1, p2, p_worst: [f64; 2], alpha, gamma: f64
// Outputs: Option<[f64; 2]> (new_point)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: points finite, alpha > 0.0
pub fn math_simplex_2d_reflect(
    p1: [f64; 2],
    p2: [f64; 2],
    p_worst: [f64; 2],
    alpha: f64,
) -> Option<[f64; 2]>;

/// @outputs: 1
/// @domain: x0, xr finite, gamma > 1.0
pub fn math_simplex_2d_expand(
    x0: [f64; 2],
    xr: [f64; 2],
    gamma: f64,
) -> Option<[f64; 2]>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何座標分量非有限數值，立即回傳 `None`。
- `math_simplex_2d_reflect`:
  - 若 `alpha <= 0.0`，立即回傳 `None`。
  - 質心：`x0 = [(p1[0] + p2[0]) / 2.0, (p1[1] + p2[1]) / 2.0]`
  - 反射點：`xr = [x0[0] + alpha * (x0[0] - p_worst[0]), x0[1] + alpha * (x0[1] - p_worst[1])]`
  - 回傳 `Some(xr)`。
- `math_simplex_2d_expand`:
  - 若 `gamma <= 1.0`，立即回傳 `None`。
  - 擴張點：`xe = [x0[0] + gamma * (xr[0] - x0[0]), x0[1] + gamma * (xr[1] - x0[1])]`
  - 回傳 `Some(xe)`。

## 4. 確定性黃金值測試要求
- 反射測試：
  - `p1 = [0.0, 0.0], p2 = [2.0, 0.0]` -> 質心 $x_0 = [1.0, 0.0]$
  - `p_worst = [1.0, 2.0], alpha = 1.0`
  - $x_r = [1.0 + 1.0*(1.0-1.0), 0.0 + 1.0*(0.0-2.0)] = [1.0, -2.0]$
- 擴張測試：
  - `x0 = [1.0, 0.0], xr = [1.0, -2.0], gamma = 2.0`
  - $x_e = [1.0, -4.0]$

```golden
let xr = math_simplex_2d_reflect([0.0, 0.0], [2.0, 0.0], [1.0, 2.0], 1.0).unwrap();
(xr[0] - 1.0).abs() < 1e-12
(xr[1] - (-2.0)).abs() < 1e-12
let xe = math_simplex_2d_expand([1.0, 0.0], [1.0, -2.0], 2.0).unwrap();
(xe[0] - 1.0).abs() < 1e-12
(xe[1] - (-4.0)).abs() < 1e-12
math_simplex_2d_reflect([0.0, 0.0], [2.0, 0.0], [1.0, 2.0], 0.0).is_none()
math_simplex_2d_expand([1.0, 0.0], [1.0, -2.0], 1.0).is_none()
```
