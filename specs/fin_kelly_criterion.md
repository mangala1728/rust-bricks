# 規格書：凱利公式最佳資金管理配置 (Kelly Criterion Position Sizing)

## 模組名稱
`fin_kelly_criterion`

## 業務目標與適用場景
在量化交易、高頻造市與避險基金資產配置中，凱利公式 (Kelly Criterion) 是最大化長期對數收益率幾何增長的經典理論。
在面對市場肥尾風險時，實務上常採用分數凱利 (Fractional Kelly, 如半凱利 Half-Kelly) 以降低最大回撤 (Drawdown)。
本模組提供零外部依賴、純 `std` 實作的全凱利與分數凱利倉位計算微積木。

## 09 號合約邊界規範

### 函式 1：計算全凱利與分數凱利最優投資比例 (Kelly Optimal Fraction)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: win_prob [0.01..0.99]
  /// @domain: odds_b [0.01..1000.0]
  /// @domain: fraction [0.01..1.0]
  pub fn kelly_optimal_fraction(
      win_prob: f64,
      odds_b: f64,
      fraction: f64,
  ) -> Option<f64>
  ```
- **數學公式**：
  - 邊界檢查：`win_prob < 0.01 || win_prob > 0.99 || odds_b < 0.01 || odds_b > 1000.0 || fraction < 0.01 || fraction > 1.0` 回傳 `None`。
  - 虧損機率 $q = 1.0 - win\_prob$。
  - 全凱利比例：$f^* = win\_prob - \frac{q}{odds\_b} = \frac{win\_prob \cdot (odds\_b + 1.0) - 1.0}{odds\_b}$。
  - 若 $f^* \le 0.0$，表示期望值為負或零，回傳 `Some(0.0)`（不開倉）。
  - 分數凱利：$f = fraction \cdot f^*$。
  - 夾止於 $[0.0, 1.0]$。
  - 回傳 `Some(f)`。

## 黃金測試向量 (Golden Vectors)
- 向量 1：勝率 60% ($win\_prob = 0.60$)，賠率 1:1 ($odds\_b = 1.0$)，全凱利 ($fraction = 1.0$):
  - $f^* = 0.60 - 0.40 / 1.0 = 0.20$ (20% 資金)。
  - 斷言 `(f - 0.20).abs() < 1e-4`。
- 向量 2：勝率 60% ($win\_prob = 0.60$)，賠率 1:1 ($odds\_b = 1.0$)，半凱利 ($fraction = 0.5$):
  - $f = 0.5 \times 0.20 = 0.10$ (10% 資金)。
  - 斷言 `(f - 0.10).abs() < 1e-4`。
- 向量 3：負期望值情境：勝率 40% ($win\_prob = 0.40$)，賠率 1:1 ($odds\_b = 1.0$):
  - $f^* = 0.40 - 0.60 = -0.20 \le 0$。
  - 回傳 `Some(0.0)`。
