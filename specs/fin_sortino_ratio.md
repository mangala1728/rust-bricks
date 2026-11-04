# 規格書：下行半方差與索提諾比率 (Sortino Ratio & Downside Deviation)

## 模組名稱
`fin_sortino_ratio`

## 業務目標與適用場景
夏普比率 (Sharpe Ratio) 對向上波動和向下波動一視同仁地懲罰，但在實際資產管理中，投資者僅畏懼虧損（下行風險）。
索提諾比率 (Sortino Ratio) 以目標報酬率（或無風險利率）為基準，僅量度下行半方差 (Downside Semi-Variance)，更真實反映不對稱風險調整後績效。
本模組提供零外部依賴、純 `std` 實作的下行偏差與索提諾比率計算微積木。

## 09 號合約邊界規範

### 函式 1：計算下行偏差 (Downside Deviation / Semi-Deviation)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: target_return [-1.0..10.0]
  pub fn downside_deviation(returns: &[f64], target_return: f64) -> Option<f64>
  ```
- **數學公式**：
  - 下行半方差 $D^2 = \frac{1}{N} \sum_{i=0}^{N-1} \min(0.0, r_i - \text{target\_return})^2$
  - 下行偏差 $D = \sqrt{D^2}$
  - 若 `returns.is_empty()`，回傳 `None`。

### 函式 2：計算索提諾比率 (Sortino Ratio)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: risk_free_rate [-1.0..10.0]
  pub fn sortino_ratio(returns: &[f64], risk_free_rate: f64) -> Option<f64>
  ```
- **數學公式**：
  - 平均報酬率 $\bar{r} = \frac{1}{N} \sum r_i$
  - 超額報酬率 $\bar{r} - r_f$
  - 下行偏差 $D = \text{downside\_deviation}(returns, r_f)$
  - 若 $D \le 1.0e-12$，回傳 `None` (無下行風險或除以零)。
  - 回傳：`Some((\bar{r} - r_f) / D)`。

## 黃金測試向量 (Golden Vectors)
- 序列：`[0.10, -0.05, 0.15, -0.10]`, $r_f = 0.0$:
  - 均值 $\bar{r} = (0.10 - 0.05 + 0.15 - 0.10) / 4 = 0.10 / 4 = 0.025$
  - 負向超額：$[0.0, -0.05, 0.0, -0.10]$
  - 平方和：$0 + 0.0025 + 0 + 0.0100 = 0.0125$
  - 半方差 $D^2 = 0.0125 / 4 = 0.003125$
  - 下行偏差 $D = \sqrt{0.003125} \approx 0.055901699$
  - 索提諾比率 $= 0.025 / 0.055901699 \approx 0.4472136$
- 容差使用 `(val - golden).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_sortino_ratio.rs）
```golden
// 出處：規格序列 [0.10, -0.05, 0.15, -0.10], r_f = 0；Python 獨立驗算
let returns = [0.10, -0.05, 0.15, -0.10];
(downside_deviation(&returns, 0.0).unwrap() - 0.055901699).abs() < 1e-4
(sortino_ratio(&returns, 0.0).unwrap() - 0.4472136).abs() < 1e-4
```
