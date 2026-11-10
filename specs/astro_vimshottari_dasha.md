# 模組名稱: 吠陀 Vimshottari Dasha 運限 32-bit 打包與解包 (astro_vimshottari_dasha)

## 1. 目標定位
- 目標模組檔: `src/astro_vimshottari_dasha.rs`
- 目標測試檔: `tests/test_astro_vimshottari_dasha.rs`
- 功能簡述: 將吠陀 Vimshottari Dasha 運限（大運 Mahadasha, 小運 Antardasha, 次小運 Pratyantardasha, 微運 Sookshmadasha, 呼吸運 Pranadasha 及其餘量）緊湊打包為 32 位元無符號整數（`dasha_packed: u32`），供 Bus 1 32-Byte 事件封包傳輸與還原。

## 2. 09 合約介面定義
- 檔案開頭必備 `//! ## Map`（含 Role, Inputs, Outputs, Invariants）。
- 行星代碼映射（1..9）：
  `1=Ketu, 2=Venus, 3=Sun, 4=Moon, 5=Mars, 6=Rahu, 7=Jupiter, 8=Saturn, 9=Mercury`
- 具名函式簽章：
  ```rust
  /// @outputs: 1
  /// @domain: maha [1..9]
  /// @domain: antar [1..9]
  /// @domain: prat [1..9]
  /// @domain: sookshma [1..9]
  /// @domain: prana [1..9]
  /// @domain: balance_permille [0..4095]
  pub fn astro_vimshottari_dasha_pack(
      maha: u8,
      antar: u8,
      prat: u8,
      sookshma: u8,
      prana: u8,
      balance_permille: u16,
  ) -> Option<u32>;

  /// @outputs: 1
  pub fn astro_vimshottari_dasha_unpack(packed: u32) -> (u8, u8, u8, u8, u8, u16);
  ```

## 3. 邊界與異常條件 (Domain Guards)
- 任何行星代碼不在 `1..=9` 範圍內，或 `balance_permille > 4095` 時，`astro_vimshottari_dasha_pack` 必須立即返回 `None`。
- 單一函式長度嚴格 $\le 60$ 行（NASA JPL 微積木原則），控制流巢狀深度 $\le 2$，禁止遞迴與 `static mut`。

## 4. 位元排佈 (Bit Allocation)
- Bit 00 .. 03 (4 bits): `maha` (1..9)
- Bit 04 .. 07 (4 bits): `antar` (1..9)
- Bit 08 .. 11 (4 bits): `prat` (1..9)
- Bit 12 .. 15 (4 bits): `sookshma` (1..9)
- Bit 16 .. 19 (4 bits): `prana` (1..9)
- Bit 20 .. 31 (12 bits): `balance_permille` (0..4095)

## 5. 機器可驗證的黃金向量 (```golden)

```golden
// 打包 Sun (3), Jupiter (7), Ketu (1), Moon (4), Mars (5), Balance 2500
// (3 << 0) | (7 << 4) | (1 << 8) | (4 << 12) | (5 << 16) | (2500 << 20)
// = 3 | 112 | 256 | 16384 | 327680 | 2621440000 = 2621784435
astro_vimshottari_dasha_pack(3, 7, 1, 4, 5, 2500) == Some(2621784435)
// 解包回傳等價
astro_vimshottari_dasha_unpack(2621784435) == (3, 7, 1, 4, 5, 2500)
// 非法行星代碼拒絕：0
astro_vimshottari_dasha_pack(0, 1, 1, 1, 1, 0).is_none()
// 非法行星代碼拒絕：10
astro_vimshottari_dasha_pack(10, 1, 1, 1, 1, 0).is_none()
// 餘量超標拒絕：4096 (超過 12 bits)
astro_vimshottari_dasha_pack(1, 1, 1, 1, 1, 4096).is_none()
```
