# 規格書：四階龍格-庫塔常微分方程求解單步 (RK4 ODE Step)

## 模組名稱
`math_rk4_ode`

## 業務目標與適用場景
在無人機動力學模擬、機器人關節軌跡規劃與物理引擎剛體模擬中，常微分方程（ODE）數值求解是動態系統預測的核心。
本模組提供經典四階 Runge-Kutta (RK4) 零外部依賴單步求解器。

## 09 號合約邊界規範

### 函式 1：純量一維 RK4 步進求解器
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: dt [1.0e-9..1000.0]
  pub fn rk4_step_1d<F>(t: f64, y: f64, dt: f64, derivative_fn: F) -> Option<f64>
  where
      F: Fn(f64, f64) -> f64,
  ```
- **數學公式**：
  - $k_1 = f(t, y)$
  - $k_2 = f(t + 0.5 dt, y + 0.5 dt \cdot k_1)$
  - $k_3 = f(t + 0.5 dt, y + 0.5 dt \cdot k_2)$
  - $k_4 = f(t + dt, y + dt \cdot k_3)$
  - $y_{n+1} = y + \frac{dt}{6} (k_1 + 2 k_2 + 2 k_3 + k_4)$

### 函式 2：二維狀態向量 (2D State Vector) RK4 步進求解器 (適用於二階系統 $x, \dot{x}$)
- **函式簽名**：
  ```rust
  /// @outputs: 2
  /// @domain: dt [1.0e-9..1000.0]
  pub fn rk4_step_2d<F>(
      t: f64,
      y: (f64, f64),
      dt: f64,
      derivative_fn: F,
  ) -> Option<(f64, f64)>
  where
      F: Fn(f64, (f64, f64)) -> (f64, f64),
  ```

## 黃金測試向量 (Golden Vectors)
- 指數衰減方程：$dy/dt = -y$，解析解 $y(t) = y_0 e^{-t}$。
  - $y_0 = 1.0, dt = 0.1, t = 0$:
  - $k_1 = -1.0$
  - $k_2 = -(1 - 0.05) = -0.95$
  - $k_3 = -(1 - 0.0475) = -0.9525$
  - $k_4 = -(1 - 0.09525) = -0.90475$
  - $y_1 = 1.0 + \frac{0.1}{6} (-1.0 - 1.90 - 1.905 - 0.90475) = 1.0 - 0.0951625 = 0.9048375$
  - 理論精確值 $e^{-0.1} \approx 0.904837418$
- 測試容差 `(y1 - 0.9048375).abs() < 1e-6`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_math_rk4_ode.rs）
```golden
// 出處：規格向量 dy/dt = -y，y0 = 1，dt = 0.1 -> 0.9048375（規格手算 k1..k4，Python 獨立驗算）
(rk4_step_1d(0.0, 1.0, 0.1, |_t: f64, y: f64| -y).unwrap() - 0.9048375).abs() < 1e-6
// 出處：規格 @domain: dt [1.0e-9..1000.0]，區間外拒絕
rk4_step_1d(0.0, 1.0, 0.0, |_t: f64, y: f64| -y).is_none()
rk4_step_1d(0.0, 1.0, 1001.0, |_t: f64, y: f64| -y).is_none()
rk4_step_2d(0.0, (1.0, 0.0), 0.0, |_t: f64, y: (f64, f64)| (y.1, -y.0)).is_none()
```
