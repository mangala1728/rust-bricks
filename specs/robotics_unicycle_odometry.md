# 模組名稱: 輪式差速移動機器人航位推算單步演算法 (Unicycle Dead Reckoning Odometry Step)

## 1. 目標定位
- 目標模組檔: `src/robotics_unicycle_odometry.rs`
- 目標測試檔: `tests/test_robotics_unicycle_odometry.rs`
- 功能簡述: 雙輪差速 AGV / AMR 移動機器人基於線速度 $v$ 與角速度 $\omega$ 之高精度圓弧積分航位推算 (Odometry Dead Reckoning) 單步純函式。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Differential drive unicycle mobile robot dead reckoning odometry
// Inputs: x, y, theta, v, omega, dt: f64
// Outputs: Option<(f64, f64, f64)> (next_x, next_y, next_theta_rad)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: dt > 0.0, v in [-100.0..=100.0], omega in [-50.0..=50.0], theta in [-6.2831853..=6.2831853]
pub fn robotics_unicycle_step(
    x: f64,
    y: f64,
    theta: f64,
    v: f64,
    omega: f64,
    dt: f64,
) -> Option<(f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `dt <= 0.0 || v.abs() > 100.0 || omega.abs() > 50.0`，立即回傳 `None`。
- 運算法則：
  - 若 `omega.abs() < 1e-7` (直行直線運動)：
    - `next_x = x + v * theta.cos() * dt`
    - `next_y = y + v * theta.sin() * dt`
    - `next_th = theta`
  - 否則 (精確圓弧積分)：
    - `d_th = omega * dt`
    - `next_x = x + (v / omega) * ((theta + d_th).sin() - theta.sin())`
    - `next_y = y - (v / omega) * ((theta + d_th).cos() - theta.cos())`
    - `next_th = theta + d_th`
  - 角度正規化至 $[-\pi, \pi]$：
    - 使用標準庫 `rem_euclid` 或歐幾里得取模將 `next_th` 映射至 $[-\pi, \pi]$。
- 回傳 `Some((next_x, next_y, next_th))`。

## 4. 確定性黃金值測試要求
- 直線行進：`x=0, y=0, theta=0, v=2.0, omega=0, dt=1.0` $\rightarrow$ `(2.0, 0.0, 0.0)`
- 原地自轉：`x=0, y=0, theta=0, v=0, omega=1.57079632679, dt=1.0` $\rightarrow$ `(0.0, 0.0, 1.57079632679)` (90度)

```golden
let (x1, y1, th1) = robotics_unicycle_step(0.0, 0.0, 0.0, 2.0, 0.0, 1.0).unwrap();
(x1 - 2.0).abs() < 1e-12
(y1 - 0.0).abs() < 1e-12
(th1 - 0.0).abs() < 1e-12
let (x2, y2, th2) = robotics_unicycle_step(0.0, 0.0, 0.0, 0.0, 1.5707963267948966, 1.0).unwrap();
(x2 - 0.0).abs() < 1e-12
(y2 - 0.0).abs() < 1e-12
(th2 - 1.5707963267948966).abs() < 1e-12
robotics_unicycle_step(0.0, 0.0, 0.0, 2.0, 0.0, 0.0).is_none()
```
