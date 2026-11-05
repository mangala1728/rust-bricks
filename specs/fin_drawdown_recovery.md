# 模組名稱: 最大回撤 MDD 與水下期數追蹤演算法 (Maximum Drawdown & Underwater Duration)

## 1. 目標定位
- 目標模組檔: `src/fin_drawdown_recovery.rs`
- 目標測試檔: `tests/test_fin_drawdown_recovery.rs`
- 功能簡述: 量化投資組合與交易策略之風險回撤分析純函式。輸入淨值走勢序列，計算歷史最大回撤百分比 (Maximum Drawdown %)、最長水下持續期數 (Longest Underwater Duration) 以及當前未修復回撤 (Current Drawdown)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Portfolio maximum drawdown (MDD) and underwater duration analysis
// Inputs: equity_curve: &[f64]
// Outputs: Option<(f64, usize, f64)> (max_drawdown_pct, max_underwater_len, current_drawdown_pct)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 3
/// @domain: equity_curve.len() in [1..=2048], values > 0.0
pub fn fin_compute_drawdown_metrics(
    equity_curve: &[f64],
) -> Option<(f64, usize, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `equity_curve.is_empty() || equity_curve.len() > 2048`，立即回傳 `None`。
- 檢查淨值皆為正有限數，若任何值 $\le 0.0$ 或含 `NaN` 立即回傳 `None`。
- 走訪數列（單一顯式迴圈）：
  - 維護歷史最高點 `peak = equity_curve[0]`
  - `max_dd = 0.0`
  - `current_underwater = 0`
  - `max_underwater = 0`
  - 對每個元素 $v$：
    - 若 $v \ge peak$：
      - `peak = v; current_underwater = 0;`
    - 否則：
      - `current_underwater += 1;`
      - `if current_underwater > max_underwater { max_underwater = current_underwater; }`
      - `let dd = (peak - v) / peak * 100.0;`
      - `if dd > max_dd { max_dd = dd; }`
  - 計算結尾當前回撤：
    - `let curr_dd = (peak - equity_curve.last().unwrap()) / peak * 100.0;`
- 回傳 `Some((max_dd, max_underwater, curr_dd))`。

## 4. 確定性黃金值測試要求
- 淨值序列：`[100.0, 120.0, 90.0, 110.0, 130.0, 104.0]`
  - Peak 依序：100, 120, 120, 120, 130, 130
  - 回撤點：
    - 90.0 時回撤 $(120 - 90)/120 = 25.0\%$
    - 104.0 時回撤 $(130 - 104)/130 = 20.0\%$
  - `max_dd = 25.0%`
  - 水下期數：[90, 110] 兩期在 120 之下，`max_underwater = 2`
  - 當前回撤：$(130 - 104)/130 = 20.0\%$
- 負淨值拒絕。

```golden
let s = [100.0, 120.0, 90.0, 110.0, 130.0, 104.0];
let (mdd, uw, curr) = fin_compute_drawdown_metrics(&s).unwrap();
(mdd - 25.0).abs() < 1e-12
uw == 2
(curr - 20.0).abs() < 1e-12
fin_compute_drawdown_metrics(&[]).is_none()
fin_compute_drawdown_metrics(&[100.0, -50.0]).is_none()
```
