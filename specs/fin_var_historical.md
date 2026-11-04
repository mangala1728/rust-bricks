# 規格書：歷史模擬風險價值 (VaR) 與條件風險價值 (CVaR) 計算 (Historical VaR and CVaR)

## 模組名稱
`fin_var_historical`

## 業務目標與適用場景
在巴塞爾協定資本適足率規範（Basel III/IV）與量化風控管理中，歷史模擬法（Historical Simulation）是計算投資組合在特定信心水準下最大預期損失（Value at Risk, VaR）與超額損失期望值（Conditional VaR / Expected Shortfall）的基準方法。
本模組提供零外部依賴、純 `std` 實作的歷史損失分位數提取與條件期望尾端風險微積木。

## 09 號合約邊界規範

### 函式 1：歷史模擬法 VaR (Historical Value at Risk)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: returns.len() >= 2, confidence in (0.0..1.0)
  pub fn fin_var_historical(returns: &[f64], confidence: f64) -> Option<f64>
  ```
- **演算法規範**：
  - 邊界檢查：`if returns.len() < 2 || confidence <= 0.0 || confidence >= 1.0 || !confidence.is_finite() { return None; }`
  - 檢查所有元素是否有效：`if returns.iter().any(|&r| !r.is_finite()) { return None; }`
  - 複製並由小至大排序：
    ```rust
    let mut sorted = returns.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    ```
  - 計算分位數索引：
    - $\alpha = 1.0 - \text{confidence}$
    - 索引位置（必須加入 1e-9 微小量防範 IEEE 754 浮點截斷）：`let idx = (((alpha * (sorted.len() as f64)) + 1e-9).floor() as usize).min(sorted.len() - 1);`
  - 損失表示法（Loss Convention）：通常 VaR 表達為非負的損失數值：
    - $\text{var} = -\text{sorted}[idx]$
  - 回傳 `Some(var)`。

### 函式 2：歷史條件風險價值 (Historical CVaR / Expected Shortfall)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: returns.len() >= 2, confidence in (0.0..1.0)
  pub fn fin_cvar_historical(returns: &[f64], confidence: f64) -> Option<f64>
  ```
- **演算法規範**：
  - 邊界檢查：同函式 1。
  - 同樣排序 returns。
  - 尾端樣本數：`let count = ((((1.0 - confidence) * (sorted.len() as f64)) + 1e-9).ceil() as usize).max(1).min(sorted.len());`
  - 計算前 `count` 個最差報酬之平均損失：
    - `let sum_loss: f64 = sorted[..count].iter().map(|&r| -r).sum();`
    - `let cvar = sum_loss / (count as f64);`
  - 回傳 `Some(cvar)`。

## 黃金測試向量 (Golden Vectors)
- 斷言規範：浮點數測試必須使用 `(val - expected).abs() < 1e-4`。
- 樣本數據 (10 個日報酬率):
  - `returns = [-0.05, -0.04, -0.03, -0.02, -0.01, 0.01, 0.02, 0.03, 0.04, 0.05]`
  - `confidence = 0.90` (alpha = 0.10)
  - VaR (90%):
    - `idx = (0.10 * 10).floor() = 1` -> `sorted[1] = -0.04` -> VaR = `0.04` (4% 損失)
  - CVaR (90%):
    - `count = (0.10 * 10).ceil() = 1` -> 最差 1 個損失為 `0.05` -> CVaR = `0.05`
  - `confidence = 0.80` (alpha = 0.20)
    - `count = (0.20 * 10).ceil() = 2` -> 最差 2 個損失平均 `(0.05 + 0.04) / 2 = 0.045` -> CVaR = `0.045`

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_fin_var_historical.rs）
```golden
// 出處：規格樣本 10 筆報酬，VaR(90%) idx = floor(0.10*10) = 1 -> -sorted[1] = 0.04（Python 獨立驗算）
let r = [-0.05, -0.04, -0.03, -0.02, -0.01, 0.01, 0.02, 0.03, 0.04, 0.05];
(fin_var_historical(&r, 0.90).unwrap() - 0.04).abs() < 1e-4
// 出處：規格邊界檢查（len < 2、confidence 不在 (0,1)、非有限值一律 None）
fin_var_historical(&[-0.01], 0.90).is_none()
fin_var_historical(&r, 0.0).is_none()
fin_var_historical(&r, 1.0).is_none()
fin_var_historical(&r, f64::NAN).is_none()
fin_var_historical(&[-0.01, f64::NAN], 0.90).is_none()
fin_cvar_historical(&[-0.01], 0.90).is_none()
fin_cvar_historical(&r, 0.0).is_none()
fin_cvar_historical(&r, 1.0).is_none()
fin_cvar_historical(&[-0.01, f64::INFINITY], 0.90).is_none()
// 註：規格的 CVaR 向量（0.90 -> 0.05、0.80 -> 0.045）與規格自身的 count 公式（ceil(alpha*n + 1e-9)）互相矛盾，暫不轉寫，待人類複核
```
