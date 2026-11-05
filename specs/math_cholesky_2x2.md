# 模組名稱: 2x2 對稱正定矩陣 Cholesky 分解演算法 (2x2 Symmetric Matrix Cholesky Decomposition)

## 1. 目標定位
- 目標模組檔: `src/math_cholesky_2x2.rs`
- 目標測試檔: `tests/test_math_cholesky_2x2.rs`
- 功能簡述: 卡爾曼濾波共變異數矩陣更新、高斯過程與機器人狀態估計之 2x2 對稱正定矩陣 Cholesky 下三角分解 ($A = L \cdot L^T$)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 2x2 Symmetric positive-definite matrix Cholesky decomposition (A = L * L^T)
// Inputs: a, b, c: f64 (Matrix: [[a, b], [b, c]])
// Outputs: Option<(f64, f64, f64)> (l11, l21, l22)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: a > 0.0, a * c - b * b > 0.0
pub fn math_cholesky_decompose_2x2(
    a: f64,
    b: f64,
    c: f64,
) -> Option<(f64, f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `a <= 0.0`，立即回傳 `None` (左上角主子式非正)。
- 計算行列式：`det = a * c - b * b`。
  若 `det <= 0.0`，立即回傳 `None` (非正定矩陣)。
- 計算下三角矩陣 $L$ 元素：
  - `l11 = a.sqrt()`
  - `l21 = b / l11`
  - `let d22 = c - l21 * l21;`
  - 若 `d22 <= 0.0` 則 `None`，否則 `l22 = d22.sqrt()`。
- 回傳 `Some((l11, l21, l22))`。

## 4. 確定性黃金值測試要求
- 矩陣 $A = \begin{bmatrix} 4 & 2 \\ 2 & 5 \end{bmatrix}$：
  - $l_{11} = \sqrt{4} = 2.0$
  - $l_{21} = 2 / 2 = 1.0$
  - $l_{22} = \sqrt{5 - 1^2} = \sqrt{4} = 2.0$
  - 驗證 $L L^T = \begin{bmatrix} 2 & 0 \\ 1 & 2 \end{bmatrix} \begin{bmatrix} 2 & 1 \\ 0 & 2 \end{bmatrix} = \begin{bmatrix} 4 & 2 \\ 2 & 5 \end{bmatrix}$ (完美吻合)。
- 非正定矩陣（如行列式為負或 $a \le 0$）回傳 `None`。

```golden
let (l11, l21, l22) = math_cholesky_decompose_2x2(4.0, 2.0, 5.0).unwrap();
(l11 - 2.0).abs() < 1e-12
(l21 - 1.0).abs() < 1e-12
(l22 - 2.0).abs() < 1e-12
math_cholesky_decompose_2x2(-1.0, 0.0, 1.0).is_none()
math_cholesky_decompose_2x2(1.0, 2.0, 1.0).is_none()
```
