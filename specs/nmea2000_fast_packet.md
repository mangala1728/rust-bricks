# 模組名稱: NMEA 2000 快速封包分包標頭與重組 (NMEA 2000 Fast Packet)

## 1. 目標定位
- 目標模組檔: `src/nmea2000_fast_packet.rs`
- 目標測試檔: `tests/test_nmea2000_fast_packet.rs`
- 功能簡述: 依據 NMEA 2000 / IEC 61162-3 航海儀表總線標準，解碼 Fast Packet 多訊框傳輸（高達 223 位元組）之首訊框標頭與序號校驗。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: NMEA 2000 fast packet protocol multi-frame header parser
// Inputs: first_byte: u8, can_frame: &[u8; 8], out_buf: &mut [u8; 223]
// Outputs: (u8, u8) or Option<(u8, usize)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: first_byte <= 0xFF
pub fn nmea2000_parse_header(first_byte: u8) -> (u8, u8);

/// @outputs: 2
/// @domain: can_frame.len() == 8
pub fn nmea2000_extract_first_frame(
    can_frame: &[u8; 8],
    out_buf: &mut [u8; 223],
) -> Option<(u8, usize)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `nmea2000_parse_header`:
  - `sequence_id = (first_byte >> 5) & 0x07;` (0..7)
  - `frame_index = first_byte & 0x1F;` (0..31)
  - 回傳 `(sequence_id, frame_index)`。
- `nmea2000_extract_first_frame`:
  - 必須為首訊框：`frame_index != 0` 時回傳 `None`。
  - `total_len = can_frame[1] as usize;` 若 `total_len == 0 || total_len > 223` 回傳 `None`。
  - 將首訊框資料（最多 6 位元組，即 `min(6, total_len)`）複製至 `out_buf` 前段。
  - 回傳 `Some((sequence_id, total_len))`。

## 4. 確定性黃金值測試要求
- 標頭解析：`first_byte = 0x40` (`0b010_00000`) -> `seq = 2, frame = 0`
- 標頭解析：`first_byte = 0xA3` (`0b101_00011`) -> `seq = 5, frame = 3`
- 首訊框萃取：
  - `frame = [0x40, 10, b'A', b'B', b'C', b'D', b'E', b'F']`
  - 應成功回傳 `Some((2, 10))` 且 `out_buf[0..6] == b"ABCDEF"`。
  - 若 `frame[0] = 0x41`（非首訊框），回傳 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_nmea2000_fast_packet.rs）
```golden
nmea2000_parse_header(0x40) == (2, 0)
nmea2000_parse_header(0xA3) == (5, 3)
let mut buf = [0u8; 223];
let f = [0x40, 10, b'A', b'B', b'C', b'D', b'E', b'F'];
nmea2000_extract_first_frame(&f, &mut buf) == Some((2, 10))
&buf[..6] == b"ABCDEF"
let f_not_first = [0x41, 10, 0, 0, 0, 0, 0, 0];
nmea2000_extract_first_frame(&f_not_first, &mut buf).is_none()
let f_overflow = [0x40, 250, 0, 0, 0, 0, 0, 0];
nmea2000_extract_first_frame(&f_overflow, &mut buf).is_none()
```
