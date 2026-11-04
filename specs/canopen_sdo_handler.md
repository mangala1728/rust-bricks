# 規格書：CANopen SDO 服務資料物件解析 (CANopen SDO Handler)

## 模組名稱
`canopen_sdo_handler`

## 業務目標與適用場景
在工業自動化、機械手臂關節驅動器與醫療儀器中，CANopen 協議的 SDO (Service Data Object) 用於讀寫節點物件字典 (Object Dictionary)。
本模組提供零拷貝解析 SDO 上傳與下載命令標頭、提取 16-bit Index 與 8-bit Sub-Index 的微積木。

## 09 號合約邊界規範

### 函式 1：解析 SDO 下載請求 (Initiate Download Request - 8 bytes)
- **函式簽名**：
  ```rust
  /// @outputs: 4
  pub fn parse_sdo_download_request(payload: &[u8; 8]) -> Option<(u8, u16, u8, usize)>
  ```
- **位元解析**：
  - Byte 0 (Command Specifier CS & Flags):
    - `ccs = (payload[0] >> 5) & 0x07` (1 = Initiate Download Request)
    - 若 `ccs != 1`，回傳 `None`。
    - `e = (payload[0] >> 1) & 0x01` (1 = expedited transfer, 0 = segmented transfer)
    - `s = payload[0] & 0x01` (1 = size indicated)
    - 若 $e=1$ 且 $s=1$：未使用的 byte 數 $n = (payload[0] >> 2) & 0x03$，有效數據長度為 $4 - n$。
    - 若 $e=1$ 且 $s=0$：長度未指定，預設 4。
    - 若 $e=0$：為分段傳輸，返回 `(1, index, sub_index, 0)`。
  - Byte 1..2: 16-bit Index (Little Endian: `payload[1] as u16 | ((payload[2] as u16) << 8)`)
  - Byte 3: 8-bit Sub-Index (`payload[3]`)
  - 回傳 `Some((ccs, index, sub_index, valid_data_len))`。

### 函式 2：編碼 SDO 終止中止碼 (SDO Abort Response)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn encode_sdo_abort(index: u16, sub_index: u8, abort_code: u32, out_frame: &mut [u8; 8]) -> bool
  ```
- **訊框格式**：
  - `out_frame[0] = 0x80` (CS = 4: Abort SDO Transfer)
  - `out_frame[1] = (index & 0xFF) as u8`
  - `out_frame[2] = ((index >> 8) & 0xFF) as u8`
  - `out_frame[3] = sub_index`
  - `out_frame[4..8]` 放 32-bit abort_code (Little Endian)。
  - 回傳 `true`。

## 黃金測試向量 (Golden Vectors - CiA 301 Standard)
- 下載請求 Payload: `[0x2B, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00]`
  - `0x2B` 二進位: `0010 1011`
    - `ccs = (0x2B >> 5) = 1` (Initiate Download)
    - `e = 1` (Expedited)
    - `s = 1` (Size indicated)
    - `n = (0x2B >> 2) & 3 = 2` (未用 2 bytes，有效長度 = $4 - 2 = 2$)
  - Index = `0x1017` (Heartbeat Producer Time)
  - Sub-index = `0x00`
  - 解析應返回 `Some((1, 0x1017, 0, 2))`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_canopen_sdo_handler.rs）
```golden
// 出處：CiA 301 黃金向量 [0x2B,0x17,0x10,0x00,...]（手算 ccs=1,e=1,s=1,n=2 → 長度 2，Index 0x1017）
parse_sdo_download_request(&[0x2B, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 2))
// 出處：位元解析規則 e=1,s=0 → 長度預設 4（0x22：ccs=1,e=1,s=0，手算）
parse_sdo_download_request(&[0x22, 0x17, 0x10, 0x00, 0x42, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 4))
// 出處：位元解析規則 e=0 → 分段傳輸，長度 0（0x20：ccs=1,e=0，手算）
parse_sdo_download_request(&[0x20, 0x17, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]) == Some((1, 0x1017, 0, 0))
// 出處：位元解析規則 ccs != 1 → None（0x40：ccs=2）
parse_sdo_download_request(&[0x40, 0x17, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00]).is_none()
// 出處：函式 2 訊框格式（手算：0x80, Index LE, Sub-Index, abort_code LE；0x06020000 → 00 00 02 06）
let mut frame = [0u8; 8];
encode_sdo_abort(0x1017, 0x00, 0x0602_0000, &mut frame)
frame == [0x80, 0x17, 0x10, 0x00, 0x00, 0x00, 0x02, 0x06]
```
