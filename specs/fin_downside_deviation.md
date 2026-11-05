# 模組名稱: 下行標準差與上行潛力比率 (Downside Deviation & Upside Potential)

## 1. 目標定位
- 目標模組檔: `src/fin_downside_deviation.rs`
- 目標測試檔: `tests/test_fin_downside_deviation.rs`
- 功能簡述: 在非對稱風險管理與後現代投資組合理論（PMPT）中，計算低於最低可接受報酬率（MAR）之下行半方差標準差與上行潛力比率（Upside Potential Ratio）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Downside semi-deviation and upside potential ratio below MAR target
// Inputs: returns: &[f64], target_mar: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: returns.len() >= 1, target_mar is finite, elements are finite
pub fn fin_downside_deviation(returns: &[f64], target_mar: f64) -> Option<f64>;

/// @outputs: 1
/// @domain: returns.len() >= 1, target_mar is finite, downside_dev > 1e-12
pub fn fin_upside_potential_ratio(returns: &[f64], target_mar: f64) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `returns.is_empty() || !target_mar.is_finite()` 或任何報酬元素非有限數，回傳 `None`。
- 下行偏差計算：
  $$\delta_{\text{down}} = \sqrt{\frac{1}{N} \sum_{i=1}^N \min(0.0, r_i - \text{target\_mar})^2}$$
- 上行潛力比率：
  $$\text{Upside} = \frac{1}{N} \sum_{i=1}^N \max(0.0, r_i - \text{target\_mar})$$
  若 $\delta_{\text{down}} \le 10^{-12}$，回傳 `None`；否則回傳 `Some(\text{Upside} / \delta_{\text{down}})`。

## 4. 確定性黃金值測試要求
- 數據：`returns = [0.10, -0.05, 0.05, -0.10]`, `target_mar = 0.0`
  - 負偏差：$(-0.05)^2 + (-0.10)^2 = 0.0125$
  - 下行變異數：$0.0125 / 4 = 0.003125$
  - $\delta_{\text{down}} = \sqrt{0.003125} \approx 0.0559017$
  - 上行均值：$(0.10 + 0.05) / 4 = 0.0375$
  - 上行潛力比率：$0.0375 / 0.0559017 \approx 0.670820$

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_downside_deviation.rs）
```golden
let r = [0.10, -0.05, 0.05, -0.10];
(fin_downside_deviation(&r, 0.0).unwrap() - 0.0559017).abs() < 1e-5
(fin_upside_potential_ratio(&r, 0.0).unwrap() - 0.670820).abs() < 1e-5
fin_downside_deviation(&[], 0.0).is_none()
fin_downside_deviation(&[0.05, f64::NAN], 0.0).is_none()
fin_upside_potential_ratio(&[0.10, 0.20], 0.0).is_none()
```
