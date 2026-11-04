# 規格書：一維牛頓-拉夫森求根演算法 (1D Newton-Raphson Solver)

## 模組名稱
`math_newton_raphson_1d`

## 業務目標與適用場景
在工程非線性方程求解、隱式物理方程平衡點計算與金融隱含波動率 (Implied Volatility) 反推中，牛頓-拉夫森法 (Newton-Raphson Method) 具備二階局部收斂速度 ($O(\epsilon^2)$)，能在極少迭代次數內精確鎖定方程式之零點。
本模組提供零外部依賴、純 `std` 實作的一維牛頓求根微積木。

## 09 號合約邊界規範

### 函式 1：一維牛頓-拉夫森求根迭代 (Newton-Raphson Root Finding)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: tol [1.0e-12..1.0]
  /// @domain: max_iter [1..1000]
  pub fn newton_raphson_1d<F, DF>(
      initial_x: f64,
      tol: f64,
      max_iter: usize,
      f: F,
      df: DF,
  ) -> Option<f64>
  where
      F: Fn(f64) -> f64,
      DF: Fn(f64) -> f64,
  ```
- **運算過程**：
  - 邊界檢查：`!initial_x.is_finite() || tol < 1.0e-12 || tol > 1.0 || max_iter < 1 || max_iter > 1000` 回傳 `None`。
  - 令 `x = initial_x`。
  - 迴圈 $iter \in [0 .. max\_iter]$：
    - 計算函數值 $y = f(x)$。
    - 若 $|y| < tol$，收斂成功，提早回傳 `Some(x)`。
    - 計算導數 $dy = df(x)$。
    - 若 $|dy| < 1.0e-12$ 或 $dy$ 為 NaN，斜率過於平緩或除以零，回傳 `None`。
    - 更新步長：$step = y / dy$。
    - $x_{next} = x - step$。
    - 若 $|x_{next} - x| < tol$，收斂成功，提早回傳 `Some(x_{next})`。
    - $x = x_{next}$。
  - 超出最大迭代次數回傳 `None`。

## 黃金測試向量 (Golden Vectors)
- 求解 $f(x) = x^2 - 2.0 = 0$ ($x = \sqrt{2} \approx 1.41421356$)：
  - 導數 $f'(x) = 2x$。
  - 初值 $x_0 = 1.0$，$tol = 1.0e-6$，$max\_iter = 20$。
  - 搜尋結果 $x^* \approx 1.41421356$。
  - 斷言 `(root - 1.41421356).abs() < 1e-4`。
- 求解 $f(x) = \cos(x) - x = 0$ (Dottie 數 $x \approx 0.739085$)：
  - 導數 $f'(x) = -\sin(x) - 1.0$。
  - 初值 $x_0 = 0.5$，$tol = 1.0e-6$，$max\_iter = 20$。
  - 斷言 `(root - 0.739085).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_math_newton_raphson_1d.rs）
```golden
// 出處：規格向量 1，x^2 - 2 = 0，x0 = 1，tol 1e-6，max_iter 20 -> sqrt(2)（Python 獨立驗算 1.41421356237）
(newton_raphson_1d(1.0, 1.0e-6, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).unwrap() - 1.41421356).abs() < 1e-4
// 出處：規格向量 2，cos(x) - x = 0，x0 = 0.5 -> Dottie 數 0.7390851（Python 獨立驗算）
(newton_raphson_1d(0.5, 1.0e-6, 20, |x: f64| x.cos() - x, |x: f64| -x.sin() - 1.0).unwrap() - 0.739085).abs() < 1e-4
// 出處：規格邊界檢查（initial_x 非有限、tol 不在 [1e-12, 1]、max_iter 不在 [1, 1000]）
newton_raphson_1d(f64::NAN, 1.0e-6, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none()
newton_raphson_1d(1.0, 1.0e-13, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none()
newton_raphson_1d(1.0, 2.0, 20, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none()
newton_raphson_1d(1.0, 1.0e-6, 0, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none()
newton_raphson_1d(1.0, 1.0e-6, 1001, |x: f64| x * x - 2.0, |x: f64| 2.0 * x).is_none()
// 出處：規格「|dy| < 1e-12 回傳 None」：x^2 + 1 於 x0 = 0，f = 1 未收斂且導數為 0（手算）
newton_raphson_1d(0.0, 1.0e-6, 20, |x: f64| x * x + 1.0, |x: f64| 2.0 * x).is_none()
```
