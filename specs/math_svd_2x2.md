# 模組名稱: 二階實數矩陣奇異值分解 (2x2 Matrix SVD)

## 1. 目標定位
- 目標模組檔: `src/math_svd_2x2.rs`
- 目標測試檔: `tests/test_math_svd_2x2.rs`
- 功能簡述: 針對 2x2 實數矩陣 $\begin{bmatrix} a & b \\ c & d \end{bmatrix}$ 提供解析解奇異值 $(\sigma_1, \sigma_2)$ 與條件數（Condition Number $\kappa = \sigma_1 / \sigma_2$）計算。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Analytical 2x2 SVD singular values and condition number
// Inputs: a, b, c, d: f64
// Outputs: Option<(f64, f64)> or Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: a, b, c, d are finite
pub fn math_svd_2x2_singular_values(a: f64, b: f64, c: f64, d: f64) -> Option<(f64, f64)>;

/// @outputs: 1
/// @domain: a, b, c, d are finite
pub fn math_matrix_2x2_condition_number(a: f64, b: f64, c: f64, d: f64) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `a, b, c, d` 任一非有限數，立即回傳 `None`。
- 解析公式：
  $$S_1 = a^2 + b^2 + c^2 + d^2$$
  $$\text{det} = a \times d - b \times c$$
  $$S_2 = \sqrt{\max(0.0, S_1^2 - 4 \times \text{det}^2)}$$
  $$\sigma_1 = \sqrt{0.5 \times (S_1 + S_2)}$$
  $$\sigma_2 = \sqrt{0.5 \times \max(0.0, S_1 - S_2)}$$
- `math_matrix_2x2_condition_number`: 若 $\sigma_2 \le 10^{-12}$，回傳 `None`（退化矩陣條件數無窮大）；否則回傳 `Some(\sigma_1 / \sigma_2)`。

## 4. 確定性黃金值測試要求
- 單位矩陣 `a=1, b=0, c=0, d=1`: $\sigma_1 = 1.0, \sigma_2 = 1.0$, $\kappa = 1.0$
- 對角矩陣 `a=3, b=0, c=0, d=2`: $\sigma_1 = 3.0, \sigma_2 = 2.0$, $\kappa = 1.5$
- 奇異矩陣 `a=1, b=2, c=2, d=4`: $\sigma_1 = 5.0, \sigma_2 = 0.0$, $\kappa = None$

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_math_svd_2x2.rs）
```golden
let (s1, s2) = math_svd_2x2_singular_values(1.0, 0.0, 0.0, 1.0).unwrap();
(s1 - 1.0).abs() < 1e-10 && (s2 - 1.0).abs() < 1e-10
(math_matrix_2x2_condition_number(1.0, 0.0, 0.0, 1.0).unwrap() - 1.0).abs() < 1e-10
let (s1, s2) = math_svd_2x2_singular_values(3.0, 0.0, 0.0, 2.0).unwrap();
(s1 - 3.0).abs() < 1e-10 && (s2 - 2.0).abs() < 1e-10
(math_matrix_2x2_condition_number(3.0, 0.0, 0.0, 2.0).unwrap() - 1.5).abs() < 1e-10
let (s1, s2) = math_svd_2x2_singular_values(1.0, 2.0, 2.0, 4.0).unwrap();
(s1 - 5.0).abs() < 1e-10 && s2.abs() < 1e-10
math_matrix_2x2_condition_number(1.0, 2.0, 2.0, 4.0).is_none()
math_svd_2x2_singular_values(f64::NAN, 0.0, 0.0, 1.0).is_none()
```
