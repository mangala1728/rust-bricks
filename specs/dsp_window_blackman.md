# 規格書：精確布萊克曼窗 (Blackman Window Function)

## 模組名稱
`dsp_window_blackman`

## 業務目標與適用場景
在頻譜分析 (FFT)、語音特徵提取 (STFT) 與 FIR 數位濾波器設計中，窗函式用於抑制頻譜洩漏（Spectral Leakage）。
布萊克曼窗 (Blackman Window) 擁有 -58 dB 的極低旁瓣峰值衰減，顯著優於矩形窗與漢明窗，是音訊特徵提取的業界黃金標準。

## 09 號合約邊界規範

### 函式 1：計算指定長度之單點布萊克曼窗係數
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: index [0..window_len - 1]
  /// @domain: window_len [2..1048576]
  pub fn blackman_window_coeff(index: usize, window_len: usize) -> Option<f64>
  ```
- **數學公式 (Blackman 3-term)**：
  - $w(n) = 0.42 - 0.5 \cdot \cos\left(\frac{2\pi n}{N - 1}\right) + 0.08 \cdot \cos\left(\frac{4\pi n}{N - 1}\right)$
- **邊界防禦**：
  - 若 `window_len < 2` 或 `index >= window_len`，第一行立即回傳 `None`。

### 函式 2：對給定緩衝區原地套用布萊克曼加窗 (In-place Windowing)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn apply_blackman_window(buffer: &mut [f64]) -> bool
  ```
- **運算過程**：
  - 若 `buffer.len() < 2`，立即回傳 `false`。
  - 對 `buffer` 的每個元素原地乘以 `blackman_window_coeff(i, n)`，若全過程無溢位回傳 `true`。

## 黃金測試向量 (Golden Vectors)
- $N = 5$ 點布萊克曼窗：
  - $n = 0$: $w(0) = 0.42 - 0.5\cos(0) + 0.08\cos(0) = 0.42 - 0.5 + 0.08 = 0.0$
  - $n = 2$: $w(2) = 0.42 - 0.5\cos(\pi) + 0.08\cos(2\pi) = 0.42 + 0.5 + 0.08 = 1.0$
  - $n = 4$: $w(4) = 0.42 - 0.5\cos(2\pi) + 0.08\cos(4\pi) = 0.42 - 0.5 + 0.08 = 0.0$
- 測試斷言嚴禁使用弱斷言，一律使用 `(val - golden).abs() < 1e-4`。
