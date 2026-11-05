# 模組名稱: Merton 跳躍擴散模型布瓦松權重與漂移修正器 (Merton Jump Diffusion Poisson Weights and Drift)

## 1. 目標定位
- 目標模組檔: `src/fin_merton_jump_diffusion.rs`
- 目標測試檔: `tests/test_fin_merton_jump_diffusion.rs`
- 功能簡述: 選擇權定價與極端市場風險度量之 Merton (1976) 跳躍擴散模型 (Jump-Diffusion) 核心組件。計算平均跳躍幅度 $\kappa$、補償漂移項、以及第 $n$ 次跳躍之布瓦松機率權重 $P(N_T = n) = e^{-\lambda T} (\lambda T)^n / n!$。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Merton Jump Diffusion jump amplitude and Poisson weight calculator
// Inputs: lambda, t, mu_j, delta: f64, n: u32
// Outputs: Option<f64> or Option<(f64, f64)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: lambda >= 0.0, t > 0.0, n <= 30
pub fn fin_merton_poisson_weight(lambda: f64, t: f64, n: u32) -> Option<f64>;

/// @outputs: 2
/// @domain: lambda >= 0.0, mu_j in [-2.0..=2.0], delta >= 0.0
pub fn fin_merton_jump_parameters(lambda: f64, mu_j: f64, delta: f64) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `fin_merton_poisson_weight`:
  - 若 `lambda < 0.0 || t <= 0.0 || n > 30` 或含 NaN，立即回傳 `None`。
  - `let lt = lambda * t;`
  - 若 `lt < 1e-15`：若 `n == 0` 回傳 `Some(1.0)` 否則 `Some(0.0)`。
  - 對數階乘穩定運算：
    - `ln_weight = -lt + (n as f64) * lt.ln() - ln_factorial(n)`
    - 其中階乘可由顯式迴圈計算對數求和。
  - 回傳 `Some(ln_weight.exp())`。
- `fin_merton_jump_parameters`:
  - 若 `lambda < 0.0 || delta < 0.0 || !mu_j.is_finite()`，立即回傳 `None`。
  - 平均跳躍比例：$\kappa = \exp(\mu_j + \delta^2 / 2) - 1.0$。
  - 調整後跳躍強度：$\lambda' = \lambda \times (1.0 + \kappa)$。
  - 回傳 `Some((kappa, lambda_prime))`。

## 4. 確定性黃金值測試要求
- 當 $\lambda = 0$ (無跳躍)：
  - $n=0$ 時權重精確為 $1.0$；$n=1$ 時權重精確為 $0.0$。
- 當 $\mu_j = 0, \delta = 0$：
  - $\kappa = e^0 - 1 = 0.0$
  - $\lambda' = \lambda$
- $n > 30$ 或負強度拒絕。

```golden
let w0 = fin_merton_poisson_weight(0.0, 1.0, 0).unwrap();
(w0 - 1.0).abs() < 1e-12
let w1 = fin_merton_poisson_weight(0.0, 1.0, 1).unwrap();
(w1 - 0.0).abs() < 1e-12
let (k, lp) = fin_merton_jump_parameters(2.0, 0.0, 0.0).unwrap();
(k - 0.0).abs() < 1e-12
(lp - 2.0).abs() < 1e-12
fin_merton_poisson_weight(-1.0, 1.0, 0).is_none()
fin_merton_poisson_weight(1.0, 1.0, 31).is_none()
```
