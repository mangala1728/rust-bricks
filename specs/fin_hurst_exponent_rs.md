# 模組名稱: 重標極差 R/S 分析與 Hurst 指數計算器 (Rescaled Range R/S Analysis & Hurst Exponent)

## 1. 目標定位
- 目標模組檔: `src/fin_hurst_exponent_rs.rs`
- 目標測試檔: `tests/test_fin_hurst_exponent_rs.rs`
- 功能簡述: 量化金融時間序列之長記憶性與碎形特徵分析。計算重標極差 (Rescaled Range, R/S) 比率與 Hurst 估算值 ($H \approx \ln(R/S) / \ln(N)$)，用於區分均值回歸 ($H < 0.5$)、隨機漫步 ($H = 0.5$) 與趨勢持久性 ($H > 0.5$)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Rescaled Range (R/S) analysis and Hurst exponent estimation
// Inputs: series: &[f64]
// Outputs: Option<(f64, f64)> (rs_ratio, hurst_exponent)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: series.len() in [8..=1024], finite values, std_dev > 0
pub fn fin_compute_hurst_rs(series: &[f64]) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `series.len() < 8 || series.len() > 1024`，立即回傳 `None`。
- 檢查所有元素是否為有限數值，若含 `NaN` 立即回傳 `None`。
- 計算：
  - `let n = series.len() as f64;`
  - `let mean = sum(series) / n;`
  - 標準差：`variance = sum((x - mean)^2) / n`
  - 若 `variance <= 1e-14`，立即回傳 `None` (常數序列無法計算)。
  - `s = variance.sqrt()`
  - 累積離差：
    - 走訪數列，維護累積和 $z_t = \sum_{k=1}^t (x_k - mean)$。
    - 追蹤 $\max(z_t)$ 與 $\min(z_t)$（初始為 $0.0$）。
  - 極差：`r = max_z - min_z`
  - `rs_ratio = r / s`
  - 若 `rs_ratio <= 1.0` 則 `hurst = 0.5`，否則 `hurst = (rs_ratio.ln() / n.ln()).clamp(0.0, 1.0)`。
- 回傳 `Some((rs_ratio, hurst))`。

## 4. 確定性黃金值測試要求
- 確定性序列計算驗證：長度 8 之嚴格序列。
- 常數序列（標準差為 0）或長度小於 8 拒絕。

```golden
let s = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
let (rs, h) = fin_compute_hurst_rs(&s).unwrap();
rs > 1.0
h > 0.0 && h <= 1.0
fin_compute_hurst_rs(&[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]).is_none()
fin_compute_hurst_rs(&[1.0, 2.0, 3.0]).is_none()
```
