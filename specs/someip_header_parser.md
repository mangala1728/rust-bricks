# 模組名稱: AUTOSAR SOME/IP 協定 16 位元組報頭零拷貝解析器 (AUTOSAR SOME/IP Header Parser)

## 1. 目標定位
- 目標模組檔: `src/someip_header_parser.rs`
- 目標測試檔: `tests/test_someip_header_parser.rs`
- 功能簡述: AUTOSAR 車用乙太網路 SOME/IP 協定 16 位元組固定標頭解析，解構 Message ID (Service ID, Method/Event ID), Length, Request ID (Client ID, Session ID), Protocol Version, Interface Version, Message Type, Return Code。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: AUTOSAR SOME/IP 16-byte fixed header parser
// Inputs: header: &[u8]
// Outputs: Option<SomeIpHeader>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SomeIpHeader {
    pub service_id: u16,
    pub method_id: u16,
    pub length: u32,
    pub client_id: u16,
    pub session_id: u16,
    pub protocol_version: u8,
    pub interface_version: u8,
    pub message_type: u8,
    pub return_code: u8,
}

/// @outputs: 1
/// @domain: header.len() >= 16, protocol_version == 1, length >= 8
pub fn someip_parse_header(header: &[u8]) -> Option<SomeIpHeader>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `header.len() < 16`，立即回傳 `None`。
- 欄位解碼 (Big-Endian)：
  - `service_id = u16::from_be_bytes([header[0], header[1]])`
  - `method_id = u16::from_be_bytes([header[2], header[3]])`
  - `length = u32::from_be_bytes([header[4], header[5], header[6], header[7]])`
  - `client_id = u16::from_be_bytes([header[8], header[9]])`
  - `session_id = u16::from_be_bytes([header[10], header[11]])`
  - `protocol_version = header[12]`
  - `interface_version = header[13]`
  - `message_type = header[14]`
  - `return_code = header[15]`
- 合約驗證守衛：
  - `protocol_version != 1` 時回傳 `None`（SOME/IP 目前標準協定版本為 0x01）。
  - `length < 8` 時回傳 `None`（Length 欄位計算包含 Request ID 至 Payload，至少包含 8 位元組標頭後續部分）。

## 4. 確定性黃金值測試要求
- 標準封包解析：
  - Service ID: 0x1234, Method ID: 0x0001, Length: 0x00000008, Client ID: 0x0001, Session ID: 0x0002, Protocol: 0x01, Interface: 0x01, Msg Type: 0x00 (Request), Return: 0x00 (E_OK)
- 長度不足 16 位元組或 Protocol Version != 1 拒絕。

```golden
let hdr_bytes = [0x12, 0x34, 0x00, 0x01, 0x00, 0x00, 0x00, 0x08, 0x00, 0x01, 0x00, 0x02, 0x01, 0x01, 0x00, 0x00];
let parsed = someip_parse_header(&hdr_bytes).unwrap();
parsed.service_id == 0x1234
parsed.method_id == 0x0001
parsed.length == 8
parsed.client_id == 1
parsed.session_id == 2
parsed.protocol_version == 1
parsed.interface_version == 1
parsed.message_type == 0
parsed.return_code == 0
someip_parse_header(&hdr_bytes[0..15]).is_none()
let mut bad_ver = hdr_bytes; bad_ver[12] = 2;
someip_parse_header(&bad_ver).is_none()
let mut bad_len = hdr_bytes; bad_len[7] = 7;
someip_parse_header(&bad_len).is_none()
```
