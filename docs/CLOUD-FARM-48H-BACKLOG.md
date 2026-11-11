# 48小時無人值守雲端農夫宏觀作戰藍圖 (48-Hour Cloud Farmer Grand Backlog)

> 本文件為 `rust-bricks` 專案之「雲端農夫 (Cloud Farmer)」與「本地精工 (Local Crafting)」雙軌並進作戰大綱。
> 涵蓋橫向五大尖端工業領域（廣度）與三大串接管線（深度），提供無間斷之規格餵料，產出符合 09 合約、NASA JPL 60 行規範與 14 大工程鐵律的純 Rust 微積木。

---

## 一、Phase 1 基礎工程（18 模組全部 100% 竣工入庫 🎉）

| 領域代號 | 領域名稱 | 核心工程價值 | 規格書路徑 | 目標模組名 | 狀態 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **DSP-01** | IIR Biquad 陷波濾波器 | 消除工頻 (50Hz/60Hz) 與單頻雜訊，Q值精確控制 | `specs/dsp_biquad_notch.md` | `dsp_biquad_notch` | ✅ 已入庫 (596d9c0) |
| **DSP-02** | IIR Biquad 帶通濾波器 | 固定裙帶增益帶通濾波，聲學訊號特徵提取 | `specs/dsp_biquad_bandpass.md` | `dsp_biquad_bandpass` | ✅ 已入庫 (fd92386) |
| **DSP-03** | 布萊克曼窗函式 (Blackman) | 精確布萊克曼 3 階窗，具備 -58 dB 旁瓣抑制 | `specs/dsp_window_blackman.md` | `dsp_window_blackman` | ✅ 已入庫 (70313aa) |
| **DSP-04** | 漢明窗函式 (Hamming) | 經典語音分析加窗，主瓣集中度優良 | `specs/dsp_window_hamming.md` | `dsp_window_hamming` | ✅ 已入庫 (fa5b9b8) |
| **DSP-05** | 頻譜質心與滾降特徵 | 語音音色分析 (Spectral Centroid & Rolloff) | `specs/dsp_spectral_features.md` | `dsp_spectral_features` | ✅ 已入庫 (92e4f91) |
| **DSP-06** | 梅爾濾波器組權重計算 | 語音辨識前端 Mel-Filterbank 能量窗帶計算 | `specs/dsp_mel_filterbank.md` | `dsp_mel_filterbank` | ✅ 已入庫 (1cbe537) |
| **BUS-01** | CANopen SDO 快速傳輸解析 | 工業自動化 CANopen SDO 下載/上傳協定解析 | `specs/canopen_sdo_handler.md` | `canopen_sdo_handler` | ✅ 已入庫 (d6bc40e) |
| **BUS-02** | Modbus RTU 訊框編解碼 | PLC 串列通訊訊框包裝與 Modbus CRC-16 檢驗 | `specs/modbus_rtu_frame.md` | `modbus_rtu_frame` | ✅ 已入庫 (2398e7d) |
| **BUS-03** | LIN 2.1 增益校驗與同位檢查 | 車身電子 LIN 總線識別碼同位校驗與增強型 Checksum | `src/can_lin_bus.rs` | `can_lin_bus` | ✅ 核心庫原生自帶 |
| **BIO-01** | Mahony 姿態解算四元數單步 | 6-DOF IMU 加速度計與陀螺儀互補姿態更新 | `specs/sensor_mahony_ahrs.md` | `sensor_mahony_ahrs` | ✅ 已入庫 (9f3d6fe) |
| **BIO-02** | Pan-Tompkins ECG 檢波器 | 實時心電圖微分、平方向量累加與 QRS 峰值識別 | `specs/sensor_pan_tompkins.md` | `sensor_pan_tompkins` | ✅ 已入庫 (340155a) |
| **BIO-03** | PPG 血氧飽和度估算器 | 雙波長光體積變化描記圖 AC/DC 比值轉 SpO2 | `specs/sensor_ppg_spo2.md` | `sensor_ppg_spo2` | ✅ 已入庫 (cba79c8) |
| **NUM-01** | 經典四階龍格-庫塔 (RK4) 單步 | 非線性常微分方程 (ODE) 數值積分求解器 | `specs/math_rk4_ode.md` | `math_rk4_ode` | ✅ 已入庫 (5c5a68b) |
| **NUM-02** | 一維黃金分割極值搜尋 | 單峰函式極小值快速數值收斂 | `specs/math_golden_section.md` | `math_golden_section` | ✅ 已入庫 (3132e46) |
| **NUM-03** | 單調鏈二維凸包交叉積掃描 | 計算幾何 Monotone Chain 凸包外輪廓計算 | `specs/geo_convex_hull_2d.md` | `geo_convex_hull_2d` | ✅ 已入庫 (5c02045) |
| **FIN-01** | Black-Scholes 歐式期權定價 | 金融工程 Black-Scholes-Merton 解析解與 Delta | `specs/fin_black_scholes.md` | `fin_black_scholes` | ✅ 已入庫 (417e9a5) |
| **FIN-02** | RiskMetrics EWMA 波動率步進 | 動態指數加權移動平均常態化風險度量 | `specs/fin_ewma_volatility.md` | `fin_ewma_volatility` | ✅ 已入庫 (9f259c9) |
| **FIN-03** | 索提諾比率與下行半方差 | 不對稱風險調整後收益指標計算 | `specs/fin_sortino_ratio.md` | `fin_sortino_ratio` | ✅ 已入庫 (0d04b84) |

