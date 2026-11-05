# 模組名稱: 機器人運動學標準 DH 參數齊次轉換矩陣計算器 (Robotics Standard DH Homogeneous Transformation Matrix)

## 1. 目標定位
- 目標模組檔: `src/robotics_dh_transform.rs`
- 目標測試檔: `tests/test_robotics_dh_transform.rs`
- 功能簡述: 串聯機械手臂與工業機器人順向運動學 (Forward Kinematics) 之標準 Denavit-Hartenberg (DH) 參數 4x4 齊次轉換矩陣計算純函式。依標準庫三角函式精確計算，符合 GEMINI 鐵律第 1 與第 10 條。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Standard Denavit-Hartenberg (DH) 4x4 homogeneous transformation matrix
// Inputs: theta: f64, d: f64, a: f64, alpha: f64
// Outputs: Option<[f64; 16]> (row-major 4x4 matrix)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: theta in [-6.2831853..=6.2831853], d in [-1e4..=1e4], a in [-1e4..=1e4], alpha in [-6.2831853..=6.2831853]
pub fn robotics_dh_matrix(theta: f64, d: f64, a: f64, alpha: f64) -> Option<[f64; 16]>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `theta.abs() > 2.0 * std::f64::consts::PI + 1e-4` 或 `alpha.abs() > 2.0 * std::f64::consts::PI + 1e-4`，立即回傳 `None`。
- 計算：
  - `ct = theta.cos(); st = theta.sin();`
  - `ca = alpha.cos(); sa = alpha.sin();`
  - 填入 4x4 矩陣 (列優先 Row-Major)：
    - Row 0: `[ct, -st * ca,  st * sa, a * ct]`
    - Row 1: `[st,  ct * ca, -ct * sa, a * st]`
    - Row 2: `[0.0, sa,      ca,      d]`
    - Row 3: `[0.0, 0.0,     0.0,     1.0]`
- 回傳 `Some([m0, m1, ..., m15])`。

## 4. 確定性黃金值測試要求
- 基準恆等檢驗：`theta = 0, d = 0, a = 0, alpha = 0` $\rightarrow$ 必須為單位矩陣 $I_{4\times 4}$。
- 平移與旋轉：`theta = 0, d = 10.0, a = 5.0, alpha = 0` $\rightarrow$ 旋轉部分為單位陣，平移為 $(5, 0, 10)$。

```golden
let id = robotics_dh_matrix(0.0, 0.0, 0.0, 0.0).unwrap();
(id[0] - 1.0).abs() < 1e-12
(id[5] - 1.0).abs() < 1e-12
(id[10] - 1.0).abs() < 1e-12
(id[15] - 1.0).abs() < 1e-12
(id[3] - 0.0).abs() < 1e-12
let m = robotics_dh_matrix(0.0, 10.0, 5.0, 0.0).unwrap();
(m[3] - 5.0).abs() < 1e-12
(m[11] - 10.0).abs() < 1e-12
robotics_dh_matrix(10.0, 0.0, 0.0, 0.0).is_none()
robotics_dh_matrix(f64::NAN, 0.0, 0.0, 0.0).is_none()
```
