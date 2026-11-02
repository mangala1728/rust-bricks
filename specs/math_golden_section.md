# 規格書：一維黃金分割極小值搜尋 (Golden-Section Search 1D)

## 模組名稱
`math_golden_section`

## 業務目標與適用場景
在無梯度最佳化、超參數調優與聲學共振頻率自適應搜尋中，當目標函式不可微或導數昂貴時，黃金分割搜尋法 (Golden-Section Search) 能保證以黃金比例 $\phi = (\sqrt{5}-1)/2 \approx 0.6180339887$ 縮小單峰區間 $[a, b]$。
本模組提供零外部依賴、純 `std` 實作的單峰極小值搜尋微積木。

## 09 號合約邊界規範

### 函式 1：一維單峰函式黃金分割極小值搜尋
- **函式簽名**：
  ```rust
  /// @outputs: 2
  /// @domain: tol [1.0e-12..1.0]
  /// @domain: max_iter [1..1000]
  pub fn golden_section_search_min<F>(
      mut a: f64,
      mut b: f64,
      tol: f64,
      max_iter: usize,
      f: F,
  ) -> Option<(f64, f64)>
  where
      F: Fn(f64) -> f64,
  ```
- **運算過程**：
  - 若 `a >= b` 或 `tol <= 0.0` 或 `max_iter == 0`，回傳 `None`。
  - 令比例縮減因子 $R = (\sqrt{5.0} - 1.0) / 2.0 \approx 0.618033988749895$。
  - 初始化內點：
    $c = b - R \cdot (b - a)$
    $d = a + R \cdot (b - a)$
    $f_c = f(c), \quad f_d = f(d)$
  - 迭代：
    若 $f_c < f_d$，則 $b = d, d = c, f_d = f_c, c = b - R \cdot (b - a), f_c = f(c)$
    否則 $a = c, c = d, f_c = f_d, d = a + R \cdot (b - a), f_d = f(d)$
    當 $(b - a) < tol$ 提前終止。
  - 回傳：`Some(((a + b) / 2.0, f((a + b) / 2.0)))`。

## 黃金測試向量 (Golden Vectors)
- 測試目標函式：$f(x) = (x - 2.0)^2 + 3.0$，理論極小值點在 $x = 2.0$，$f(2.0) = 3.0$。
- 初始區間 $[0.0, 5.0]$，$tol = 1.0e-5$，$max\_iter = 100$：
  - 搜尋結果 $x^* \approx 2.0$，$f(x^*) \approx 3.0$。
- 容差使用 `(x_opt - 2.0).abs() < 1e-4` 與 `(y_opt - 3.0).abs() < 1e-4`。
