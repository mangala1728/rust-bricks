# 模組名稱: 吠陀五分曆 27 曜與 60 關計算 (astro_panchanga_yoga_karana)

## 1. 目標定位
- 目標模組檔: `src/astro_panchanga_yoga_karana.rs`
- 目標測試檔: `tests/test_astro_panchanga_yoga_karana.rs`
- 功能簡述: 根據太陽與月球黃道經度（毫度 $10^{-3}$ deg，範圍 `0..360000`），以純整數算術計算吠陀 27 曜（Yoga 1..27）與 60 關（Karana 1..60），零浮點誤差。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: moon_milli_deg [0..360000]
  /// @domain: sun_milli_deg [0..360000]
  pub fn astro_panchanga_yoga_karana(moon_milli_deg: u32, sun_milli_deg: u32) -> Option<(u8, u8)>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- `moon_milli_deg > 360_000 || sun_milli_deg > 360_000` 時，函式第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。
- 零外部依賴（100% Rust `std`/`core`），純整數定點數計算。

## 4. 算術公式與常數
- 毫度轉角分：$\text{sun\_min} = (\text{sun\_milli\_deg} \times 3) / 50$，$\text{moon\_min} = (\text{moon\_milli\_deg} \times 3) / 50$
- **Yoga（日月經度和）**：
  - $\text{sum\_min} = (\text{sun\_min} + \text{moon\_min}) \pmod{21600}$
  - $\text{yoga\_id} = \lfloor \text{sum\_min} / 800 \rfloor + 1 \in [1..27]$
- **Karana（半個 Tithi，每 $6^\circ = 6000\text{ milli-deg}$ 一關）**：
  - $\Delta = (\text{moon\_milli\_deg} + 360000 - \text{sun\_milli\_deg}) \pmod{360000}$
  - $\text{karana\_id} = \lfloor \Delta / 6000 \rfloor + 1 \in [1..60]$

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 日月同在 0 毫度：Yoga 1 (Vishkambha), Karana 1 (Bava, 第 1 Tithi 前半)
astro_panchanga_yoga_karana(0, 0) == Some((1, 1))
// 月亮 142.5 度 (142500 milli-deg -> 8550 arcmin), 太陽 30.0 度 (30000 milli-deg -> 1800 arcmin)
// Sum = 10350 arcmin -> 10350 / 800 = 12 -> Yoga 13 (Dhriti)
// Delta = 112500 milli-deg -> 112500 / 6000 = 18 -> Karana 19 (Bhadra / Vishti)
astro_panchanga_yoga_karana(142_500, 30_000) == Some((13, 19))
// 越界拒絕：大於 360000 毫度
astro_panchanga_yoga_karana(360_001, 10_000).is_none()
astro_panchanga_yoga_karana(10_000, 360_001).is_none()
```
