# 規格書：Modbus RTU 訊框編解碼與 CRC-16 檢驗 (Modbus RTU Frame)

## 模組名稱
`modbus_rtu_frame`

## 業務目標與適用場景
在工業物聯網 (IIoT)、PLC 自動化控制與智慧電表中，Modbus RTU 透過 RS-485 序列通訊進行主從資料交換。
本模組提供零外部依賴、純 `std` 實作的 Modbus RTU 訊框驗證、CRC-16 計算與異常回應訊框編碼微積木。

## 09 號合約邊界規範

### 函式 1：Modbus RTU 標準 CRC-16 查表/運算 (多項式 0xA001)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn modbus_crc16(data: &[u8]) -> u16
  ```
- **演算法**：
  - 初始值 $0xFFFF$，多項式 $0xA001$ (反向 $0x8005$)。
  - 每個 byte 進行 XOR，右移 8 次，若 LSB 為 1 則 XOR $0xA001$。
  - 回傳小端序校驗碼 (Low byte first in Modbus stream)。

### 函式 2：Modbus RTU 訊框合法性驗證 (Frame Validate)
- **函式簽名**：
  ```rust
  /// @outputs: 2
  pub fn modbus_validate_frame(frame: &[u8]) -> Option<(u8, u8, usize)>
  ```
- **運算過程**：
  - 若 `frame.len() < 4` (最少含 Slave ID, Function Code, 2-byte CRC)，回傳 `None`。
  - 取出末尾 2 bytes，與前面數據計算之 CRC-16 比對 (末尾為 `[crc_lo, crc_hi]`)。
  - 比對失敗回傳 `None`。
  - 成功回傳 `Some((slave_id, function_code, payload_len))`。

## 黃金測試向量 (Golden Vectors - Modbus Protocol Standard)
- 測試數據：`[0x01, 0x03, 0x00, 0x00, 0x00, 0x01]` (讀取 Slave 1 保持暫存器 0):
  - 計算出的 CRC-16 暫存器值為 `0x0A84` (低位組 `0x84`，高位組 `0x0A`)。
  - Modbus 序列傳輸低位在前，完整合規訊框為 `[0x01, 0x03, 0x00, 0x00, 0x00, 0x01, 0x84, 0x0A]`。
  - 通過驗證，回傳 `Some((1, 3, 4))`。
