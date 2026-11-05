# 模組名稱: 精算風險克拉梅爾-倫德伯格破產概率模型 (Cramer-Lundberg Ruin Probability)

## 1. 目標定位
- 目標模組檔: `src/fin_ruin_probability.rs`
- 目標測試檔: `tests/test_fin_ruin_probability.rs`
- 功能簡述: 在保險精算與量化做市商風控中，針對指數型損失分佈，解析求解 Cramér-Lundberg 模型下之破產機率 $\psi(u)$ 與倫德伯格調節係數。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Analytical Cramer-Lundberg ruin probability for exponential claims
// Inputs: initial_capital: f64, safety_loading: f64, mean_claim: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: initial_capital >= 0.0, safety_loading > 0.0, mean_claim > 0.0
pub fn fin_ruin_probability_exponential(
    initial_capital: f64,
    safety_loading: f64,
    mean_claim: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `initial_capital < 0.0 || safety_loading <= 0.0 || mean_claim <= 0.0` 或任何參數非有限數，立即回傳 `None`。
- 解析公式：
  $$R = \frac{\text{safety\_loading}}{\text{mean\_claim} \times (1.0 + \text{safety\_loading})}$$
  $$\psi(u) = \frac{1.0}{1.0 + \text{safety\_loading}} \times \exp(-R \times \text{initial\_capital})$$
  回傳 `Some(\psi(u))`。

## 4. 確定性黃金值測試要求
- 零初始本金基準：`u = 0.0, safety_loading = 0.25, mean_claim = 100.0`
  $$\psi(0) = \frac{1.0}{1.25} = 0.80$$
- 本金 $u = 500.0$：
  $R = 0.25 / 125.0 = 0.002$
  $$\psi(500) = 0.80 \times \exp(-0.002 \times 500) = 0.80 \times \exp(-1.0) \approx 0.2943036$$

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_ruin_probability.rs）
```golden
(fin_ruin_probability_exponential(0.0, 0.25, 100.0).unwrap() - 0.80).abs() < 1e-10
(fin_ruin_probability_exponential(500.0, 0.25, 100.0).unwrap() - 0.2943036).abs() < 1e-6
fin_ruin_probability_exponential(-10.0, 0.25, 100.0).is_none()
fin_ruin_probability_exponential(100.0, 0.0, 100.0).is_none()
fin_ruin_probability_exponential(100.0, 0.25, 0.0).is_none()
```
