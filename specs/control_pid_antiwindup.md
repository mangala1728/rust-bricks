# 模組名稱: 帶條件抗積分飽和夾止之離散 PID 控制器 (Discrete PID Controller with Anti-Windup Clamping)

## 1. 目標定位
- 目標模組檔: `src/control_pid_antiwindup.rs`
- 目標測試檔: `tests/test_control_pid_antiwindup.rs`
- 功能簡述: 工業自動化、無人機與機器人致動器之離散 PID 控制單步演算法。具備條件積分抗飽和 (Conditional Integration Anti-Windup) 與輸出限幅夾止，嚴格符合 GEMINI 鐵律第 9 條。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Discrete PID step controller with clamping anti-windup protection
// Inputs: setpoint, measured, integral, prev_error, dt, kp, ki, kd, out_min, out_max: f64
// Outputs: Option<(f64, f64)> (output, next_integral)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: dt > 0.0, kp >= 0.0, ki >= 0.0, kd >= 0.0, out_min < out_max
pub fn control_pid_step(
    setpoint: f64,
    measured: f64,
    integral: f64,
    prev_error: f64,
    dt: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    out_min: f64,
    out_max: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `dt <= 0.0 || kp < 0.0 || ki < 0.0 || kd < 0.0 || out_min >= out_max`，立即回傳 `None`。
- 運算邏輯：
  - `error = setpoint - measured`
  - `p_term = kp * error`
  - `d_term = kd * (error - prev_error) / dt`
  - 暫存積分：`cand_integral = integral + error * dt`
  - 未限幅輸出：`raw_out = p_term + ki * cand_integral + d_term`
  - 限幅輸出：`let output = raw_out.clamp(out_min, out_max);`
  - 抗飽和條件積分：若 `(raw_out > out_max && error > 0.0) || (raw_out < out_min && error < 0.0)`，凍結積分增長 `next_integral = integral`；否則 `next_integral = cand_integral`。
- 回傳 `Some((output, next_integral))`。

## 4. 確定性黃金值測試要求
- 正常未飽和控制：
  - `sp = 10.0, measured = 8.0 (err=2.0), prev_err = 2.0, integral = 0.0, dt = 0.1`
  - `kp = 2.0, ki = 1.0, kd = 0.5, out_min = -100.0, out_max = 100.0`
  - `p = 4.0, d = 0.0, cand_i = 0.2, ki*i = 0.2` -> `raw = 4.2`
  - `output = 4.2, next_integral = 0.2`
- 輸出飽和抗積分累積測試：
  - `out_max = 3.0`，此時 `raw = 4.2 > 3.0` 且 `err = 2.0 > 0.0`
  - `output = 3.0, next_integral = 0.0` (成功凍結積分增長)

```golden
let (out1, next_i1) = control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -100.0, 100.0).unwrap();
(out1 - 4.2).abs() < 1e-12
(next_i1 - 0.2).abs() < 1e-12
let (out2, next_i2) = control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, -10.0, 3.0).unwrap();
(out2 - 3.0).abs() < 1e-12
(next_i2 - 0.0).abs() < 1e-12
control_pid_step(10.0, 8.0, 0.0, 2.0, 0.0, 2.0, 1.0, 0.5, -10.0, 10.0).is_none()
control_pid_step(10.0, 8.0, 0.0, 2.0, 0.1, 2.0, 1.0, 0.5, 10.0, -10.0).is_none()
```
