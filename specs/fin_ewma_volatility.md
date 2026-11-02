# 規格書：RiskMetrics EWMA 動態波動率步進 (EWMA Volatility Step)

## 模組名稱
`fin_ewma_volatility`

## 業務目標與適用場景
在量化避險基金、高頻造市與 VaR (Value at Risk) 風控系統中，J.P. Morgan RiskMetrics 的指數加權移動平均 (EWMA) 模型能動態追蹤資產報酬率的條件變異數與波動率。
本模組提供零外部依賴、純 `std` 實作的 EWMA 波動率單步更新微積木。

## 09 號合約邊界規範

### 函式 1：EWMA 條件變異數單步更新
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: prev_variance [0.0..1.0e12]
  /// @domain: current_return [-1.0..10.0]
  /// @domain: lambda [0.5..0.999]
  pub fn ewma_variance_step(
      prev_variance: f64,
      current_return: f64,
      lambda: f64,
  ) -> Option<f64>
  ```
- **數學公式 (RiskMetrics Daily Standard $\lambda = 0.94$)**：
  - $\sigma_t^2 = \lambda \cdot \sigma_{t-1}^2 + (1 - \lambda) \cdot r_t^2$
  - 若 `prev_variance < 0.0 || lambda < 0.5 || lambda >= 1.0`，回傳 `None`。

### 函式 2：計算年化波動率 (Annualized Volatility)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: daily_variance [0.0..1.0e12]
  /// @domain: trading_days_per_year [200.0..365.0]
  pub fn ewma_annualized_volatility(
      daily_variance: f64,
      trading_days_per_year: f64,
  ) -> Option<f64>
  ```
- **數學公式**：
  - $\sigma_{\text{ann}} = \sqrt{\text{daily\_variance} \cdot \text{trading\_days\_per\_year}}$
  - 回傳 `Some(sigma_ann)`。

## 黃金測試向量 (Golden Vectors)
- $\sigma_{t-1}^2 = 0.0004$ (對應日波動率 2%), $r_t = 0.03$ ($r_t^2 = 0.0009$), $\lambda = 0.94$:
  - $\sigma_t^2 = 0.94 \times 0.0004 + 0.06 \times 0.0009 = 0.000376 + 0.000054 = 0.000430$
  - 年化波動率 (以 252 交易日計)：
    $\sigma_{\text{ann}} = \sqrt{0.000430 \times 252} = \sqrt{0.10836} \approx 0.3291808$ (約 32.92%)
- 容差使用 `(val - golden).abs() < 1e-4`。
