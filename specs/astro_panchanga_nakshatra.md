# 模組名稱: 吠陀五分曆 27 宿與 4 足計算 (astro_panchanga_nakshatra)

## 1. 目標定位
- 目標模組檔: `src/astro_panchanga_nakshatra.rs`
- 目標測試檔: `tests/test_astro_panchanga_nakshatra.rs`
- 功能簡述: 根據月球黃道經度（毫度 $10^{-3}$ deg，範圍 `0..360000`），以純整數算術計算吠陀 27 宿（Nakshatra 1..27）、4 足（Pada 1..4，對應 D9 九分盤 Navamsha）以及宿內進度（Progress bp, 0..9999 bp），零浮點誤差。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: moon_milli_deg [0..360000]
  pub fn astro_panchanga_nakshatra(moon_milli_deg: u32) -> Option<(u8, u8, u16)>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- `moon_milli_deg > 360_000` 時，函式第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。
- 零外部依賴（100% Rust `std`/`core`），純整數定點數計算。

## 4. 算術公式與常數
- 毫度轉角分純整數公式：$\text{arcmin} = (\text{moon\_milli\_deg} \times 3) / 50$
- 每宿跨度：$13^\circ 20' = 800\text{ arcmin}$。
- 每足跨度：$3^\circ 20' = 200\text{ arcmin}$。
- $\text{nakshatra\_id} = \lfloor \text{arcmin} / 800 \rfloor + 1 \in [1..27]$（$360^\circ$ 時環繞回 27）。
- $\text{rem} = \text{arcmin} \pmod{800}$。
- $\text{pada\_id} = \lfloor \text{rem} / 200 \rfloor + 1 \in [1..4]$。
- $\text{progress\_bp} = \lfloor (\text{rem} \times 10000) / 800 \rfloor \in [0..9999]$。

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 0 毫度（0°00'）: 第 1 宿 Ashwini, 第 1 足, 進度 0 bp
astro_panchanga_nakshatra(0) == Some((1, 1, 0))
// 142.500 度 (142500 milli-deg) -> 142500 * 3 / 50 = 8550 arcmin
// 8550 / 800 = 10 -> Nakshatra 11 (Purva Phalguni), rem = 550 arcmin
// Pada = 550 / 200 + 1 = 3 (第 3 足)
// Progress = 550 * 10000 / 800 = 6875 bp (68.75%)
astro_panchanga_nakshatra(142_500) == Some((11, 3, 6875))
// 360.000 度極限（360000 milli-deg -> 21600 arcmin -> 剛好是 27 宿終點）
// 21600 / 800 = 27 (若取整在邊界時保持為 27 宿第 4 足 9999 bp 或 27 宿結算)
// 越界拒絕：大於 360000 毫度
astro_panchanga_nakshatra(360_001).is_none()
```