---

## 二、Phase 2 拓展工程（22 模組全部 100% 竣工入庫 🎉）

| 領域代號 | 領域名稱 | 核心工程價值 | 規格書路徑 | 目標模組名 | 狀態 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **DSP-07** | Goertzel 單頻率 DFT 檢波 | DTMF 雙音多頻電話撥號與特定頻率微秒級偵測 | `specs/dsp_goertzel.md` | `dsp_goertzel` | ✅ 已入庫 (756c426) |
| **DSP-08** | 有理分數線性插值重採樣 | 聲學音訊取樣率轉換 (如 44.1kHz 轉 48kHz) | `specs/dsp_resample_linear.md` | `dsp_resample_linear` | ✅ 已入庫 (d4aad6e) |
| **DSP-09** | 峰值檢波與包絡線跟隨器 | 音量壓縮器、AGC 與語音包絡提取 (Attack/Release) | `specs/dsp_envelope_peak_hold.md` | `dsp_envelope_peak_hold` | ✅ 已入庫 (ce84618) |
| **DSP-10** | 滑動視窗均方根 (Moving RMS) | 交流能量、真有效值 True-RMS 實時功率量測 | `specs/dsp_moving_rms.md` | `dsp_moving_rms` | ✅ 已入庫 (f59a163) |
| **DSP-11** | 回授與前饋梳狀濾波器 | Schroeder 經典混響與物理建模音色諧波延遲線 | `specs/dsp_comb_filter.md` | `dsp_comb_filter` | ✅ 已入庫 (e42aae5) |
| **BUS-04** | SAE J1939 PGN 解包器 | 重型商用車、工程機械 CAN J1939 參數群解包 | `specs/j1939_pgn_parser.md` | `j1939_pgn_parser` | ✅ 已入庫 (1cbab94) |
| **BUS-05** | NMEA 0183 GPRMC GPS 解析 | 船載與車載 GPS 語句校驗和驗證與經緯度解析 | `specs/nmea0183_gprmc.md` | `nmea0183_gprmc` | ✅ 已入庫 (811e206) |
| **BUS-06** | AUTOSAR DLT 診斷日誌標頭 | 車用控制器標準診斷記錄與追蹤協定解包 | `specs/dlt_protocol_header.md` | `dlt_protocol_header` | ✅ 已入庫 (ce84618) |
| **BIO-04** | 心率變異度 (HRV) 時域指標 | 穿戴健康監測 SDNN, RMSSD, pNN50 自律神經指標 | `specs/sensor_hrv_time_domain.md` | `sensor_hrv_time_domain` | ✅ 已入庫 (a01cc1a) |
| **BIO-05** | 一維自適應卡爾曼濾波器 | 氣壓高度計與 GPS 高度即時最優估計 | `specs/sensor_kalman_1d.md` | `sensor_kalman_1d` | ✅ 已入庫 (58d3c7a) |
| **BIO-06** | 三軸加速度計動態遲滯計步 | 穿戴手環運動步伐即時辨識與雙門檻遲滯防抖 | `specs/sensor_step_counter.md` | `sensor_step_counter` | ✅ 已入庫 (ce84618) |
| **BIO-07** | 訊號模長面積 (SMA) 活動分類 | 穿戴活動量估算、久坐/輕度/劇烈運動三級分類 | `specs/sensor_activity_intensity.md` | `sensor_activity_intensity` | ✅ 已入庫 (f59a163) |
| **NUM-04** | 辛普森複合 1/3 數值積分 | 物理模擬與高精度平滑曲線數值面積計算 | `specs/math_simpson_integral.md` | `math_simpson_integral` | ✅ 已入庫 (305f7e6) |
| **NUM-05** | 射線交叉法任意多邊形內外判別 | 地理資訊圍欄 (Geofencing) 與點在多邊形判定 | `specs/geo_point_in_polygon_ray.md` | `geo_point_in_polygon_ray` | ✅ 已入庫 (01a147a) |
| **NUM-06** | 一維牛頓-拉夫森求根演算法 | 隱式方程、物理平衡態非線性方程快速求根 | `specs/math_newton_raphson_1d.md` | `math_newton_raphson_1d` | ✅ 已入庫 (dec31b2) |
| **NUM-07** | 柴比雪夫多項式與 Clenshaw | 最佳一致逼近多項式與數值穩定求和 | `specs/math_chebyshev_poly.md` | `math_chebyshev_poly` | ✅ 已入庫 (f59a163) |
| **GEO-01** | Geohash 二進位位元交錯編解碼 | 地理空間前綴聚集索引與區域範圍搜尋 | `specs/geo_geohash.md` | `geo_geohash` | ✅ 已入庫 (35e287b) |
| **GEO-02** | 球面大圓航法目標點推算 (Direct) | 給定起點、方位角與距離，求目標經緯度 | `specs/geo_haversine_destination.md` | `geo_haversine_destination` | ✅ 已入庫 (e42aae5) |
| **FIN-04** | 歷史模擬法 VaR 與 CVaR | 尾端風險評估與預期損失 Expected Shortfall | `specs/fin_var_historical.md` | `fin_var_historical` | ✅ 已入庫 (35e287b) |
| **FIN-05** | 債券麥考利久期與修正久期 | 利率敏感度、固定收益債券資產負債配對 | `specs/fin_macaulay_duration.md` | `fin_macaulay_duration` | ✅ 已入庫 (95afdbc) |
| **FIN-06** | 凱利準則最佳資金管理分配 | 投資組合最優部位配置與分數凱利 (Fractional Kelly) | `specs/fin_kelly_criterion.md` | `fin_kelly_criterion` | ✅ 已入庫 (eab858c) |
| **FIN-07** | 年化夏普比率與資訊比率 | 風險調整後超額收益與主動投資追蹤指標 | `specs/fin_sharpe_ratio.md` | `fin_sharpe_ratio` | ✅ 已入庫 (f59a163) |

