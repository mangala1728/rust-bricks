# 模組名稱: GARCH(1,1) 條件異質變異數步進模型 (GARCH(1,1) Volatility Model)

## 1. 目標定位
- 目標模組檔: `src/fin_garch11.rs`
- 目標測試檔: `tests/test_fin_garch11.rs`
- 功能簡述: 實作計量金融標準 Bollerslev GARCH(1,1) 波動率聚類動態更新與長期無條件變異數解析計算。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: GARCH(1,1) conditional variance recursion and long-term equilibrium variance
// Inputs: omega, alpha, beta, prev_var, prev_return: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: omega > 0.0, alpha >= 0.0, beta >= 0.0, alpha + beta < 1.0, prev_var > 0.0
pub fn fin_garch11_step(
    omega: f64,
    alpha: f64,
    beta: f64,
    prev_var: f64,
    prev_return: f64,
) -> Option<f64>;

/// @outputs: 1
/// @domain: omega > 0.0, alpha >= 0.0, beta >= 0.0, alpha + beta < 1.0
pub fn fin_garch11_unconditional_variance(omega: f64, alpha: f64, beta: f64) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `omega <= 0.0 || alpha < 0.0 || beta < 0.0 || (alpha + beta) >= 1.0` 或任何參數非有限數，立即回傳 `None`（弱平穩性保證）。
- `fin_garch11_step`: 若 `prev_var <= 0.0` 回傳 `None`。
  $$\sigma_t^2 = \omega + \alpha \times \text{prev\_return}^2 + \beta \times \text{prev\_var}$$
- `fin_garch11_unconditional_variance`:
  $$V_L = \frac{\omega}{1.0 - \alpha - \beta}$$

## 4. 確定性黃金值測試要求
- 參數：`omega = 0.000002, alpha = 0.08, beta = 0.90`
- 長期變異數：$0.000002 / (1.0 - 0.98) = 0.0001$
- 單步更新：`prev_var = 0.0001, prev_return = 0.02`
  $$\sigma_t^2 = 0.000002 + 0.08 \times 0.0004 + 0.90 \times 0.0001 = 0.000124$$
- 非法邊界：$\alpha + \beta \ge 1.0$ 回傳 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_garch11.rs）
```golden
let vl = fin_garch11_unconditional_variance(0.000002, 0.08, 0.90).unwrap();
(vl - 0.0001).abs() < 1e-8
let v_step = fin_garch11_step(0.000002, 0.08, 0.90, 0.0001, 0.02).unwrap();
(v_step - 0.000124).abs() < 1e-8
fin_garch11_step(0.000002, 0.50, 0.60, 0.0001, 0.02).is_none()
fin_garch11_step(0.0, 0.08, 0.90, 0.0001, 0.02).is_none()
fin_garch11_step(0.000002, 0.08, 0.90, -0.0001, 0.02).is_none()
```
