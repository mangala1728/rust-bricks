# 模組名稱: 四元數球面線性插值 SLERP 運算器 (Unit Quaternion SLERP Interpolator)

## 1. 目標定位
- 目標模組檔: `src/robotics_quaternion_slerp.rs`
- 目標測試檔: `tests/test_robotics_quaternion_slerp.rs`
- 功能簡述: 機器人與無人機姿態旋轉之四元數球面線性插值 (SLERP)。自動處理最短路徑反轉 (Shortest Path Inversion, $\text{dot} < 0$) 與接近平行時的 NLERP 數值穩定平滑過渡。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Unit quaternion spherical linear interpolation (SLERP)
// Inputs: q1: [f64; 4], q2: [f64; 4], t: f64 (format: [w, x, y, z])
// Outputs: Option<[f64; 4]>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: q1 and q2 are unit quaternions (norm approx 1.0), t in [0.0..=1.0]
pub fn robotics_quat_slerp(q1: [f64; 4], q2: [f64; 4], t: f64) -> Option<[f64; 4]>;
```

## 3. 邊界與异常條件 (Domain Guards)
- 若任何分量非有限數值或 `t < 0.0 || t > 1.0`，立即回傳 `None`。
- 檢查範數：`norm1_sq = sum(q1[i]^2), norm2_sq = sum(q2[i]^2)`。
  若 `(norm1_sq - 1.0).abs() > 1e-3 || (norm2_sq - 1.0).abs() > 1e-3`，立即回傳 `None`。
- 計算內積：`let mut dot = q1[0]*q2[0] + q1[1]*q2[1] + q1[2]*q2[2] + q1[3]*q2[3];`
- 最短弧翻轉：若 `dot < 0.0`，則 `q2_adj = [-q2[0], -q2[1], -q2[2], -q2[3]]; dot = -dot;` 否則 `q2_adj = q2;`。
- 限幅 `dot = dot.clamp(-1.0, 1.0)`。
- 近似線性過渡：若 `dot > 0.9995`：
  - `cand = [ (1.0-t)*q1[i] + t*q2_adj[i] ]`
  - 歸一化 `cand` 並回傳。
- 否則：
  - `theta = dot.acos();`
  - `sin_theta = theta.sin();`
  - `w1 = ((1.0 - t) * theta).sin() / sin_theta;`
  - `w2 = (t * theta).sin() / sin_theta;`
  - `res = [ w1 * q1[i] + w2 * q2_adj[i] ];`
  - 歸一化並回傳 `Some(res)`。

## 4. 確定性黃金值測試要求
- 恆等檢驗：`t = 0.0` 時輸出精確為 `q1`；`t = 1.0` 時輸出精確為 `q2`。
- 中點插值：`q1 = [1, 0, 0, 0]` (0度), `q2 = [0, 1, 0, 0]` (繞X軸180度)，`t = 0.5` $\rightarrow$ `[0.70710678, 0.70710678, 0, 0]` (繞X軸90度)。

```golden
let q1 = [1.0, 0.0, 0.0, 0.0];
let q2 = [0.0, 1.0, 0.0, 0.0];
let mid = robotics_quat_slerp(q1, q2, 0.5).unwrap();
(mid[0] - 0.7071067811865475).abs() < 1e-6
(mid[1] - 0.7071067811865475).abs() < 1e-6
let start = robotics_quat_slerp(q1, q2, 0.0).unwrap();
(start[0] - 1.0).abs() < 1e-6
robotics_quat_slerp(q1, q2, 1.5).is_none()
robotics_quat_slerp([2.0, 0.0, 0.0, 0.0], q2, 0.5).is_none()
```
