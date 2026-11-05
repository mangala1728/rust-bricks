# 模組名稱: 工業乙太網 EtherCAT 協定 10 位元組資料報標頭解析器 (EtherCAT Datagram Header Parser)

## 1. 目標定位
- 目標模組檔: `src/ethercat_datagram_header.rs`
- 目標測試檔: `tests/test_ethercat_datagram_header.rs`
- 功能簡述: 依據 IEC 61158-4-12 規範解析 EtherCAT 訊框內的 10 位元組資料報 (Datagram) 標頭。解構 Command (Cmd), Index (Idx), Address (32-bit), Data Length (11-bit), More Datagrams (M flag), 與 IRQ (16-bit)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: IEC 61158-4-12 EtherCAT 10-byte datagram header parser
// Inputs: header: &[u8]
// Outputs: Option<EthercatDatagramHeader>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EthercatDatagramHeader {
    pub cmd: u8,
    pub idx: u8,
    pub address: u32,
    pub data_len: u16,
    pub more_follows: bool,
    pub irq: u16,
}

/// @outputs: 1
/// @domain: header.len() >= 10, cmd > 0 && cmd <= 14, data_len <= 1498
pub fn ethercat_parse_datagram_header(header: &[u8]) -> Option<EthercatDatagramHeader>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `header.len() < 10`，立即回傳 `None`。
- 解碼欄位 (Little-Endian，EtherCAT 標準為低位優先)：
  - `cmd = header[0]`
  - `idx = header[1]`
  - `address = u32::from_le_bytes([header[2], header[3], header[4], header[5]])`
  - `len_flags = u16::from_le_bytes([header[6], header[7]])`
  - `data_len = len_flags & 0x07FF` (前 11 個位元為長度，0..=2047)
  - `more_follows = (len_flags & 0x8000) != 0` (Bit 15 為 More Datagrams 標記)
  - `irq = u16::from_le_bytes([header[8], header[9]])`
- 合約邊界守衛：
  - `cmd == 0 || cmd > 14` $\rightarrow$ `None` (EtherCAT 定義指令碼為 1 到 14)
  - `data_len > 1498` $\rightarrow$ `None` (以太網最大標準訊框有效資料量上限)
- 成功回傳 `Some(EthercatDatagramHeader { ... })`。

## 4. 確定性黃金值測試要求
- 標準封包解析：
  - `Cmd = 0x01` (APRD, 讀取), `Idx = 0x05`, `Address = 0x00010000`, `Length = 0x0004` (4 bytes, More=false), `IRQ = 0x0000`
- 命令碼為 0 或超過 14、長度超過 1498 時返回 `None`。

```golden
let raw = [0x01, 0x05, 0x00, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00];
let parsed = ethercat_parse_datagram_header(&raw).unwrap();
parsed.cmd == 1
parsed.idx == 5
parsed.address == 0x00010000
parsed.data_len == 4
parsed.more_follows == false
parsed.irq == 0
let mut more_raw = raw; more_raw[7] = 0x80;
let parsed_more = ethercat_parse_datagram_header(&more_raw).unwrap();
parsed_more.more_follows == true
parsed_more.data_len == 4
let mut bad_cmd = raw; bad_cmd[0] = 0;
ethercat_parse_datagram_header(&bad_cmd).is_none()
ethercat_parse_datagram_header(&raw[0..9]).is_none()
```
