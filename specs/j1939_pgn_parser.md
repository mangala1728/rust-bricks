# 規格書：SAE J1939 參數群編號 (PGN) 與 29-bit CAN 標識符解析器 (SAE J1939 PGN Parser)

## 模組名稱
`j1939_pgn_parser`

## 業務目標與適用場景
在重型卡車、柴油發電機、工程機械與農業機械（如 John Deere、Caterpillar、Volvo 總線）中，SAE J1939 規範以 29-bit 擴展 CAN 標識符傳遞車載診斷與實時工況（如引擎轉速、油耗、冷卻水溫）。
本模組提供零外部依賴、純 `std` 實作的 29-bit CAN ID 解碼、PDU1/PDU2 格式判定與 18-bit PGN 提取微積木。

## 09 號合約邊界規範

### 函式 1：解析 29-bit CAN 標識符 (Parse J1939 CAN ID)
- **函式簽名**：
  ```rust
  /// @outputs: 4
  /// @domain: can_id_29bit [0..0x1FFFFFFF]
  pub fn j1939_parse_can_id(can_id_29bit: u32) -> Option<(u8, u32, u8, u8)>
  ```
- **位元解析結構**：
  - 邊界檢查：`can_id_29bit > 0x1FFFFFFF` 回傳 `None`。
  - Priority (3 bits): `(can_id_29bit >> 26) as u8 & 0x07`
  - Data Page (DP, 1 bit): `(can_id_29bit >> 24) & 0x01`
  - PDU Format (PF, 8 bits): `((can_id_29bit >> 16) & 0xFF) as u8`
  - PDU Specific (PS, 8 bits): `((can_id_29bit >> 8) & 0xFF) as u8`
  - Source Address (SA, 8 bits): `(can_id_29bit & 0xFF) as u8`
  - PGN (Parameter Group Number) 計算：
    - 若 $PF < 240$ (PDU1 格式 - 點對點傳輸，PS 為目標地址 DA)：
      $PGN = (DP \ll 16) | ((PF \text{ as u32}) \ll 8)$
    - 若 $PF \ge 240$ (PDU2 格式 - 廣播傳輸，PS 為群組擴展 GE)：
      $PGN = (DP \ll 16) | ((PF \text{ as u32}) \ll 8) | (PS \text{ as u32})$
  - 回傳 `Some((priority, pgn, ps, sa))`。

### 函式 2：判斷是否為 PDU1 點對點格式
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn j1939_is_pdu1(pf: u8) -> bool
  ```
- **定義**：若 `pf < 240` 回傳 `true`，否則回傳 `false`。

## 黃金測試向量 (Golden Vectors - SAE J1939 Standard)
- 向量 1 (PDU2 廣播): CAN ID = `0x18FEE600` (時間與日期 Time/Date)
  - Priority = `6`
  - PF = `0xFE` (254 >= 240 -> PDU2, 廣播)
  - PS = `0xE6` (Group Extension)
  - SA = `0x00` (Engine)
  - PGN = `0xFEE6` (65254)
  - 解析應回傳 `Some((6, 65254, 0xE6, 0x00))`。
- 向量 2 (PDU1 點對點): CAN ID = `0x18EA0001` (請求 PGN 訊框 Request)
  - Priority = `6`
  - PF = `0xEA` (234 < 240 -> PDU1, 點對點)
  - PS (Destination Address) = `0x00`
  - SA = `0x01`
  - PGN = `0xEA00` (59904)
  - 解析應回傳 `Some((6, 59904, 0x00, 0x01))`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_j1939_pgn_parser.rs）
```golden
// 出處：規格向量 1（SAE J1939 Time/Date PGN 65254，PDU2）；Python 位元運算獨立驗算
j1939_parse_can_id(0x18FEE600) == Some((6, 65254, 0xE6, 0x00))
// 出處：規格向量 2（Request PGN 59904，PDU1，PS 不併入 PGN）
j1939_parse_can_id(0x18EA0001) == Some((6, 59904, 0x00, 0x01))
// 出處：規格邊界檢查（> 0x1FFFFFFF 回傳 None）
j1939_parse_can_id(0x2000_0000).is_none()
j1939_parse_can_id(0xFFFF_FFFF).is_none()
// 出處：規格函式 2 定義 pf < 240
j1939_is_pdu1(0xEA)
!j1939_is_pdu1(0xFE)
j1939_is_pdu1(239)
!j1939_is_pdu1(240)
```
