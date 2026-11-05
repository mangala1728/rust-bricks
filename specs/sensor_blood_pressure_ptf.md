# 模組名稱: 基於脈搏傳遞時間 PTT 之無袖帶連續血壓估算器 (Pulse Transit Time Cuffless Blood Pressure Estimator)

## 1. 目標定位
- 目標模組檔: `src/sensor_blood_pressure_ptf.rs`
- 目標測試檔: `tests/test_sensor_blood_pressure_ptf.rs`
- 功能簡述: 智慧手環與健康穿戴裝置基於心電 (ECG) R 峰至光體積變化圖 (PPG) 峰值之脈搏傳遞時間 (PTT, 毫秒)，以個人化校準參數估算收縮壓 (SBP) 與舒張壓 (DBP)。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: Cuffless continuous blood pressure estimation via Pulse Transit Time (PTT)
// Inputs: ptt_ms, a_sbp, b_sbp, a_dbp, b_dbp: f64
// Outputs: Option<(f64, f64)> (sbp, dbp)
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 2
/// @domain: ptt_ms in [100.0..=500.0], a_sbp in [-1.5..=-0.01], b_sbp in [120.0..=350.0], a_dbp in [-1.5..=-0.01], b_dbp in [80.0..=250.0]
pub fn sensor_estimate_bp_from_ptt(
    ptt_ms: f64,
    a_sbp: f64,
    b_sbp: f64,
    a_dbp: f64,
    b_dbp: f64,
) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若任何參數非有限浮點數，立即回傳 `None`。
- 若 `ptt_ms < 100.0 || ptt_ms > 500.0`，立即回傳 `None`。
- 若 `a_sbp >= 0.0 || a_dbp >= 0.0` (PTT 與血壓成反比，斜率必須為負)，立即回傳 `None`。
- 若 `b_sbp <= b_dbp` (收縮壓截距必須大於舒張壓截距)，立即回傳 `None`。
- 計算：
  - `sbp = a_sbp * ptt_ms + b_sbp`
  - `dbp = a_dbp * ptt_ms + b_dbp`
- 合約生理合理性檢查：
  - 若 `sbp < 60.0 || sbp > 260.0 || dbp < 40.0 || dbp > 160.0 || sbp <= dbp`，立即回傳 `None`。
- 回傳 `Some((sbp, dbp))`。

## 4. 確定性黃金值測試要求
- 典型健康成人校準值：
  - `ptt_ms = 220.0, a_sbp = -0.5, b_sbp = 230.0` -> `sbp = 120.0`
  - `a_dbp = -0.3, b_dbp = 146.0` -> `dbp = 80.0`
  - 輸出 `(120.0, 80.0)`。
- PTT 超過 500ms 或無效正斜率拒絕。

```golden
let (sbp, dbp) = sensor_estimate_bp_from_ptt(220.0, -0.5, 230.0, -0.3, 146.0).unwrap();
(sbp - 120.0).abs() < 1e-12
(dbp - 80.0).abs() < 1e-12
sensor_estimate_bp_from_ptt(50.0, -0.5, 230.0, -0.3, 146.0).is_none()
sensor_estimate_bp_from_ptt(220.0, 0.5, 230.0, -0.3, 146.0).is_none()
sensor_estimate_bp_from_ptt(220.0, -0.5, 100.0, -0.3, 120.0).is_none()
```
