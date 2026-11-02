# 規格書：LIN 2.1 車身總線訊框校驗 (LIN 2.1 Frame Validator)

## 模組名稱
`lin_frame_validator`

## 業務目標與適用場景
在汽車電子（車門、車窗、雨刷、空調風門控制）中，LIN (Local Interconnect Network) 2.1 總線廣泛採用低成本單線通訊。
本模組提供硬體級識別碼 (Protected Identifier, PID) 奇偶校驗計算與增強型校驗和 (Enhanced Checksum) 驗證。

## 09 號合約邊界規範

### 函式 1：LIN PID 奇偶校驗碼計算 (Parity Calculation)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: frame_id [0..63]
  pub fn lin_calc_pid(frame_id: u8) -> Option<u8>
  ```
- **位元公式**：
  - 若 `frame_id > 63`，回傳 `None`。
  - $P_0 = ID_0 \oplus ID_1 \oplus ID_2 \oplus ID_4$
  - $P_1 = \neg (ID_1 \oplus ID_3 \oplus ID_4 \oplus ID_5)$
  - 回傳：$frame\_id \;|\; (P_0 \ll 6) \;|\; (P_1 \ll 7)$。

### 函式 2：LIN 2.1 增強型校驗和計算 (Enhanced Checksum)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn lin_enhanced_checksum(pid: u8, data: &[u8]) -> u8
  ```
- **運算過程**：
  - 累加器以 `u16` 加總 `pid` 與所有 `data` 位元組。
  - 進位折疊：`while sum > 0xFF { sum = (sum & 0xFF) + (sum >> 8); }`
  - 取反碼：`(!sum) as u8`。

## 黃金測試向量 (Golden Vectors - LIN 2.1 Specification)
- PID 向量：
  - Frame ID = `0x01` (二進位 000001):
    $P_0 = 1 \oplus 0 \oplus 0 \oplus 0 = 1$
    $P_1 = \neg (0 \oplus 0 \oplus 0 \oplus 0) = 1$
    PID = `0x01 | (1 << 6) | (1 << 7)` = `0xC1`。
- 增強校驗和：
  - PID = `0xC1`, Data = `[0x01, 0x02, 0x03]`
  - Sum = $0xC1 + 1 + 2 + 3 = 0xC7 = 199$
  - Inverted = `!0xC7` = `0x38`。
