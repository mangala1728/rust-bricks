# 規格書：最大回撤率與卡瑪比率 (Maximum Drawdown & Calmar Ratio)

## 模組名稱
`fin_calmar_ratio`

## 業務目標與適用場景
在避險基金（Hedge Funds）、CTA 量化趨勢策略與高頻交易回測中，最大回撤率（Maximum Drawdown, MDD）衡量歷史走勢中自峰值至谷底的最極端本金回檔幅度；卡瑪比率（Calmar Ratio）以年化收益率除以最大回撤率，是評估抗風險能力與獲利彈性的關鍵指標。
本模組提供零外部依賴、純 `std` 實作的 $O(N)$ 單次遍歷最大回撤率提取與卡瑪比率計算微積木。

## 09 號合約邊界規範

### 函式 1：淨值序列最大回撤率 (Maximum Drawdown)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: prices.len() >= 2, prices[i] > 0.0
  pub fn fin_max_drawdown(prices: &[f64]) -> Option<f64>
  ```
- **演算法規範**：
  - 邊界檢查：`if prices.len() < 2 || prices.iter().any(|&p| !p.is_finite() || p <= 0.0) { return None; }`
  - 單次遍歷動態追蹤歷史高點（Running Peak）與最大相對回撤：
    ```rust
    let mut peak = prices[0];
    let mut max_dd = 0.0;
    for &price in prices.iter().skip(1) {
        if price > peak {
            peak = price;
        } else {
            let dd = (peak - price) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
    }
    Some(max_dd)
    ```

### 函式 2：卡瑪比率計算 (Calmar Ratio)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: max_drawdown in (0.0..=1.0], annualized_return is finite
  pub fn fin_calmar_ratio(annualized_return: f64, max_drawdown: f64) -> Option<f64>
  ```
- **演算法規範**：
  - 邊界檢查：`if !annualized_return.is_finite() || !max_drawdown.is_finite() || max_drawdown <= 1e-9 || max_drawdown > 1.0 { return None; }`
  - 計算公式：
    $$\text{Calmar} = \frac{\text{annualized\_return}}{\text{max\_drawdown}}$$
  - 回傳 `Some(calmar)`。

## 黃金測試向量 (Golden Vectors)
- 向量 1 (淨值回撤):
  - 價格序列 `prices = [100.0, 120.0, 90.0, 110.0, 80.0, 130.0]`
  - 峰值 120.0 跌至 90.0：回撤 $(120 - 90)/120 = 25\%$
  - 峰值 120.0 跌至 80.0：回撤 $(120 - 80)/120 = 33.3333\%$
  - 最大回撤率應為 $1/3 \approx 0.333333$
  - 斷言驗證：`(fin_max_drawdown(&prices).unwrap() - 1.0 / 3.0).abs() < 1e-4`
- 向量 2 (卡瑪比率):
  - `annualized_return = 0.20` (20%), `max_drawdown = 0.10` (10%) -> `Calmar = 2.0`
  - 斷言驗證：`(fin_calmar_ratio(0.20, 0.10).unwrap() - 2.0).abs() < 1e-4`

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_calmar_ratio.rs）
```golden
// 出處：規格向量 1（最大回撤 1/3）與向量 2（Calmar = 2.0）；Python 獨立驗算
let prices = [100.0, 120.0, 90.0, 110.0, 80.0, 130.0];
(fin_max_drawdown(&prices).unwrap() - 1.0 / 3.0).abs() < 1e-4
(fin_calmar_ratio(0.20, 0.10).unwrap() - 2.0).abs() < 1e-4
```
