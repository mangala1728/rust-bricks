# 模組名稱: 皮膚電反應 GSR 導電度與 Tonic/Phasic 分解演算法 (GSR Conductance & Tonic/Phasic Decomposition)

## 1. 目標定位
- 目標模組檔: `src/sensor_galvanic_skin_response.rs`
- 目標測試檔: `tests/test_sensor_galvanic_skin_response.rs`
- 功能簡述: 穿戴式壓力與情緒感測器之皮膚電活動 (EDA / GSR) 訊號轉換純函式。包含分壓電路 ADC 電壓轉皮膚導電度微西門子 ($\mu S$)，以及緩變張力成份 (Tonic) 與交感神經突發脈衝相位成份 (Phasic) 線性分離。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Electrodermal Activity (GSR/EDA) voltage conversion and tonic/phasic filter
// Inputs: adc_volt, v_ref, r_fixed, conductance_us, prev_tonic, alpha: f64
// Outputs: Option<f64> or Option<(f64, f64)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: adc_volt in (0.01..v_ref), v_ref in [1.0..=5.0], r_fixed in [1e4..=1e6]
pub fn sensor_gsr_adc_to_microsiemens(
    adc_volt: f64,
    v_ref: f64,
    r_fixed: f64,
) -> Option<f64>;

/// @outputs: 2
/// @domain: conductance_us in [0.01..=100.0], prev_tonic in [0.01..=100.0], alpha in [0.0001..=0.9999]
pub fn sensor_eda_tonic_phasic_step(
    conductance_us: f64,
    prev_tonic: f64,
    alpha: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- `sensor_gsr_adc_to_microsiemens`:
  - 若 `adc_volt <= 0.001 || adc_volt >= v_ref || v_ref < 1.0 || v_ref > 5.0 || r_fixed < 1e4 || r_fixed > 1e6`，立即回傳 `None`。
  - 皮膚電阻：$R_{skin} = R_{fixed} \times (V_{ref} / V_{adc} - 1.0)$。
  - 導電度 $\mu S$：$G_{\mu S} = 10^6 / R_{skin}$。
  - 回傳 `Some(G_us)`。
- `sensor_eda_tonic_phasic_step`:
  - 若 `conductance_us < 0.01 || conductance_us > 100.0 || prev_tonic < 0.01 || prev_tonic > 100.0 || alpha <= 0.0 || alpha >= 1.0`，立即回傳 `None`。
  - `next_tonic = alpha * prev_tonic + (1.0 - alpha) * conductance_us`
  - `phasic = conductance_us - next_tonic`
  - 回傳 `Some((next_tonic, phasic))`。

## 4. 確定性黃金值測試要求
- 電路轉換：
  - `v_ref = 3.3, r_fixed = 100_000.0 (100kΩ)`
  - 當 `adc_volt = 1.65`（恰為 $V_{ref}/2$），則 $R_{skin} = 100,000\Omega$。
  - $G = 10^6 / 100,000 = 10.0\ \mu S$。
- Tonic/Phasic 分解：
  - `conductance = 12.0, prev_tonic = 10.0, alpha = 0.9`
  - `next_tonic = 0.9 * 10.0 + 0.1 * 12.0 = 10.2`
  - `phasic = 12.0 - 10.2 = 1.8`

```golden
let g = sensor_gsr_adc_to_microsiemens(1.65, 3.3, 100_000.0).unwrap();
(g - 10.0).abs() < 1e-6
let (tonic, phasic) = sensor_eda_tonic_phasic_step(12.0, 10.0, 0.9).unwrap();
(tonic - 10.2).abs() < 1e-12
(phasic - 1.8).abs() < 1e-12
sensor_gsr_adc_to_microsiemens(3.3, 3.3, 100_000.0).is_none()
sensor_eda_tonic_phasic_step(12.0, 10.0, 1.0).is_none()
```
