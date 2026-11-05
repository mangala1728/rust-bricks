# 模組名稱: ISO 14229 UDS 統一診斷服務服務碼與正負回應解構器 (UDS ISO 14229 SID & NRC Parser)

## 1. 目標定位
- 目標模組檔: `src/uds_iso14229_sid.rs`
- 目標測試檔: `tests/test_uds_iso14229_sid.rs`
- 功能簡述: 解析 ISO 14229-1 車載診斷請求服務碼 (SID)，計算正回應 ID (SID + 0x40)，並對負回應訊框 (0x7F, Rejected SID, NRC) 進行結構化解碼。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: ISO 14229-1 UDS diagnostic request/response service ID and NRC decoder
// Inputs: sid: u8, frame: &[u8]
// Outputs: Option<u8> or Option<(u8, u8)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: sid in [0x10..=0x3E] || sid in [0x83..=0x88] || sid in [0xBA..=0xBE]
pub fn uds_positive_response_id(sid: u8) -> Option<u8>;

/// @outputs: 2
/// @domain: frame.len() >= 3, frame[0] == 0x7F
pub fn uds_parse_negative_response(frame: &[u8]) -> Option<(u8, u8)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `uds_positive_response_id`:
  - 允許標準 SID 範圍：`0x10..=0x3E`（診斷與通信管理、資料傳輸、傳輸儲存、輸入輸出控制）、`0x83..=0x88`、`0xBA..=0xBE`。
  - 其餘未定義或已經是回應碼（高位元已加 0x40）之值立即回傳 `None`。
  - 正回應 ID 為 `sid.checked_add(0x40)`。
- `uds_parse_negative_response`:
  - 若 `frame.len() < 3` 或 `frame[0] != 0x7F`，立即回傳 `None`。
  - 回傳 `Some((rejected_sid, nrc))`，其中 `rejected_sid = frame[1]`, `nrc = frame[2]`。

## 4. 確定性黃金值測試要求
- `sid = 0x10` (DiagnosticSessionControl) -> 正回應 ID `0x50`
- `sid = 0x22` (ReadDataByIdentifier) -> 正回應 ID `0x62`
- `sid = 0x50` -> 已經是回應碼，回傳 `None`
- 負回應 `[0x7F, 0x22, 0x31]` (RequestOutOfRange) -> `(0x22, 0x31)`
- 負回應長度小於 3 或標頭非 0x7F -> `None`

```golden
uds_positive_response_id(0x10) == Some(0x50)
uds_positive_response_id(0x22) == Some(0x62)
uds_positive_response_id(0x19) == Some(0x59)
uds_positive_response_id(0x27) == Some(0x67)
uds_positive_response_id(0x50).is_none()
uds_positive_response_id(0x00).is_none()
uds_parse_negative_response(&[0x7F, 0x22, 0x31]) == Some((0x22, 0x31))
uds_parse_negative_response(&[0x7F, 0x10, 0x12, 0x00]) == Some((0x10, 0x12))
uds_parse_negative_response(&[0x7E, 0x22, 0x31]).is_none()
uds_parse_negative_response(&[0x7F, 0x22]).is_none()
```
