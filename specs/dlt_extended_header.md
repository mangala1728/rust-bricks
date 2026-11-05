# 模組名稱: AUTOSAR DLT 擴展標頭解析器 (DLT Extended Header Parser)

## 1. 目標定位
- 目標模組檔: `src/dlt_extended_header.rs`
- 目標測試檔: `tests/test_dlt_extended_header.rs`
- 功能簡述: 依據 AUTOSAR PRS_DLT_00001 規範，解析車載診斷日誌之 10 位元組擴展標頭（MSIN 訊息資訊、NOAR 參數個數、APID 應用程式識別碼、CTID 上下文識別碼）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: AUTOSAR DLT extended header binary field parser
// Inputs: buf: &[u8]
// Outputs: Option<(bool, u8, u8, u8, [u8; 4], [u8; 4])>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 6
/// @domain: buf.len() >= 10
pub fn dlt_parse_extended_header(
    buf: &[u8],
) -> Option<(bool, u8, u8, u8, [u8; 4], [u8; 4])>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `buf.len() < 10`，立即回傳 `None`。
- 位元拆解：
  - `msin = buf[0];`
  - `is_verbose = (msin & 0x01) != 0;`
  - `msg_type = (msin >> 1) & 0x07;` (0..7)
  - `log_level = (msin >> 4) & 0x0F;` (0..15)
  - `noar = buf[1];`
  - `app_id = [buf[2], buf[3], buf[4], buf[5]];`
  - `context_id = [buf[6], buf[7], buf[8], buf[9]];`
  - 回傳 `Some((is_verbose, msg_type, log_level, noar, app_id, context_id))`。

## 4. 確定性黃金值測試要求
- 向量 1:
  `buf = [0x41, 0x02, b'A', b'P', b'P', b'1', b'C', b'T', b'X', b'1']`
  - `msin = 0x41`: `is_verbose = true`, `msg_type = (0x41 >> 1) & 7 = 0` (Log), `log_level = 0x41 >> 4 = 4` (Info)
  - `noar = 2`
  - `app_id = b"APP1"`, `context_id = b"CTX1"`
- 長度不足 10 位元組回傳 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dlt_extended_header.rs）
```golden
let pkt = [0x41, 0x02, b'A', b'P', b'P', b'1', b'C', b'T', b'X', b'1'];
let (verb, mtype, level, noar, app, ctx) = dlt_parse_extended_header(&pkt).unwrap();
verb == true && mtype == 0 && level == 4 && noar == 2
&app == b"APP1" && &ctx == b"CTX1"
dlt_parse_extended_header(&[0x41, 0x02, b'A']).is_none()
```
