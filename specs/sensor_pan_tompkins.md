# 規格書：Pan-Tompkins 心電圖 QRS 波檢波步進 (Pan-Tompkins QRS Step)

## 模組名稱
`sensor_pan_tompkins`

## 業務目標與適用場景
在心律穿戴設備、智慧手環與遠距醫療監護儀中，Pan-Tompkins 是公認最具強健性的 ECG QRS 波實時檢測演算法。
本模組提供零依賴的五點微分濾波器 (Derivative Filter) 與移動窗口積分 (Moving Window Integration) 步進微積木。

## 09 號合約邊界規範

### 函式 1：Pan-Tompkins 5點導數微分單步 (Derivative Step)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: x_n [-1.0e6..1.0e6]
  pub fn pan_tompkins_derivative_step(
      x_n: f64,
      history: &mut [f64; 4],
  ) -> Option<f64>
  ```
- **數學公式**：
  - 標準轉移函式 $H(z) = \frac{1}{8T} (2 + z^{-1} - z^{-3} - 2z^{-4})$
  - 差分方程：$y[n] = \frac{1}{8} \left(2 x[n] + x[n-1] - x[n-3] - 2 x[n-4]\right)$
  - 歷史狀態：`history` 記錄 $[x[n-1], x[n-2], x[n-3], x[n-4]]$。
  - 更新後將歷史狀態右移並壓入 $x[n]$。

### 函式 2：平方運算與移動窗口平均積分 (Moving Window Integration Step)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: derivative [-1.0e6..1.0e6]
  pub fn pan_tompkins_integrate_step(
      derivative: f64,
      ring_buf: &mut [f64],
      head_idx: &mut usize,
      running_sum: &mut f64,
  ) -> Option<f64>
  ```
- **運算過程**：
  - 平方：$s[n] = (\text{derivative})^2$
  - 環形緩衝區滑動維護 $O(1)$ 總和：
    $*running\_sum = *running\_sum - ring\_buf[*head\_idx] + s[n]$
  - $ring\_buf[*head\_idx] = s[n]$
  - $*head\_idx = (*head\_idx + 1) \% ring\_buf.len()$
  - 回傳：$*running\_sum / ring\_buf.len()$。

## 黃金測試向量 (Golden Vectors)
- 導數濾波：
  - 輸入序列 $[0.0, 0.0, 0.0, 0.0, 8.0]$，前 4 點為 0，第 5 點為 8.0：
    $y[n] = (2 \times 8.0 + 0 - 0 - 0) / 8 = 2.0$。
- 積分滑窗：
  - 窗口大小 $4$，初始全 0。輸入導數分別為 $2.0$ (平方 $4.0$)：
    窗口總和 $= 4.0$，輸出 $= 1.0$。
- 容差使用 `(val - golden).abs() < 1e-4`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_sensor_pan_tompkins.rs）
```golden
// 出處：規格向量：歷史全 0、x[n] = 8 → y = (2×8)/8 = 2.0（手算）
let mut pt_hist = [0.0_f64; 4];
(pan_tompkins_derivative_step(8.0, &mut pt_hist).unwrap() - 2.0).abs() < 1e-4
// 出處：函式 1：history = [x[n-1], x[n-2], x[n-3], x[n-4]]，右移並壓入 x[n] → [8, 0, 0, 0]
pt_hist == [8.0, 0.0, 0.0, 0.0]
// 出處：規格向量：窗口 4、初始全 0，導數 2.0 → 平方 4，總和 4，輸出 1.0（手算）
let mut pt_ring = [0.0_f64; 4];
let mut pt_head = 0_usize;
let mut pt_sum = 0.0_f64;
(pan_tompkins_integrate_step(2.0, &mut pt_ring, &mut pt_head, &mut pt_sum).unwrap() - 1.0).abs() < 1e-4
// 出處：函式 2 運算過程：running_sum = 4、ring_buf[0] = 4、head_idx = 1
(pt_sum - 4.0).abs() < 1e-12 && (pt_ring[0] - 4.0).abs() < 1e-12 && pt_head == 1
```
