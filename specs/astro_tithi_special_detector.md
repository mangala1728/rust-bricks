# 模組名稱: 吠陀特殊 Tithi（缺日與增日）檢測器 (astro_tithi_special_detector)

## 1. 目標定位
- 目標模組檔: `src/astro_tithi_special_detector.rs`
- 目標測試檔: `tests/test_astro_tithi_special_detector.rs`
- 功能簡述: 根據特定 Tithi 的起始與結束時間戳記（Unix 秒），比對當日日出與次日日出時間戳記，判定該 Tithi 是否為缺日（Kshaya Tithi，未涵蓋任何日出而在曆法中跳過）或增日（Vriddhi Tithi，同一個 Tithi 橫跨連續兩次日出）。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: tithi_start_ts [0..4102444800]
  /// @domain: tithi_end_ts [0..4102444800]
  /// @domain: sunrise_curr_ts [0..4102444800]
  /// @domain: sunrise_next_ts [0..4102444800]
  pub fn astro_tithi_special_detector(
      tithi_start_ts: u64,
      tithi_end_ts: u64,
      sunrise_curr_ts: u64,
      sunrise_next_ts: u64,
  ) -> Option<u8>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- 任何時間戳記大於 `4_102_444_800`（西元 2100 年），或 `tithi_start_ts >= tithi_end_ts`，或 `sunrise_curr_ts >= sunrise_next_ts` 時，立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。

## 4. 判準與回傳代碼
- 回傳值 `u8`：
  - `0`: 常態 Tithi（Normal，正常交接）
  - `1`: 缺日 / 虧日（Kshaya Tithi，在今日日出之後開始，且在明日日出之前結束：`tithi_start_ts > sunrise_curr_ts && tithi_end_ts < sunrise_next_ts`）
  - `2`: 增日（Vriddhi Tithi，在今日日出之前開始，且在明日日出之後結束，涵蓋了連續兩次日出：`tithi_start_ts <= sunrise_curr_ts && tithi_end_ts >= sunrise_next_ts`）

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 缺日 Kshaya: 日出為 1000 與 2000，Tithi 介於 1200 至 1800 (未涵蓋任何日出)
astro_tithi_special_detector(1200, 1800, 1000, 2000) == Some(1)
// 增日 Vriddhi: 日出為 1000 與 2000，Tithi 介於 900 至 2100 (涵蓋兩次日出)
astro_tithi_special_detector(900, 2100, 1000, 2000) == Some(2)
// 常態 Normal: Tithi 介於 800 至 1500 (只涵蓋第 1 個日出 1000)
astro_tithi_special_detector(800, 1500, 1000, 2000) == Some(0)
// 異常拒絕：起始時間晚於結束時間
astro_tithi_special_detector(2000, 1000, 500, 1500).is_none()
// 異常拒絕：日出時間順序錯誤
astro_tithi_special_detector(1000, 2000, 1500, 1000).is_none()
```