---

## 三、Phase 3 下一波 48 小時深耕計畫（待餵料規格候選庫）

| 領域代號 | 領域名稱 | 核心工程價值 | 預計規格書路徑 | 預計模組名 |
| :--- | :--- | :--- | :--- | :--- |
| **DSP-12** | DTMF 雙音多頻電話解碼狀態機 | 結合 Goertzel 的標準電話鍵盤 (0-9,*,#,A-D) 8 頻率解碼 | `specs/dsp_dtmf_decoder.md` | `dsp_dtmf_decoder` |
| **DSP-13** | 一階直流阻斷濾波器 (DC Blocker) | 消除音訊與生物感測器訊號中的漂移直流偏置 | `specs/dsp_dc_blocker.md` | `dsp_dc_blocker` |
| **DSP-14** | 粉紅雜訊生成濾波器 (Voss-McCartney) | 聲學測試 1/f 功率頻譜密度均勻分佈雜訊產生 | `specs/dsp_pink_noise.md` | `dsp_pink_noise` |
| **BUS-07** | CAN-FD CRC-17 / CRC-21 校驗 | 支援 ISO CAN-FD 擴展訊框高可靠度 CRC 校驗 | `specs/canfd_crc.md` | `canfd_crc` |
| **BUS-08** | NMEA 2000 快速封包傳輸協議 | 航海與戶外 CAN 網路大資料長度訊框重組 | `specs/nmea2000_fast_packet.md` | `nmea2000_fast_packet` |
| **BIO-08** | 基於 SVM 模長之跌倒檢測特徵 | 穿戴照護老年人突發跌倒撞擊即時警報檢測 | `specs/sensor_fall_detection.md` | `sensor_fall_detection` |
| **BIO-09** | ECG 導出呼吸率 (EDR) 計算 | 利用 QRS 振幅調變估算每分鐘呼吸次數 | `specs/sensor_ecg_derived_resp.md` | `sensor_ecg_derived_resp` |
| **GEO-03** | Vincenty 逆算大圓測地距離與航向 | 橢球體高精度距離與正反方位角解析 | `specs/geo_vincenty_inverse.md` | `geo_vincenty_inverse` |
| **NUM-08** | 2x2 對稱矩陣奇異值分解 (SVD) | 即時慣性橢球主軸提取與幾何主成分分析 | `specs/math_svd_2x2.md` | `math_svd_2x2` |
| **NUM-09** | 三次自然樣條插值單步求值 | 軌跡規劃與平滑路徑生成 | `specs/math_cubic_spline.md` | `math_cubic_spline` |
| **FIN-08** | GARCH(1,1) 動態波動率步進更新 | 金融時序波動率聚類效應即時預測 | `specs/fin_garch11.md` | `fin_garch11` |
| **FIN-09** | 二分法反推 Black-Scholes 隱含波動率 | 衍生性金融商品市場隱含波動率 (IV) 快速收斂 | `specs/fin_implied_volatility.md` | `fin_implied_volatility` |
| **FIN-10** | 卡瑪比率與最大回撤評估 (Calmar) | 避險基金與CTA量化策略回撤風險調整收益 | `specs/fin_calmar_ratio.md` | `fin_calmar_ratio` |

---

## 四、縱向深度管線設計 (Pipeline Depth)

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
[DSP-10: Moving RMS 能量偵測 / 語音活動檢測 VAD]
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
[ECG Dry Electrode Lead-I]       ──▶ [BIO-02: Pan-Tompkins QRS 波峰與心率檢測] ──▶ [BIO-04: HRV 自律神經指標]
[3-Axis Accel + Gyro IMU]        ──▶ [BIO-01: Mahony AHRS 四元數實時姿態解算]
[3-Axis Accel]                   ──▶ [BIO-06: Step Counter 計步器] ──▶ [BIO-07: SMA 活動強度分類]
```

### 管線三：工業與車用實時總線網關 (Fieldbus Gateway Pipeline)
```
[CANopen SDO 請求訊框] ──▶ [BUS-01: canopen_sdo_handler 零拷貝解析]
[Modbus 儀表讀取]      ──▶ [BUS-02: modbus_rtu_frame CRC-16 檢驗與封裝]
[車門/車窗 LIN 節點]   ──▶ [BUS-03: lin_frame_validator 同位元與 Checksum 驗收]
[商用車 J1939 引擎]    ──▶ [BUS-04: j1939_pgn_parser 29-bit CAN 標識符與 PGN 解析]
[車載控制器日誌]       ──▶ [BUS-06: dlt_protocol_header AUTOSAR DLT 診斷標頭解析]
[車載 GPS 接收機]      ──▶ [BUS-05: nmea0183_gprmc NMEA0183 語句與經緯度解析] ──▶ [GEO-02: 大圓目標點正算]
```

---

## 五、Phase 5 個人硬體公司前瞻工程（Signex + Fyrox + Blender 全閉環）

> 詳細待辦清單與五大支柱微積木規劃，請參閱專屬作戰文件：[`docs/HARDWARE-EDA-DIGITAL-TWIN-BACKLOG.md`](HARDWARE-EDA-DIGITAL-TWIN-BACKLOG.md)。  
> 涵蓋：
> 1. **EDA-RF**：微帶線/帶狀線/共面波導阻抗、S 參數、史密斯圓圖、趨膚深度。
> 2. **EDA-GEO**：DRC 點線距離、差分蛇形繞線、淚滴圓弧、IPC-7351 焊盤尺寸、Gerber/Drill 解析。
> 3. **EDA-THM**：晶片結溫 TJ、散熱過孔熱阻、IPC-2152 電流溫升、2D 穩態熱傳導 FDM。
> 4. **EDA-3D**：2D 板框拉伸 3D Mesh、AABB/OBB 機殼干涉、glTF 頂點串流解析。
> 5. **EDA-DOC**：晶片規格書視覺+OCR 表格萃取、暫存器位元欄位驗證、電氣邊界守衛。

---

## 六、Phase 6 吠陀五分曆、人生事件記錄器與時空數位孿生微積木（Vedic Panchanga & Spatiotemporal Pipeline）

> 依據 `portable/domain-pipe-standards/04_VEDIC_ASTRO_SPATIOTEMPORAL_EXT.md` 之跨倉庫標準規格，為線上/手機/本地版吠陀占星、人生事件記錄器與高精度街景VR星空數位孿生提供純 Rust 09 合約積木：

| 領域代號 | 領域名稱 | 核心工程價值 | 規範與演算法 | 預計模組名 |
| :--- | :--- | :--- | :--- | :--- |
| **ASTRO-01** | 純整數 Tithi 與月相百分比計算 | 純整數毫度計算 30 Tithi (1..30) 與月相百分比 (0..100) | $\Delta = (M + 360000 - S) \pmod{360000} / 12000 + 1$ | `astro_panchanga_tithi` |
| **ASTRO-02** | 27 宿與 4 足 (Nakshatra & Pada) | 月球經度轉角分純整數計算 27 宿與 4 足 (D9 Navamsha) | $M_{\text{min}} = M \times 3 / 50$; $Nak = M_{\text{min}} / 800 + 1$ | `astro_panchanga_nakshatra` | ✅ 已入庫 (雲端農夫收割) |
| **ASTRO-03** | 27 曜與 60 關 (Yoga & Karana) | 日月經度和與半 Tithi 關位純整數計算 | $Yoga = (S_{\text{min}} + M_{\text{min}}) \pmod{21600} / 800 + 1$ | `astro_panchanga_yoga_karana` |
| **ASTRO-04** | 缺日與增日檢測 (Kshaya / Vriddhi) | 比對日出日落時間戳記判定無日出之虧日或跨雙日出之增日 | 比較 `ts_sunrise_curr` 與 `ts_sunrise_next` 之包含性 | `astro_tithi_special_detector` |
| **ASTRO-05** | Vimshottari 運限 32-bit 位元封裝 | 大運/小運/次小運/微運/呼吸運多層階梯式整數壓縮編解碼 | 5 層 4-bit + 12-bit 餘量封裝至 `dasha_packed: u32` | `astro_vimshottari_dasha` |
| **ASTRO-06** | 歲差與恆星/熱帶黃道轉換 | 支援 Lahiri, KP, Raman, Fagan-Bradley, Yukteshwar 歲差角秒轉換 | $\lambda_{\text{sidereal}} = (\lambda_{\text{tropical}} + 360000 - \text{Ayanamsha}) \pmod{360000}$ | `astro_ayanamsha_converter` | ✅ 已入庫 (雲端農夫收割) |
| **ASTRO-07** | 行星留與逆行轉換狀態機 | 根據黃道每日日視速度符號與門檻判定順行/逆行/停滯 | 遲滯雙門檻判定留 (Stationary) 與逆行開始/結束 | `astro_planetary_station` |
| **ASTRO-08** | 局部站心座標系 (ENU) 毫米旋轉 | 浮動原點 (Floating Origin) 消除 VR 與街景 32-bit 浮點抽搐 | 觀測者腳下為 (0,0,0)，毫米整數轉換 | `geo_enu_floating_origin` |
| **ASTRO-09** | 相對論皮秒時間膨脹偏移 | 根據高度差 (重力場) 與載具速度計算愛因斯坦原時膨脹 | $\Delta \tau = \int (1 - gh/c^2 - v^2/2c^2) dt$ (皮秒整數) | `phys_relativistic_drift_ps` |

---

## 七、48小時巡邏與收割作戰指引

1. **雲端農夫運作**：
   - GitHub Actions 透過 `cloud_farmer.yml` 手動觸發或定時排程。
   - 每完成一個規格，自動推送到 `contrib/cloud-farm-<module_name>`。
2. **衝突防禦與零等待收割**：
   - 本地常駐執行 `cargo run -- patrol` 巡檢分支狀態。
   - 執行 `cargo run -- harvest`，收割機以唯讀方式萃取模組與測試，在獨立 target 目錄執行隔離驗證，並自動按字母排序無衝突插入 `src/lib.rs`。
3. **主幹道整合與簽署**：
   - 通過 `cargo run -- guard` -> `cargo run -- catalog` -> `cargo run -- receipt kev-4b PASS` -> `cargo run -- verify-receipt` 後推進 `main`。
