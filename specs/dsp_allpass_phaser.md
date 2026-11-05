# 模組名稱: 一階全通濾波器與混響擴散步進 (First-Order Allpass Filter)

## 1. 目標定位
- 目標模組檔: `src/dsp_allpass_phaser.rs`
- 目標測試檔: `tests/test_dsp_allpass_phaser.rs`
- 功能簡述: 在 Schroeder 經典數位混響擴散器（Diffusion Core）與吉他 Phaser 移相效果器中，提供全頻帶增益恆等於 1.0、僅改變頻率相位之 Direct Form II 一階全通濾波器。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: First-order canonical allpass filter step
// Inputs: input: f64, state: &mut f64, gain: f64
// Outputs: Option<f64>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: |gain| < 1.0, input is finite, *state is finite
pub fn dsp_allpass_first_order_step(
    input: f64,
    state: &mut f64,
    gain: f64,
) -> Option<f64>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `!input.is_finite() || !state.is_finite() || !gain.is_finite() || gain <= -1.0 || gain >= 1.0`，立即回傳 `None`（極點嚴格在單位圓內以保證因果穩定）。
- Direct Form II 遞推運算：
  $$v = \text{input} + \text{gain} \times (*\text{state})$$
  $$y = -\text{gain} \times v + (*\text{state})$$
  $$*\text{state} = v$$
  回傳 `Some(y)`。

## 4. 確定性黃金值測試要求
- 脈衝響應解析驗證（$\text{gain} = 0.5$, 初始 state = 0.0）：
  - 輸入 $1.0$ $\rightarrow$ 輸出 $-0.5$, state 變為 $1.0$
  - 下一步輸入 $0.0$ $\rightarrow$ 輸出 $0.75$, state 變為 $0.5$
  - 下一步輸入 $0.0$ $\rightarrow$ 輸出 $0.375$, state 變為 $0.25$
- 非法邊界：$\text{gain} = 1.0$ 或 $-1.0$ 回傳 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_dsp_allpass_phaser.rs）
```golden
let mut st = 0.0;
(dsp_allpass_first_order_step(1.0, &mut st, 0.5).unwrap() - (-0.5)).abs() < 1e-10
(st - 1.0).abs() < 1e-10
(dsp_allpass_first_order_step(0.0, &mut st, 0.5).unwrap() - 0.75).abs() < 1e-10
(dsp_allpass_first_order_step(0.0, &mut st, 0.5).unwrap() - 0.375).abs() < 1e-10
dsp_allpass_first_order_step(1.0, &mut st, 1.0).is_none()
dsp_allpass_first_order_step(1.0, &mut st, -1.0).is_none()
dsp_allpass_first_order_step(f64::NAN, &mut st, 0.5).is_none()
```
