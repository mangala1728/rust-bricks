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

### 5. 金融終端與智慧卡安全防禦（Financial POS, ATM & Smartcard Security）
金融刷卡機、ATM 提款機、PC/SC 讀卡機韌體極度重視防竄改、防時序攻擊與安全合規：
- **ISO 7816-4 APDU 指令協定封裝與狀態碼判讀**：
  - Command APDU（CLA/INS/P1/P2/Lc/Data/Le）二進位打包與解析、Response APDU 狀態碼（SW1/SW2，如 `0x9000` 成功、`0x6A82` 檔案不存在、`0x63Cx` 密碼剩餘次數）判定。
- **ISO 9564-1 Format 0 PIN Block 密碼塊構造**：
  - 金融鍵盤 (PINPad) 加密前處理：將明文 PIN 與主帳號 (PAN) 進行標準補位與 XOR 混淆，防止明文密碼殘留。
- **EMV 晶片卡 BER-TLV (Tag-Length-Value) 零配置解析**：
  - 信用卡晶片、身分證晶片金融交易資料解碼（單/多位元組 Tag 與長度解析）。
- **Luhn 演算法（Mod-10 卡號防打錯校驗）**：
  - 信用卡、提款卡、IMEI 碼標準同位校驗。
- **記憶體敏感資料安全清零（Zeroize Memory Sanitization）**：
  - 處理完私鑰或 PIN 後，透過防編譯器最佳化抹除的內存清零技術，杜絕 Heartbleed 式的內存轉儲外洩。

### 6. 車載總線與知名 MCU/SoC SDK（Automotive CAN/CAN-FD & MCU Firmware）
Microchip, STMicroelectronics (STM32), NXP, TI, Nordic (nRF51/nRF52), Peak CAN 官方 C 庫常有記憶體溢出與未定義行為隱患：
- **CAN 2.0A/B 與 CAN FD 訊號封包結構解析**：
  - 11-bit 標準 ID 與 29-bit 擴展 ID 位元拆解、RTR 遠端幀標記判定、CAN FD 數據長度碼 (DLC 0..15) 映射至 0..64 位元組長度表。
- **車載診斷協議 ISO-TP (ISO 15765-2) 單幀 (SF) / 首幀 (FF) 封裝**：
  - 車載 ECU 診斷 UDS (ISO 14229) 傳輸層分包演算法，純陣列處理，零動態記憶體分配。
- **工業與嵌入式通訊校驗 CRC16（Modbus RTU / CCITT）**：
  - 廣泛應用於 8051, STM32, Arduino, 樹莓派 RS485 模組的 Modbus RTU (0xA001 反轉多項式) 與 XMODEM/CCITT 校驗。
- **定時器中斷計數重裝載（Timer Prescaler & Period Calculation）**：
  - 輸入系統時鐘頻率、目標定時微秒/毫秒，輸出最佳預分頻器 (PSC) 與自動重裝載值 (ARR)，消除 C 語言整數溢出計算錯誤。

---

## 三、避坑指引：低價值與禁獵區

1. ❌ **禁止移植巨型黑箱模型**：例如直接手刻 Transformer 推理引擎（應由專用 NPU/ONNX 運行）。
2. ❌ **禁止純包裝系統原生 API**：如果該功能 Android 本身一行 Java 就能搞定且無效能問題（如取得目前時間戳），無需刻意包成 Rust 微積木。
3. ❌ **禁止無界動態記憶體配置**：凡無法在編譯期或呼叫入口確定最大記憶體上界的演算法，必須重構為有界微積木。
