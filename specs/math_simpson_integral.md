# 規格書：辛普森複合數值積分法 (Simpson's 1/3 Rule 1D)

## 模組名稱
`math_simpson_integral`

## 業務目標與適用場景
在工程數值模擬、物理動態面積求解、機率密度分佈累加中，當目標函式無解析原函數時，辛普森複合 1/3 積分公式以拋物線擬合，具備比梯形法高出一個收斂階數的精度 ($O(h^4)$)。
本模組提供零外部依賴、純 `std` 實作的一維辛普森數值積分微積木。

## 09 號合約邊界規範

### 函式 1：辛普森複合一維數值積分
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: intervals [2..100000]
  pub fn simpson_integral_1d<F>(
      a: f64,
      b: f64,
      intervals: usize,
      f: F,
  ) -> Option<f64>
  where
      F: Fn(f64) -> f64,
  ```
- **運算過程**：
  - 若 `a >= b || intervals < 2 || intervals > 100000 || !a.is_finite() || !b.is_finite()` 回傳 `None`。
  - 若 `intervals % 2 != 0`，向上補齊為偶數：`let n = intervals + 1;`
  - 步長 $h = (b - a) / n$。
  - 初值：$s = f(a) + f(b)$。
  - 對 $i \in [1 .. n-1]$：
    - $x = a + i \cdot h$
    - 若 $i$ 為奇數則加 $4 \cdot f(x)$，若為偶數則加 $2 \cdot f(x)$。
  - 積分值 $I = s \cdot \frac{h}{3.0}$。
  - 若 $I$ 為 NaN 或 Inf 回傳 `None`，否則回傳 `Some(I)`。

## 黃金測試向量 (Golden Vectors)
- 積分目標：$\int_0^\pi \sin(x) dx = [-\cos(x)]_0^\pi = -(-1) - (-1) = 2.0$。
- 區間 $[0.0, \pi]$，$intervals = 100$：
  - 積分結果 $I \approx 2.0000$。
- 斷言 `(integral - 2.0).abs() < 1e-4`。
- 多項式積分：$\int_0^3 x^2 dx = [x^3 / 3]_0^3 = 9.0$。
  - 辛普森法對二次多項式為代數精確值：斷言 `(integral - 9.0).abs() < 1e-6`。
