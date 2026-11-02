# 48小時無人值守雲端農夫宏觀作戰藍圖 (48-Hour Cloud Farmer Grand Backlog)

> 本文件為 `rust-bricks` 專案之「雲端農夫 (Cloud Farmer)」與「本地精工 (Local Crafting)」雙軌並進作戰大綱。
> 涵蓋橫向五大尖端工業領域（廣度）與三大串接管線（深度），目標在 48 小時內提供無間斷之規格餵料，產出符合 09 合約、NASA JPL 60 行規範與 14 大鐵律的純 Rust 微積木。

---

## 一、宏觀架構與雙軌分工

| 領域代號 | 領域名稱 | 核心工程價值 | 規格書路徑 | 目標模組名 |
| :--- | :--- | :--- | :--- | :--- |
| **DSP-01** | IIR Biquad 陷波濾波器 | 消除工頻 (50Hz/60Hz) 與單頻雜訊，Q值精確控制 | `specs/dsp_biquad_notch.md` | `dsp_biquad_notch` |
| **DSP-02** | IIR Biquad 帶通濾波器 | 固定裙帶增益帶通濾波，聲學訊號特徵提取 | `specs/dsp_biquad_bandpass.md` | `dsp_biquad_bandpass` |
| **DSP-03** | 布萊克曼窗函式 (Blackman) | 精確布萊克曼 3 階窗，具備 -58 dB 旁瓣抑制 | `specs/dsp_window_blackman.md` | `dsp_window_blackman` |
| **DSP-04** | 漢明窗函式 (Hamming) | 經典語音分析加窗，主瓣集中度優良 | `specs/dsp_window_hamming.md` | `dsp_window_hamming` |
| **DSP-05** | 頻譜質心與滾降特徵 | 語音音色分析 (Spectral Centroid & Rolloff) | `specs/dsp_spectral_features.md` | `dsp_spectral_features` |
| **DSP-06** | 梅爾濾波器組權重計算 | 語音辨識前端 Mel-Filterbank 能量窗帶計算 | `specs/dsp_mel_filterbank.md` | `dsp_mel_filterbank` |
| **BUS-01** | CANopen SDO 快速傳輸解析 | 工業自動化 CANopen SDO 下載/上傳協定解析 | `specs/canopen_sdo_handler.md` | `canopen_sdo_handler` |
| **BUS-02** | Modbus RTU 訊框編解碼 | PLC 串列通訊訊框包裝與 Modbus CRC-16 檢驗 | `specs/modbus_rtu_frame.md` | `modbus_rtu_frame` |
| **BUS-03** | LIN 2.1 增益校驗與同位檢查 | 車身電子 LIN 總線識別碼同位校驗與增強型 Checksum | `specs/lin_frame_validator.md` | `lin_frame_validator` |
| **BIO-01** | Mahony 姿態解算四元數單步 | 6-DOF IMU 加速度計與陀螺儀互補姿態更新 | `specs/sensor_mahony_ahrs.md` | `sensor_mahony_ahrs` |
| **BIO-02** | Pan-Tompkins ECG 檢波器 | 實時心電圖微分、平方向量累加與 QRS 峰值識別 | `specs/sensor_pan_tompkins.md` | `sensor_pan_tompkins` |
| **BIO-03** | PPG 血氧飽和度估算器 | 雙波長光體積變化描記圖 AC/DC 比值轉 SpO2 | `specs/sensor_ppg_spo2.md` | `sensor_ppg_spo2` |
| **NUM-01** | 經典四階龍格-庫塔 (RK4) 單步 | 非線性常微分方程 (ODE) 數值積分求解器 | `specs/math_rk4_ode.md` | `math_rk4_ode` |
| **NUM-02** | 一維黃金分割極值搜尋 | 單峰函式極小值快速數值收斂 | `specs/math_golden_section.md` | `math_golden_section` |
| **NUM-03** | 單調鏈二維凸包交叉積掃描 | 計算幾何 Monotone Chain 凸包外輪廓計算 | `specs/geo_convex_hull_2d.md` | `geo_convex_hull_2d` |
| **FIN-01** | Black-Scholes 歐式期權定價 | 金融工程 Black-Scholes-Merton 解析解與 Delta | `specs/fin_black_scholes.md` | `fin_black_scholes` |
| **FIN-02** | RiskMetrics EWMA 波動率步進 | 動態指數加權移動平均常態化風險度量 | `specs/fin_ewma_volatility.md` | `fin_ewma_volatility` |
| **FIN-03** | 索提諾比率與下行半方差 | 不對稱風險調整後收益指標計算 | `specs/fin_sortino_ratio.md` | `fin_sortino_ratio` |

---

## 二、縱向深度管線設計 (Pipeline Depth)

### 管線一：邊緣語音特徵提取全鏈條 (Acoustic Feature Extraction Pipeline)
```
[Raw Audio PCM]
       │
       ▼
[DSP-03 / 04: Blackman / Hamming Window 加窗]
       │
       ▼
[DSP-01: Biquad Notch 濾除 50/60Hz 工頻干擾]
       │
       ▼
[DSP-05: Spectral Centroid & Rolloff 頻譜質心提取]
       │
       ▼
[DSP-06: Mel-Filterbank 三角窗能量特徵向量]
```

### 管線二：智慧手環穿戴生醫監測管線 (Biomedical Wearable Pipeline)
```
[PPG Dual-Wavelength Photodiode] ──▶ [BIO-03: SpO2 Ratio-of-Ratios 血氧估算]
[ECG Dry Electrode Lead-I]       ──▶ [BIO-02: Pan-Tompkins QRS 波峰與心率檢測]
[6-DOF IMU Accel + Gyro]        ──▶ [BIO-01: Mahony AHRS 四元數實時姿態解算]
```

### 管線三：工業與車用實時總線網關 (Fieldbus Gateway Pipeline)
```
[CANopen SDO 請求訊框] ──▶ [BUS-01: canopen_sdo_handler 零拷貝解析]
[Modbus 儀表讀取]      ──▶ [BUS-02: modbus_rtu_frame CRC-16 檢驗與封裝]
[車門/車窗 LIN 節點]   ──▶ [BUS-03: lin_frame_validator 同位元與 Checksum 驗收]
```

---

## 三、48小時巡邏與收割作戰指引

1. **雲端農夫運作**：
   - GitHub Actions 透過 `cloud_farmer.yml` 手動觸發或定時排程。
   - 每完成一個規格，自動推送到 `contrib/cloud-farm-<module_name>`。
2. **衝突防禦與零等待收割**：
   - 本地常駐執行 `cargo run -- patrol` 巡檢分支狀態。
   - 執行 `cargo run -- harvest`，收割機以唯讀方式萃取模組與測試，在獨立 target 目錄執行隔離驗證，並自動按字母排序無衝突插入 `src/lib.rs`。
3. **主幹道整合與簽署**：
   - 通過 `cargo run -- guard` -> `cargo run -- catalog` -> `cargo run -- receipt kev-4b PASS` -> `cargo run -- verify-receipt` 後推進 `main`。
