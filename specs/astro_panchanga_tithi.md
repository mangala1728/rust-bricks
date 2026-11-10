# 模組名稱: 吠陀五分曆純整數 Tithi 與進度計算 (astro_panchanga_tithi)

## 1. 目標定位
- 目標模組檔: `src/astro_panchanga_tithi.rs`
- 目標測試檔: `tests/test_astro_panchanga_tithi.rs`
- 功能簡述: 根據月球與太陽黃道經度（毫度 $10^{-3}$ deg，範圍 `0..360000`），以純整數算術計算吠陀 30 Tithi（1..15 白月 Shukla, 16..30 黑月 Krishna）及當前 Tithi 的進度基點（Progress Basis Points, 0..9999 bp），零浮點誤差。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: moon_milli_deg [0..360000]
  /// @domain: sun_milli_deg [0..360000]
  pub fn astro_panchanga_tithi(moon_milli_deg: u32, sun_milli_deg: u32) -> Option<(u8, u16)>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- `moon_milli_deg > 360_000 || sun_milli_deg > 360_000` 時，函式第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。
- 零外部依賴（100% Rust `std`/`core`），純整數定點數計算。

## 4. 算術公式與常數
- 相對黃經角差（毫度）：$\Delta = (\text{moon} + 360000 - \text{sun}) \pmod{360000}$
- 每個 Tithi 跨度：$12^\circ = 12,000\text{ milli-deg}$。
- $\text{tithi\_id} = \lfloor \Delta / 12000 \rfloor + 1 \in [1..30]$。
- $\text{rem} = \Delta \pmod{12000}$。
- $\text{progress\_bp} = \lfloor (\text{rem} \times 10000) / 12000 \rfloor \in [0..9999]$。

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 基準恆等：日月同度（0度合相，新月/朔 Amavasya 終點，第 1 Tithi 初始）
astro_panchanga_tithi(0, 0) == Some((1, 0))
// 朔月 180 度沖相（滿月/望 Purnima，Delta = 180000 -> 180000/12000 = 15 -> Tithi 15, rem 0 -> progress 0）
astro_panchanga_tithi(180_000, 0) == Some((16, 0))
// 月亮 142.5 度 (142500)，太陽 30.0 度 (30000) -> Delta = 112500
// 112500 / 12000 = 9 (Tithi 10 Dashami), rem = 4500, progress = 4500 * 10000 / 12000 = 3750 bp (37.5%)
astro_panchanga_tithi(142_500, 30_000) == Some((10, 3750))
// 越界拒絕：大於 360000 毫度
astro_panchanga_tithi(360_001, 10_000).is_none()
astro_panchanga_tithi(10_000, 360_001).is_none()
```
