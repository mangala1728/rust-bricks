# 規格書：音訊頻譜質心與頻譜滾降特徵提取 (Spectral Features)

## 模組名稱
`dsp_spectral_features`

## 業務目標與適用場景
在語音識別、音樂風格分類、機器故障聲學診斷與人聲/雜訊檢測中，頻譜質心 (Spectral Centroid) 與頻譜滾降點 (Spectral Rolloff) 是最具辨識度的頻域聲學特徵。
本模組提供零外部依賴、純 `std` 實作的頻譜統計特徵提取微積木。

## 09 號合約邊界規範

### 函式 1：計算頻譜質心 (Spectral Centroid)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: sample_rate_hz [1.0..1000000.0]
  pub fn spectral_centroid(
      magnitudes: &[f64],
      sample_rate_hz: f64,
  ) -> Option<f64>
  ```
- **數學公式**：
  - 設頻率解析度 $\Delta f = \frac{f_s / 2}{N - 1}$ (其中 $N = \text{magnitudes.len()}$)
  - 質心 $C = \frac{\sum_{k=0}^{N-1} f_k \cdot M[k]}{\sum_{k=0}^{N-1} M[k]}$，其中 $f_k = k \cdot \Delta f$
  - 若所有振幅總和 $\le 1.0e-12$ 或 $N < 2$，回傳 `None`。

### 函式 2：計算頻譜滾降點 (Spectral Rolloff)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  /// @domain: rolloff_ratio [0.5..0.99]
  /// @domain: sample_rate_hz [1.0..1000000.0]
  pub fn spectral_rolloff(
      magnitudes: &[f64],
      sample_rate_hz: f64,
      rolloff_ratio: f64,
  ) -> Option<f64>
  ```
- **運算過程**：
  - 總能量 $E = \sum_{k=0}^{N-1} M[k]$
  - 閾值能量 $T = E \cdot \text{rolloff\_ratio}$
  - 累積能量 $\sum_{k=0}^{m} M[k] \ge T$ 之最小頻率 $f_m = m \cdot \Delta f$。
  - 回傳 `Some(f_m)`。

## 黃金測試向量 (Golden Vectors)
- 採樣率 $1000.0\text{ Hz}$，$N = 5$ 點頻譜，Nyquist 頻率為 $500.0\text{ Hz}$，$\Delta f = 125.0\text{ Hz}$：
  - 頻率軸：$[0.0, 125.0, 250.0, 375.0, 500.0]\text{ Hz}$
  - 對稱振幅：$[1.0, 2.0, 4.0, 2.0, 1.0]$，總和 $= 10.0$：
  - 質心分子 $= 0 \times 1 + 125 \times 2 + 250 \times 4 + 375 \times 2 + 500 \times 1 = 0 + 250 + 1000 + 750 + 500 = 2500$
  - 質心 $= 2500 / 10 = 250.0\text{ Hz}$。
  - 滾降點 ($\text{ratio} = 0.85$，$T = 8.5$)：
    $k=0: 1.0$
    $k=1: 3.0$
    $k=2: 7.0$
    $k=3: 9.0 \ge 8.5 \implies$ 滾降頻率為 $f_3 = 375.0\text{ Hz}$。
- 測試容差 `(val - golden).abs() < 1e-4`。
