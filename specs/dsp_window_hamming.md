# 規格書：廣義漢明窗函式 (Hamming Window Function)

## 模組名稱
`dsp_window_hamming`

## 業務目標與適用場景
在語音特徵提取 (如 MFCC、線性預測編碼 LPC)、聲學感測與通訊雷達中，漢明窗 (Hamming Window) 具備優異的主瓣集中度與 -43 dB 旁瓣衰減。
本模組提供零外部依賴、純 `std` 實作的漢明窗係數計算與就地緩衝區加窗微積木。

## 09 號合約邊界規範

### 函式 1：計算指定長度之單點漢明窗係數
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: index [0..window_len - 1]
  /// @domain: window_len [2..1048576]
  pub fn hamming_window_coeff(index: usize, window_len: usize) -> Option<f64>
  ```
- **數學公式 (Standard Hamming $\alpha = 0.54, \beta = 0.46$)**：
  - $w(n) = 0.54 - 0.46 \cdot \cos\left(\frac{2\pi n}{N - 1}\right)$
- **邊界防禦**：
  - 若 `window_len < 2` 或 `index >= window_len`，第一行立即回傳 `None`。

### 函式 2：對給定緩衝區原地套用漢明加窗 (In-place Windowing)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn apply_hamming_window(buffer: &mut [f64]) -> bool
  ```
- **運算過程**：
  - 若 `buffer.len() < 2`，立即回傳 `false`。
  - 對 `buffer` 的每個元素原地乘以 `hamming_window_coeff(i, n)`，全過程無溢位回傳 `true`。

## 黃金測試向量 (Golden Vectors)
- $N = 5$ 點漢明窗：
  - $n = 0$: $w(0) = 0.54 - 0.46\cos(0) = 0.54 - 0.46 = 0.08$
  - $n = 2$: $w(2) = 0.54 - 0.46\cos(\pi) = 0.54 + 0.46 = 1.00$
  - $n = 4$: $w(4) = 0.54 - 0.46\cos(2\pi) = 0.54 - 0.46 = 0.08$
- 測試斷言使用 `(val - golden).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_window_hamming.rs）
```golden
// 出處：規格 N = 5 點向量；Python 依 w(n)=0.54-0.46cos(2πn/(N-1)) 獨立驗算
(hamming_window_coeff(0, 5).unwrap() - 0.08).abs() < 1e-4
(hamming_window_coeff(2, 5).unwrap() - 1.0).abs() < 1e-4
(hamming_window_coeff(4, 5).unwrap() - 0.08).abs() < 1e-4
```
