# 模組名稱: CAN-FD 17位元循環冗餘校驗碼 (CRC-17/CAN-FD)

> **2026-11-05 校正（MSA 試點，見 `msa/canfd_crc.rs`）**：前一版的演算法把資料位元推進暫存器最低位、沒有乘 x^17，算出的是 M(x) mod G(x) 而不是 CRC（資料 ≤ 16 位元時原樣回傳）；其黃金向量是照該錯誤演算法推出的，量具驗收判定「量具壞了」。本版改依 RevEng CRC 目錄參數，黃金向量全部標註外部出處。

## 1. 目標定位
- 目標模組檔: `src/canfd_crc.rs`
- 目標測試檔: `tests/test_canfd_crc.rs`
- 功能簡述: 計算 **CRC-17/CAN-FD**（RevEng CRC Catalogue 參數：width=17、poly=0x1685B、init=0x00000、refin=false、refout=false、xorout=0x00000、check=0x04F03、residue=0x00000），多項式 $x^{17}+x^{16}+x^{14}+x^{13}+x^{11}+x^6+x^4+x^3+x+1$。
- 範圍限制（必須照實命名，不得誇大）：只對**位元組資料**計算（MSB 先）。ISO 11898-1:2015 實際訊框的 CRC 另含 SOF／仲裁／控制欄位、stuff count，且暫存器初值不同，**不在本積木範圍**；CRC-17 只適用負載 ≤ 16 位元組（更長用 CRC-21）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: CRC-17/CAN-FD（RevEng 參數）位元組資料校驗
// Inputs: crc: u32, bit: u8, data: &[u8]
// Outputs: Option<u32>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines
/// @outputs: 2
/// @domain: crc [0..0x1FFFF]
/// @domain: bit [0..1]
pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32>;
/// @outputs: 2
/// @domain: data.len() [0..16]
pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32>;
```

## 3. 演算法與邊界 (Domain Guards)
- `canfd_crc17_bit`（ISO 11898-1 移位暫存器定義：CRC_NXT = NXTBIT xor CRC_RG(16)）：
  - 若 `crc > 0x1FFFF || bit > 1`，回傳 `None`。
  - `let feedback = ((crc >> 16) & 1) ^ (bit as u32);`
  - `let shifted = (crc << 1) & 0x1FFFF;`
  - 若 `feedback != 0` 則回傳 `Some(shifted ^ 0x1685B)`，否則 `Some(shifted)`。
- `canfd_crc17_slice`：若 `data.len() > 16`，回傳 `None`。初值 0，每個位元組自 MSB (bit 7) 至 LSB (bit 0) 逐位元呼叫 `canfd_crc17_bit`。

## 4. 確定性黃金值測試要求
預期值全部來自外部出處，**不得**照本規格的演算法自行推算：
- RevEng CRC Catalogue 的 check 值：`"123456789"` → `0x04F03`。
- 多項式長除法 M(x)·x^17 mod G(x) 的獨立計算（與 RevEng 參數一致）：`[0x80]` → `0x0587B`、`[0x12, 0x34]` → `0x1ACDB`、`[0xFF]` → `0x19052`。
- ISO 11898-1 移位暫存器定義的單步結果，以及長度、值域邊界。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_canfd_crc.rs）
```golden
// @source: RevEng CRC Catalogue「CRC-17/CAN-FD」check value
canfd_crc17_slice(b"123456789") == Some(0x04F03)
// @source: 多項式長除法 M(x)·x^17 mod G(x) 獨立計算（Python），與 RevEng 參數一致
canfd_crc17_slice(&[0x80]) == Some(0x0587B)
canfd_crc17_slice(&[0x12, 0x34]) == Some(0x1ACDB)
canfd_crc17_slice(&[0xFF]) == Some(0x19052)
canfd_crc17_slice(&[]) == Some(0)
// @source: ISO 11898-1 移位暫存器定義 CRC_NXT = NXTBIT xor CRC_RG(16)
canfd_crc17_bit(0, 1) == Some(0x1685B)
canfd_crc17_bit(0x10000, 0) == Some(0x1685B)
canfd_crc17_bit(0x10000, 1) == Some(0)
canfd_crc17_bit(1, 0) == Some(2)
// @source: 本規格範圍（CRC-17 只適用 ≤ 16 位元組）與值域
canfd_crc17_slice(&[0u8; 16]) == Some(0)
canfd_crc17_slice(&[0u8; 17]).is_none()
canfd_crc17_bit(0x20000, 0).is_none()
canfd_crc17_bit(0, 2).is_none()
```
