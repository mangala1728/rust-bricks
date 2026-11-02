# 規格書：一維自適應卡爾曼濾波器單步 (1D Adaptive Kalman Filter Step)

## 模組名稱
`sensor_kalman_1d`

## 業務目標與適用場景
在無人機高度定高（氣壓高度計濾波）、智慧手錶溫度與氣壓平滑、電池荷電狀態 (SoC) 單變數估計中，一維卡爾曼濾波器能在測量雜訊 $R$ 與過程雜訊 $Q$ 下遞迴求解最優最小均方誤差狀態估計。
本模組提供零外部依賴、純 `std` 實作的一維卡爾曼濾波單步更新微積木。

## 09 號合約邊界規範

### 函式 1：一維卡爾曼濾波器單步預測與更新 (Kalman 1D Step)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: q_noise [0.0..1000.0]
  /// @domain: r_noise [1.0e-9..1000.0]
  pub fn kalman_1d_step(
      state: &mut (f64, f64),
      measurement: f64,
      q_noise: f64,
      r_noise: f64,
  ) -> Option<f64>
  ```
- **狀態與參數定義**：
  - `state.0`: 狀態估計值 $\hat{x}$
  - `state.1`: 估計誤差共變異數 $P$ (必須 $\ge 0.0$)
  - `measurement`: 當前觀測量 $z$
  - `q_noise`: 過程雜訊共變異數 $Q$
  - `r_noise`: 測量雜訊共變異數 $R$
- **運算過程**：
  - 邊界檢查：`q_noise < 0.0 || r_noise < 1.0e-9 || state.1 < 0.0 || !measurement.is_finite()` 回傳 `None`。
  - 時間更新（預測）：
    - $P^- = state.1 + q\_noise$
  - 測量更新：
    - 卡爾曼增益：$K = \frac{P^-}{P^- + r\_noise}$
    - 狀態修正：$\hat{x} = state.0 + K \cdot (measurement - state.0)$
    - 共變異數更新：$P = (1.0 - K) \cdot P^-$
  - 寫回 `state.0 = \hat{x}`，`state.1 = P`。
  - 回傳 `Some(\hat{x})`。

## 黃金測試向量 (Golden Vectors)
- 初始狀態：$\hat{x}_0 = 0.0$，$P_0 = 1.0$。
- 雜訊參數：$Q = 0.1$，$R = 0.4$。
- 第 1 步測量 $z_1 = 1.0$：
  - $P^- = 1.1$，$K = 1.1 / 1.5 \approx 0.733333$。
  - 更新後 $\hat{x}_1 \approx 0.733333$，$P_1 \approx 0.293333$。
  - 斷言 `(state.0 - 0.733333).abs() < 1e-4`。
- 第 2 步測量 $z_2 = 1.0$：
  - $P^- = 0.393333$，$K \approx 0.495798$。
  - 更新後 $\hat{x}_2 \approx 0.865546$，$P_2 \approx 0.198319$。
  - 斷言 `(state.0 - 0.865546).abs() < 1e-4`。
