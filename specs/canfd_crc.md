# 模組名稱: CAN-FD 17位元循環冗餘校驗碼 (CAN-FD CRC-17)

## 1. 目標定位
- 目標模組檔: `src/canfd_crc.rs`
- 目標測試檔: `tests/test_canfd_crc.rs`
- 功能簡述: 依據 ISO 11898-1:2015 規範，實作 CAN-FD 傳輸訊框（負載 <= 16 位元組）之 17-bit CRC 多項式 $x^{17} + x^{16} + x^{14} + x^{13} + x^{11} + x^6 + x^4 + x^3 + x^1 + 1$ (0x1685B) 校驗計算。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: ISO 11898-1:2015 CAN-FD 17-bit CRC calculation
// Inputs: crc: u32, bit: u8, data: &[u8]
// Outputs: Option<u32> or u32
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: crc <= 0x1FFFF, bit <= 1
pub fn canfd_crc17_bit(crc: u32, bit: u8) -> Option<u32>;

/// @outputs: 1
/// @domain: data.len() <= 64
pub fn canfd_crc17_slice(data: &[u8]) -> Option<u32>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `canfd_crc17_bit`: 若 `crc > 0x1FFFF || bit > 1`，回傳 `None`。
  - 計算：`let msb = (crc >> 16) & 1;`
  - `let next = ((crc << 1) & 0x1FFFF) | (bit as u32);`
  - 若 `msb != 0` 則 `next ^ 0x1685B`，否則 `next`。回傳 `Some(res)`。
- `canfd_crc17_slice`: 若 `data.len() > 64`，回傳 `None`。自 MSB (bit 7) 至 LSB (bit 0) 逐位元推遞，初始 CRC 為 0。

## 4. 確定性黃金值測試要求
- `canfd_crc17_bit(0x10000, 0)` -> MSB=1，移位後為 0，異或 0x1685B -> `0x1685B` (92251)
- `canfd_crc17_bit(0x00001, 1)` -> MSB=0，移位後為 `(1 << 1) | 1 = 3`
- `canfd_crc17_slice(&[])` -> `0`
- `canfd_crc17_slice(&[0x80])` -> `128`
- `canfd_crc17_slice(&[0x00])` -> `0`

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_canfd_crc.rs）
```golden
canfd_crc17_bit(0x10000, 0) == Some(0x1685B)
canfd_crc17_bit(0x00001, 1) == Some(3)
canfd_crc17_bit(0x20000, 0).is_none()
canfd_crc17_bit(0, 2).is_none()
canfd_crc17_slice(&[]) == Some(0)
canfd_crc17_slice(&[0x00]) == Some(0)
canfd_crc17_slice(&[0x80]) == Some(128)
```
