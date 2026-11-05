# 模組名稱: 雙時間常數音訊動態包絡檢測器 (Audio Dynamic Envelope Follower)

## 1. 目標定位
- 目標模組檔: `src/dsp_envelope_follower.rs`
- 目標測試檔: `tests/test_dsp_envelope_follower.rs`
- 功能簡述: 即時動態音訊包絡檢測純函式。具備獨立之起音 (Attack) 與釋音 (Release) 時間濾波係數，廣泛應用於動態範圍壓縮器 (Compressor)、雜音門 (Noise Gate) 與語音活動特徵提取。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Real-time audio envelope follower with attack/release coefficients
// Inputs: input_sample: f64, prev_env: f64, attack_coeff: f64, release_coeff: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: input_sample in [-1e6..=1e6], prev_env >= 0.0, attack_coeff in [0.0..=1.0], release_coeff in [0.0..=1.0]
pub fn dsp_envelope_follower_step(
    input_sample: f64,
    prev_env: f64,
    attack_coeff: f64,
    release_coeff: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何輸入包含 `NaN` 或 `Infinite`，立即回傳 `None`。
- 若 `prev_env < 0.0`，立即回傳 `None`。
- 若 `attack_coeff < 0.0 || attack_coeff > 1.0 || release_coeff < 0.0 || release_coeff > 1.0`，立即回傳 `None`。
- 計算：
  - `let abs_x = input_sample.abs();`
  - 若 `abs_x > prev_env` (處於起音上升階段)：
    - `env = attack_coeff * prev_env + (1.0 - attack_coeff) * abs_x`
  - 否則 (處於釋音衰減階段)：
    - `env = release_coeff * prev_env + (1.0 - release_coeff) * abs_x`
  - 回傳 `Some(env)`。

## 4. 確定性黃金值測試要求
- 起音測試：`input = -2.0 (|x|=2.0), prev = 0.0, attack = 0.8, release = 0.2`
  - `0.8 * 0.0 + 0.2 * 2.0 = 0.4`
- 釋音測試：`input = 0.0, prev = 1.0, attack = 0.8, release = 0.9`
  - `0.9 * 1.0 + 0.1 * 0.0 = 0.9`
- 負包絡值或無效係數拒絕。

```golden
(dsp_envelope_follower_step(-2.0, 0.0, 0.8, 0.2).unwrap() - 0.4).abs() < 1e-12
(dsp_envelope_follower_step(0.0, 1.0, 0.8, 0.9).unwrap() - 0.9).abs() < 1e-12
dsp_envelope_follower_step(1.0, -0.1, 0.8, 0.2).is_none()
dsp_envelope_follower_step(1.0, 0.5, 1.2, 0.2).is_none()
dsp_envelope_follower_step(f64::NAN, 0.5, 0.8, 0.2).is_none()
```
