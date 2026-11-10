# 模組名稱: 分點歲差熱帶與恆星黃道轉換器 (astro_ayanamsha_converter)

## 1. 目標定位
- 目標模組檔: `src/astro_ayanamsha_converter.rs`
- 目標測試檔: `tests/test_astro_ayanamsha_converter.rs`
- 功能簡述: 根據分點歲差值（角秒 Arcseconds，範圍 `0..1296000`），在熱帶黃道經度（Tropical）與恆星黃道經度（Sidereal）之間以純整數毫度進行雙向轉換。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 常用歲差參考值（J2000.0）：
  - Lahiri (Chitra Paksha): $85,871''$ ($23^\circ 51' 11'' = 23,853\text{ milli-deg}$)
  - Krishnamurti (KP): $85,410''$ ($23^\circ 43' 30'' = 23,725\text{ milli-deg}$)
  - Raman: $77,934''$ ($21^\circ 38' 54'' = 21,648\text{ milli-deg}$)
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: tropical_milli_deg [0..360000]
  /// @domain: ayanamsha_arcsec [0..1296000]
  pub fn astro_tropical_to_sidereal(tropical_milli_deg: u32, ayanamsha_arcsec: u32) -> Option<u32>;

  /// @outputs: 1
  /// @domain: sidereal_milli_deg [0..360000]
  /// @domain: ayanamsha_arcsec [0..1296000]
  pub fn astro_sidereal_to_tropical(sidereal_milli_deg: u32, ayanamsha_arcsec: u32) -> Option<u32>;
  ```

## 3. 邊界與異常條件 (Domain Guards)
- 角度大於 `360_000` 或歲差大於 `1_296_000`（全週角秒 $360 \times 3600$）時，函式第一行守衛必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。

## 4. 算術公式
- 角秒轉毫度純整數公式（四捨五入）：
  $$\text{ayan\_milli} = \frac{\text{ayanamsha\_arcsec} \times 1000 + 1800}{3600}$$
- 熱帶轉恆星：$\lambda_{\text{sidereal}} = (\lambda_{\text{tropical}} + 360000 - \text{ayan\_milli}) \pmod{360000}$
- 恆星轉熱帶：$\lambda_{\text{tropical}} = (\lambda_{\text{sidereal}} + \text{ayan\_milli}) \pmod{360000}$

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 0 歲差：熱帶等於恆星
astro_tropical_to_sidereal(150_000, 0) == Some(150_000)
// Lahiri 歲差 85871" -> (85871 * 1000 + 1800) / 3600 = 85872800 / 3600 = 23853 milli-deg (23.853°)
// 熱帶 30.000 度 (30000 milli-deg) -> 恆星 = 30000 + 360000 - 23853 = 366147 % 360000 = 6147 milli-deg (6.147°)
astro_tropical_to_sidereal(30_000, 85_871) == Some(6147)
// 逆向還原：恆星 6147 milli-deg -> 熱帶 = 6147 + 23853 = 30000 milli-deg
astro_sidereal_to_tropical(6147, 85_871) == Some(30_000)
// 越界拒絕：大於 360000 毫度
astro_tropical_to_sidereal(360_001, 85_871).is_none()
// 越界拒絕：大於 1296000 角秒
astro_tropical_to_sidereal(30_000, 1_296_001).is_none()
```
