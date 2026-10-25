# 獵物探勘指南：邊緣高價值演算法識別與評估矩陣（Hunting High-Value Bricks）

> **「不造無人問津的象牙塔大庫，專獵每天在幾億台手機與邊緣裝置上高頻觸發、省電省記憶體最有感的微積木。」**  
> 本指南為 `rust-bricks` 跨語言逆向轉譯團隊評估「什麼演算法值得被移植為 Rust 微積木」的戰略決策標準。

---

## 一、獵物價值評估矩陣（The 4-Factor Valuation Matrix）

在挑選 Python / C++ / Java / TS 演算法進行轉譯前，必須以四維矩陣評分（滿分 100 分，$\ge 75$ 分列為特級獵物）：

| 維度 | 評估標準 | 權重 | 典型正向訊號 | 典型負向訊號（低價值） |
|---|---|---|---|---|
| **$V_1$ 觸發高頻度** | 是否在邊緣裝置每秒或每幀高頻運算？ | 30% | 60FPS 觸控取樣、音訊 PCM 串流、相機預覽幀 | 一天只跑一次的報表彙整、離線大數據 Batch |
| **$V_2$ GC 與記憶體痛點** | 原語言在該處是否頻繁引發 GC 停頓或記憶體複製？ | 30% | 音訊 RingBuffer、相機 NV21 轉灰階、高頻物件分配 | 純冷啟動靜態設定檔解析 |
| **$V_3$ 零依賴純淨度** | 能否剝離龐大框架，以純標準庫與有限記憶體完成？ | 20% | 數學公式、位元運算、查表法、狀態機 | 必須連線雲端或高度綁定系統私有 API |
| **$V_4$ 省電與低功耗** | 轉為 Rust 純記憶體後是否能顯著延長手機待機？ | 20% | 消除 JNI 跨邊界拷貝、非線性軟削峰、感測器去噪 | 伺服器端無限電源的重型矩陣分解 |

---

## 二、重點獵物獵場清單（High-Value Target Categories）

### 1. 手機微控制與觸控交互（Touch & Micro-interaction）
- **觸控防抖與滑動速度估算（Touch Debounce & Velocity Tracker）**：
  - 物理按鍵有機械彈跳時間過濾（10ms~50ms），手機電容螢幕則有高頻微小抖動。
  - 手刻純浮點數滑動視窗指數衰減平滑（EMA）與瞬時速度向量計算，讓清單滑動與小遊戲跟手度提升 300%。
- **壓力觸控強度判定（Force / Pressure Touch Thresholds）**：
  - 將 Android `MotionEvent.getPressure()` 原始連續浮點數分級為多階觸覺反饋（Light / Medium / Heavy），具備遲滯曲線（Hysteresis）防止臨界點反覆觸發。
- **手勢判讀狀態機（Gesture State Machine）**：
  - 單擊、雙擊超時判定、長按門檻、滑動防誤觸之純狀態機。

### 2. 底層硬體感測器讀取與微控制（Hardware Sensors & Actuation）
- **霍爾效應感測器與磁力翻蓋偵測（Hall Effect & Magnetic Cover）**：
  - 利用磁場強度突變判定皮套翻蓋開合，手刻磁力異常過濾演算法（排除耳機或桌子金屬干擾）。
- **指南針八字校準與地磁補償（Compass Calibration & Tilt Compensation）**：
  - 結合加速度計倾斜角，將 3 軸磁力計讀數投影至水平面，消除手機傾斜帶來的指南針誤差。
- **閃光燈頻閃與 PWM 調光時序（Flashlight Strobe / PWM Timing）**：
  - 相機補光、求救訊號（SOS 摩斯密碼時序編碼）微積木。

### 3. 端系統狀態與網路遙測（System State & Network Telemetry）
- **網路延遲與抖動平滑估計（Wi-Fi vs Cellular RTT Jitter Estimator）**：
  - 手機在 Wi-Fi 與 4G/5G 切換時，利用滑動標準差與卡爾曼濾波估算當前網路往返時間（RTT），主動調整串流緩衝區大小。
- **極速二進位 IP / CIDR 網段比對（Fast Subnet Matcher）**：
  - 零 Heap 配置、零 Regex，直接以 `u32` / `u128` 位元遮罩判斷 IP 是否落在私有網段或特定安全網段。

### 4. 進階影像特徵萃取（Advanced Edge Vision from Python/OpenCV）
Python OpenCV 代碼龐大動輒數十 MB，但其底層演算法核心極小：
- **FAST 角點偵測（Features from Accelerated Segment Test）**：
  - 圓周 16 畫素連續閾值比較，純位元操作快速找出影像特徵角點。
- **Sobel 邊緣梯度計算（Edge Gradients）**：
  - $3 \times 3$ 卷積核純整數位移計算水平與垂直梯度強度。
- **Otsu 自適應雙峰二值化（Otsu Thresholding）**：
  - 基於 256-bin 直方圖尋找類間變異數最大值，自動計算最佳黑白門檻。
- **積分圖（Integral Image）**：
  - $O(1)$ 時間求任意矩形區域畫素和，為人臉/條碼特徵比對基石。

---

## 三、避坑指引：低價值與禁獵區

1. ❌ **禁止移植巨型黑箱模型**：例如直接手刻 Transformer 推理引擎（應由專用 NPU/ONNX 運行）。
2. ❌ **禁止純包裝系統原生 API**：如果該功能 Android 本身一行 Java 就能搞定且無效能問題（如取得目前時間戳），無需刻意包成 Rust 微積木。
3. ❌ **禁止無界動態記憶體配置**：凡無法在編譯期或呼叫入口確定最大記憶體上界的演算法，必須重構為有界微積木。
