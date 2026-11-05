# 模組名稱: 帶邊界層抗抖振一維滑模控制器 (1D Sliding Mode Controller with Boundary Layer)

## 1. 目標定位
- 目標模組檔: `src/control_sliding_mode_1d.rs`
- 目標測試檔: `tests/test_control_sliding_mode_1d.rs`
- 功能簡述: 機器人非線性強健控制之滑模控制 (SMC) 單步計算純函式。採用飽和連續函數 (Saturation Function) 構造邊界層 (Boundary Layer $\epsilon$)，消除傳統符號函數 ($\text{sgn}$) 引起的致動器高頻抖振 (Chattering)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 1D Sliding mode control with boundary layer chattering alleviation
// Inputs: error, error_dot, c, k, epsilon: f64
// Outputs: Option<(f64, f64)> (sliding_surface_s, control_u)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: c > 0.0, k > 0.0, epsilon > 0.0
pub fn control_sliding_mode_step(
    error: f64,
    error_dot: f64,
    c: f64,
    k: f64,
    epsilon: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限數值，立即回傳 `None`。
- 若 `c <= 0.0 || k <= 0.0 || epsilon <= 0.0`，立即回傳 `None`。
- 計算：
  - 滑模面：`s = c * error + error_dot`
  - 邊界層歸一化：`norm_s = s / epsilon`
  - 飽和函數：
    - 若 `norm_s > 1.0` $\rightarrow$ `sat = 1.0`
    - 若 `norm_s < -1.0` $\rightarrow$ `sat = -1.0`
    - 否則 $\rightarrow$ `sat = norm_s`
  - 控制量：`u = k * sat`
- 回傳 `Some((s, u))`。

## 4. 確定性黃金值測試要求
- 邊界層內線性切換：
  - `error = 1.0, error_dot = -0.5, c = 1.0` -> `s = 0.5`
  - `k = 10.0, epsilon = 1.0` -> `norm_s = 0.5` -> `sat = 0.5` -> `u = 5.0`
- 邊界層外飽和切換：
  - `s = 2.0, epsilon = 1.0` -> `sat = 1.0` -> `u = 10.0`
- 無效 $\epsilon \le 0$ 拒絕。

```golden
let (s1, u1) = control_sliding_mode_step(1.0, -0.5, 1.0, 10.0, 1.0).unwrap();
(s1 - 0.5).abs() < 1e-12
(u1 - 5.0).abs() < 1e-12
let (s2, u2) = control_sliding_mode_step(2.0, 1.0, 1.0, 10.0, 1.0).unwrap();
(s2 - 3.0).abs() < 1e-12
(u2 - 10.0).abs() < 1e-12
control_sliding_mode_step(1.0, 0.0, 0.0, 10.0, 1.0).is_none()
control_sliding_mode_step(1.0, 0.0, 1.0, 10.0, 0.0).is_none()
```
