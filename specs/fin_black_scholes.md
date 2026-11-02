# 規格書：Black-Scholes 歐式期權定價與 Delta (Black-Scholes Option Pricing)

## 模組名稱
`fin_black_scholes`

## 業務目標與適用場景
在量化交易、金融衍生品風控與投資組合動態對沖中，Black-Scholes-Merton 封閉解是歐式期權定價與避險比率 (Delta) 計算之行業基石。
本模組提供零外部依賴、純 `std` 實作的解析定價與 Greeks 計算。

## 09 號合約邊界規範

### 函式 1：累計標準常態分佈函式 CDF (Abramowitz & Stegun 7.1.26 數值解)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: x [-10.0..10.0]
  pub fn standard_normal_cdf(x: f64) -> Option<f64>
  ```
- **數值公式 (誤差 < 1.5e-7)**：
  - 令 $z = |x|$
  - $t = \frac{1}{1 + 0.2316419 z}$
  - $p(z) = \frac{1}{\sqrt{2\pi}} \exp\left(-\frac{z^2}{2}\right) \left(b_1 t + b_2 t^2 + b_3 t^3 + b_4 t^4 + b_5 t^5\right)$
    其中 $b_1 = 0.319381530, b_2 = -0.356563782, b_3 = 1.781477937, b_4 = -1.821255978, b_5 = 1.330274429$
  - 若 $x \ge 0$，回傳 $1 - p(z)$；若 $x < 0$，回傳 $p(z)$。

### 函式 2：Black-Scholes 歐式期權理論價格與 Delta
- **函式簽名**：
  ```rust
  /// @outputs: 4
  /// @domain: s0 [0.001..1.0e9]
  /// @domain: strike [0.001..1.0e9]
  /// @domain: rate [-0.5..1.0]
  /// @domain: sigma [0.001..10.0]
  /// @domain: t_years [0.001..100.0]
  pub fn black_scholes_european(
      s0: f64,
      strike: f64,
      rate: f64,
      sigma: f64,
      t_years: f64,
  ) -> Option<(f64, f64, f64, f64)>
  ```
- **回傳結構**：
  `(call_price, put_price, call_delta, put_delta)`
- **公式**：
  - $d_1 = \frac{\ln(S_0 / K) + (r + \sigma^2 / 2) T}{\sigma \sqrt{T}}$
  - $d_2 = d_1 - \sigma \sqrt{T}$
  - $\text{Call} = S_0 N(d_1) - K e^{-r T} N(d_2)$
  - $\text{Put} = K e^{-r T} N(-d_2) - S_0 N(-d_1)$
  - $\text{Call Delta} = N(d_1)$
  - $\text{Put Delta} = N(d_1) - 1$

## 黃金測試向量 (Golden Vectors - John Hull 經典教材)
- $S_0 = 100.0, K = 100.0, r = 0.05, \sigma = 0.20, T = 1.0$:
  - $d_1 = (0 + 0.07) / 0.2 = 0.35$
  - $d_2 = 0.35 - 0.2 = 0.15$
  - $N(d_1) \approx 0.63683, N(d_2) \approx 0.55962$
  - $\text{Call Price} \approx 10.4506$
  - $\text{Put Price} \approx 5.5735$
  - $\text{Call Delta} \approx 0.6368$
  - $\text{Put Delta} \approx -0.3632$
- 容差使用 `(val - golden).abs() < 1e-3`。
