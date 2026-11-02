# 規格書：心率變異度 (HRV) 時域指標計算 (HRV Time-Domain Metrics)

## 模組名稱
`sensor_hrv_time_domain`

## 業務目標與適用場景
在穿戴式智慧手錶、心電貼片與運動壓力監測系統中，心率變異度 (Heart Rate Variability, HRV) 是評估自律神經系統（交感神經與副交感神經平衡狀態）、疲勞恢復度與心臟健康的重要指標。
本模組提供零外部依賴、純 `std` 實作的標準時域指標計算：SDNN, RMSSD 與 pNN50。

## 09 號合約邊界規範

### 函式 1：計算心率變異度時域綜合指標 (HRV Time-Domain Analysis)
- **函式簽名**：
  ```rust
  /// @outputs: 4
  pub fn hrv_time_domain_metrics(rr_intervals_ms: &[f64]) -> Option<(f64, f64, f64, f64)>
  ```
- **輸入輸出定義**：
  - 輸入：心跳間期數組 `rr_intervals_ms` (單位：毫秒，正常範圍 [200.0..3000.0])。
  - 回傳：`Some((mean_rr, sdnn, rmssd, pnn50))`。
- **運算過程**：
  - 若 `rr_intervals_ms.len() < 2` 回傳 `None`。
  - 檢查每個間期是否在 `[200.0, 3000.0]` 範圍內，若有越界回傳 `None`。
  - 均值 $Mean\_RR = \frac{1}{N} \sum RR_i$。
  - 總標準差 $SDNN = \sqrt{\frac{1}{N} \sum_{i=0}^{N-1} (RR_i - Mean\_RR)^2}$。
  - 連續差分 $\Delta_i = RR_{i+1} - RR_i \quad (i = 0 .. N-2)$。
  - 差值均方根 $RMSSD = \sqrt{\frac{1}{N-1} \sum_{i=0}^{N-2} \Delta_i^2}$。
  - $pNN50 = \frac{\text{Count}(|\Delta_i| > 50.0)}{N - 1} \times 100.0$ (百分比)。

## 黃金測試向量 (Golden Vectors)
- 測試數組：`[800.0, 810.0, 890.0, 820.0, 900.0]` ($N = 5$):
  - 均值 $Mean\_RR = 844.0\text{ ms}$。
  - $SDNN \approx 42.2374\text{ ms}$。
  - 差分向量：$[10.0, 80.0, -70.0, 80.0]$ ($N-1 = 4$)。
  - 平方和：$100 + 6400 + 4900 + 6400 = 17800$。
  - 均方：$17800 / 4 = 4450$。
  - $RMSSD = \sqrt{4450} \approx 66.7083\text{ ms}$。
  - 超過 50ms 筆數：$[80, -70, 80]$ 共 3 筆，$pNN50 = 3/4 \times 100 = 75.0\%$。
- 測試斷言：
  - `(mean_rr - 844.0).abs() < 1e-4`
  - `(sdnn - 42.2374).abs() < 1e-4`
  - `(rmssd - 66.7083).abs() < 1e-4`
  - `(pnn50 - 75.0).abs() < 1e-4`
