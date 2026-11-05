# 模組名稱: 粉紅雜訊三極點濾波器 (Pink Noise Filter)

## 1. 目標定位
- 目標模組檔: `src/dsp_pink_noise.rs`
- 目標測試檔: `tests/test_dsp_pink_noise.rs`
- 功能簡述: 採用 Kellet 簡化三極點 IIR 濾波器，將均勻白雜訊採樣整形為具備 -3 dB/oct (1/f) 功率譜斜率之粉紅雜訊。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 3-pole IIR pink noise generator filter
// Inputs: white: f64, state: &mut [f64; 3]
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: white in [-1.0..=1.0], state elements are finite
pub fn dsp_pink_filter_step(white: f64, state: &mut [f64; 3]) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `white < -1.0 || white > 1.0 || !white.is_finite()` 或 `state` 任一元素非有限數時，立即回傳 `None`。
- 運算更新：
  $$s_0 = 0.99765 \times s_0 + white \times 0.0990460$$
  $$s_1 = 0.96300 \times s_1 + white \times 0.2965164$$
  $$s_2 = 0.57000 \times s_2 + white \times 1.0526913$$
  $$\text{output} = s_0 + s_1 + s_2 + white \times 0.1848$$

## 4. 確定性黃金值測試要求
- 初始狀態 `[0.0, 0.0, 0.0]`，輸入 `white = 1.0`：
  $\text{output} = 0.099046 + 0.2965164 + 1.0526913 + 0.1848 = 1.6330537$
- 容差 `< 1e-5`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_pink_noise.rs）
```golden
let mut s = [0.0, 0.0, 0.0];
(dsp_pink_filter_step(1.0, &mut s).unwrap() - 1.6330537).abs() < 1e-5
(s[0] - 0.0990460).abs() < 1e-6
(s[1] - 0.2965164).abs() < 1e-6
(s[2] - 1.0526913).abs() < 1e-6
dsp_pink_filter_step(1.5, &mut s).is_none()
dsp_pink_filter_step(-1.5, &mut s).is_none()
dsp_pink_filter_step(f64::NAN, &mut s).is_none()
```
