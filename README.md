# rust-bricks — 純淨零依賴 Rust 函式積木庫（09 合約規範）

[![Zero Dependencies](https://img.shields.io/badge/dependencies-0-brightgreen.svg)](#)
[![09 Contract Compliant](https://img.shields.io/badge/09--contract-100%25-blue.svg)](#)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#)

> **「不求快開發，一切以狀態可窮舉、邊界可靜態驗證、出事 3 分鐘內定位為最高原則。」**  
> 專為 Antigravity 高放權 AI 代理工作流、Android 跨端 (Kotlin JNI / NDK) 與邊緣節點打造的純 Rust 高性能函式積木庫。

---

## 一、核心設計哲學（Pillars）

1. **100% 零第三方依賴（Zero 3rd-party Dependencies）**
   - 拒絕 `cargo add` 隨機拉入數十個黑箱傳染性依賴（Transitive Dependencies）。
   - 僅依賴 Rust `core` / `alloc` / `std`，可秒級交叉編譯至 Windows、Linux、macOS、Android (`aarch64-linux-android`) 與 WebAssembly (`wasm32`)。
2. **全面落實 09 號合約邊界規範（Contract Boundary）**
   - 每個原始檔開頭必附 `## Map` 接線圖（Role / Inputs / Outputs / Invariants）。
   - 每個函式必標 `// @outputs: N`（輸出狀態基數上界）與 `// @domain: param [min..max]`。
   - **函式入口剛性實作邊界守衛**：越界立即收斂或回傳安全錯誤。
   - **零全域可變狀態（`R09-2`，Zero `static mut`）**、**禁止遞迴（`R09-7`，No Recursion）**、**有界迴圈（`R09-8`）**。
3. **Python 語意對照與資產複利（Rosetta Stone）**
   - 針對 Python 標準庫或常用演算法（如時間差、二分法、滑動視窗、音訊能量計算），提煉其純粹數學與演算法核心，以 Rust 嚴密語法重現，成為跨專案長期技術複利。
4. **指揮官大局思維：廣度泛化與深度泛化雙軸演化（Dual-Axis Evolution）**
   - **橫向廣度泛化**：開闢新領域橋頭堡（手機、音訊、視覺、車載 CAN、金融卡、3D VR 物理、溫控 PID、Agent Harness）。
   - **縱向深度泛化**：深耕細作打穿工業級護城河（如 CAN DBC 跨字節信號解包、ISO 8583 銀行交易點陣圖）。
   - 詳見戰略文件：[`docs/DUAL-AXIS-EVOLUTION-STRATEGY.md`](docs/DUAL-AXIS-EVOLUTION-STRATEGY.md)。

---

## 二、模組方塊圖（Module Map）

| 模組 | 職責 | 典型應用場景 |
|---|---|---|
| [`agent_terminal`](src/agent_terminal.rs) | 終端 ANSI 色彩碼脫色淨化、非遞迴極速 Glob 檔案搜尋、PTY 退格鍵與行編輯模擬 | AI 代理馬具 (Agent Harness)、虛擬終端路由、Shell 日誌純文字清洗 |
| [`ai_tensor_train`](src/ai_tensor_train.rs) | 數值穩定 Softmax (防極端溢位)、LayerNorm/RMSNorm (LLaMA/Mistral 核心)、RoPE 旋轉位置編碼、AdamW 步進、融合交叉熵梯度、GELU 激活 | 零依賴 AI 模型訓練迴圈 (nanoGPT/llm.c/micrograd 架構)、邊緣 LLM 推論引擎、神經網路純 Rust 重現 |
| [`math_time`](src/math_time.rs) | 高精度數值夾止、滑動均線、儒略日曆法轉換、占星大限 (Dasha) 比例運算、XorShift64 | 股市量化、時間序列平滑、極速亂數生成 |
| [`audio_vad`](src/audio_vad.rs) | PCM16 音訊取樣點能量 (RMS) 計算、靜音幀判定、CPS 字幕字速審查 | 語音活動檢測 (VAD)、即時翻譯 App、多模態字幕對齊 |
| [`audio_dsp`](src/audio_dsp.rs) | 聲學功率、dBFS 能量、零交越率、Butterworth 高通濾波、語音預加重、軟削峰防破音 | 麥克風音訊前處理、即時降噪前置濾波、音訊串流 |
| [`audio_mfcc_pitch`](src/audio_mfcc_pitch.rs) | Hz 與 Mel 刻度互轉、三角梅爾濾波器組權重、DCT-II 倒頻譜係數 (MFCC)、自相關基頻 (Pitch / F0) 估算、YIN 演算法平方差分與 CMND 歸一化、DRC 軟拐點動態壓縮、Q15 預加重、頻譜平坦度 (SFM) | 語音辨識聲學特徵提取 (ASR)、說話人識別、音高偵測、動態範圍壓縮增益控制、濁音/噪聲分類 |
| [`audio_wav`](src/audio_wav.rs) | 零依賴 16-bit PCM WAV 檔頭解析與二進位編碼 | 音訊錄音檔存檔、跨平台 WAV 串流解析 |
| [`ring_buffer`](src/ring_buffer.rs) | 純記憶體無鎖固定容量 PCM16 環形緩衝區、批次讀寫、溢出守衛 | 手機麥克風串流 (Oboe/AudioRecord)、感測器事件佇列、消滅 GC 停頓 |
| [`sensor_motion`](src/sensor_motion.rs) | 陀螺儀/加速度計互補濾波姿態融合、1D 卡爾曼濾波、Haversine GPS 大圓距離與航向角 | 手機小遊戲體感控制、防手震、GPS 經緯度去噪與定位 |
| [`gnss_nmea_geodesy`](src/gnss_nmea_geodesy.rs) | NMEA 0183 語句 XOR 異或校驗和、經緯度度分轉十進制度數、WGS84 經緯高 (LLH) 與地心地固直角坐標 (ECEF) 雙向變換 (Bowring 閉式解)、ECEF 轉站心東-北-天 (ENU) 局域坐標、衛星仰角/方位角解算、幾何精度因子 (PDOP/HDOP/VDOP)、大圓大地線前向方位角、偽距鐘差校正、RTK 載波相位周跳檢測 | 車載/無人機高精度 RTK 定位、GNSS 接收機基頻協議解析、地理測繪坐標轉換、衛星星曆可見性計算 |
| [`spatial_physics`](src/spatial_physics.rs) | 四元數哈密頓乘法、羅德里格向量旋轉、VR 射線拾取求交、彈簧阻尼系統步進 | 3D VR/AR 頭顯與手把姿態追蹤、虛擬手把拉扯動畫、空間射線選取 |
| [`sensor_hardware`](src/sensor_hardware.rs) | 霍爾磁力翻蓋開合偵測、3軸磁力計傾斜補償指南針、閃光燈 PWM 頻閃與 SOS 時序 | 手機磁力皮套偵測、手機傾斜指南針定向、相機補光/求救信號 |
| [`net_telemetry`](src/net_telemetry.rs) | RTT 網路延遲與抖動平滑、零配置 IPv4/IPv6 CIDR 子網比對、私有網段判定 | Wi-Fi/LTE 串流動態緩衝、局域網設備識別、IP 安全檢核 |
| [`vision_yuv`](src/vision_yuv.rs) | Android Camera2 (NV21 / YUV420sp) 預覽幀灰階快速抽取、2x2 平均降採樣、亮度直方圖 | 手機相機邊緣運算、OCR 前處理、條碼辨識、自動曝光評估 |
| [`packet_checksum`](src/packet_checksum.rs) | 零依賴 CRC32 (IEEE 802.3)、Adler-32 校驗、16-bit PCM/感測器專用校驗 | 檔案總管校驗、感測器封包傳輸、PNG/ZIP 區塊校驗 |
| [`pcb_cad_fem`](src/pcb_cad_fem.rs) | Hammerstad 50Ω 微帶線阻抗求解、OpenEMS FDTD CFL 數值時間步長極限、Gerber 座標微米解析、RF 倒角補償、2D 多邊形面積 | KiCad / FreeCAD / OpenEMS 開源 EDA 工具鏈、PCB 高頻射頻走線設計、電磁仿真網格計算 |
| [`crypto_vault`](src/crypto_vault.rs) | 零依賴 SHA-256 摘要、常數時間比較 (防時序側信道攻擊)、XOR 流式混淆 | 機密分片校驗、快照雜湊校驗、敏感字串安全防護 |
| [`crypto_modern_wire`](src/crypto_modern_wire.rs) | RFC 8439 ChaCha20 四分之一輪與 16-word 狀態初始化、Poly1305 金鑰夾緊、32-bit 與 Curve25519 常數時間條件交換 (CSWAP)、安全 Zeroize 記憶體清除、BLAKE2s G 函數混合置換、GF(2^8) 有限域無分支乘法、Shamir 秘密共享門檻多項式求值、RFC 4226 HOTP/TOTP 動態截斷 | 現代線路加密通訊、雙向認證金鑰交換、防側信道時序攻擊、多方門檻密鑰分片、兩步驟動態驗證 (2FA) |
| [`sys_path`](src/sys_path.rs) | 跨平台相對路徑防穿越校驗、史實層只增不刪 (Append-Only) 差異校驗 | 檔案系統守衛、檔案總管路徑校驗、日誌保鮮關卡 |
| [`touch_control`](src/touch_control.rs) | 觸控螢幕消抖、EMA 滑動速度估算、壓力觸控三階遲滯分級、手勢狀態機 | 手機 App 觸控跟手度提升、小遊戲手勢判讀、虛擬搖桿 |
| [`vision_feature`](src/vision_feature.rs) | Otsu 自適應雙峰二值化、Sobel 3x3 邊緣梯度、積分圖 O(1) 區域像素求和 | 手機相機邊緣 OCR、條碼快速定位、文件邊界掃描、特徵萃取 |
| [`vision_morphology_fast`](src/vision_morphology_fast.rs) | 3x3 二值形態學膨脹/侵蝕、形態學梯度邊緣提取、FAST-9 圓環特徵角點判定、霍夫直線累積索引換算、投影積分直方圖、IoU 邊界框交併比、NCC 標準化互相關樣板匹配、RGB 轉 HSV 色彩空間與環形色相遮罩 | 邊緣視覺特徵提取、目標定位與 NMS 非極大值抑制、OCR 文本行切割、即時樣板比對、HSV 顏色分割 |
| [`vision_optical_flow_homography`](src/vision_optical_flow_homography.rs) | Harris 局部梯度結構張量步進、Lucas-Kanade 2x2 稀疏光流克萊姆法則求解、3x3 單應矩陣 (Homography) 齊次射影轉換、2D 仿射變換前向映射、次像素雙線性插值採樣、3x3 伴隨矩陣求逆、RANSAC 單應內點判定、SE(2) 平面剛體運動變換、Graham Scan 凸包外積轉向判定、2D 點集最小二乘直線擬合 | 影像特徵追蹤、透視投影畸變校正、全景拼接、相機位姿估計、機器視覺引導定位 |
| [`bezier_curve`](src/bezier_curve.rs) | 三次貝茲曲線座標內插、切線速度向量、曲線曲率幾何計算 | 網頁 Wasm 動態圖表、AI 簡報路徑平滑動畫、粒子幾何軌跡 |
| [`motion_dead_reckon`](src/motion_dead_reckon.rs) | 手機慣性導航零速更新 (ZUPT)、一維航位推算積分、觸覺震動馬達多模式時序生成 | 手機遊戲體感位移、步態/停頓消除漂移、馬達短震/警示反饋 |
| [`motion_render_timeline`](src/motion_render_timeline.rs) | Remotion 影格與時間戳雙向精確整數換算、SMPTE 時間碼格式化、CSS Cubic-Bezier 曲線牛頓法求值、SVG 描邊動畫、FFMPEG 剪輯時間字串 | React Remotion 視訊動態渲染、Headless 瀏覽器/Wasm 動態動畫、FFMPEG 影音剪輯管線 |
| [`fast_math`](src/fast_math.rs) | Safe Quake 快速倒數平方根 (0x5f3759df)、純整數開方 isqrt_u64、線性內插 lerp、Hermite 平滑階躍 smoothstep | 邊緣手遊物理引擎、粒子光照著色、UI 動畫平滑過渡 |
| [`geo_collision`](src/geo_collision.rs) | 軸對齊包圍盒 (AABB) 碰撞重疊、2D 線段空間精確交點、Jordan 射線法點在多邊形內外判定 | 手機虛擬按鍵/點擊命中判定、手遊 2D 碰撞、自定義卡片佈局排版 |
| [`instrument_industrial`](src/instrument_industrial.rs) | SCPI 科學計數法零拷貝解析、六位半電表量程指令構建、CANopen SDO 加速下載/心跳訊框、Modbus ASCII LRC/RTU 訊框構建、雙暫存器轉 Float32、DSLogic 深度換算、LiteVNA 步進頻率 | 工廠自動化 (PLC/CANopen/Modbus)、Keysight 34461A / Siglent SDM3065X 六位半電表、DSLogic 邏輯分析儀、LiteVNA 向量網絡分析儀 |
| [`instrument_lxi_hislip`](src/instrument_lxi_hislip.rs) | RFC 4506 XDR 編解碼與 4 位元組對齊填充、ONC RPC (RFC 5531) 呼叫標頭、VXI-11 Create_Link 與 Device_Read 參數序列化、IVI HiSLIP 16-byte 協議封包頭封裝與幻數校驗、LXI mDNS 服務名稱格式化 | 實驗室儀器網路連線 (LXI)、VXI-11 示波器/電表遠端乙太網控制、高速 HiSLIP 協定通訊 |
| [`iot_wire_protocol`](src/iot_wire_protocol.rs) | MQTT 3.1.1/5.0 變長整數 (Variable Byte Integer) 編解碼、PUBLISH 固定頭標誌打包、RFC 7252 CoAP 4-byte 固定頭封裝與 Option 增量壓縮、LoRaWAN MHDR 解碼與 16-bit 訊框計數器解纏繞、Modbus FC 0x10 封裝、RFC 1055 SLIP 串列封裝與轉義還原 | 物聯網雲端對接 (MQTT/CoAP)、遠距離低功耗廣域網 (LoRaWAN)、工控 PLC 多暫存器寫入 (Modbus)、串列通訊網路封裝 (SLIP) |
| [`instrument_scpi_vna`](src/instrument_scpi_vna.rs) | SCPI 儀器指令解析、六位半電表讀值統計、VNA S11 反射係數與 Smith 圓圖阻抗解算、VSWR/回波損耗、示波器過零頻率 | 實驗室儀器自動化 (Keysight/Siglent)、向量網絡分析儀 (LiteVNA)、示波器/六位半電表邊緣控制 |
| [`iso7816_smartcard`](src/iso7816_smartcard.rs) | Luhn 卡號模 10 校驗、ISO 7816 狀態碼判讀、Format 0 PIN Block 密碼塊構造、EMV BER-TLV 解析、敏感記憶體安全擦除 | 金融刷卡機 POS、ATM 提款機、PC/SC 讀卡機、防側信道洩漏 |
| [`cad_geom_model`](src/cad_geom_model.rs) | KiCAD 焊盤孔環寬度 (Annular Ring)、走線間距 (DRC Clearance) 違規校驗、IPC-2152 走線溫升、FreeCAD 圓角弦長/拉伸柱體體積/3D 包圍盒包含、OpenEMS Mur 吸收邊界係數與奈奎斯特網格步長、Remotion 彈簧阻尼比 | PCB 電路板製造驗證 (KiCAD)、開源機構 CAD (FreeCAD)、電磁仿真 (OpenEMS)、網頁影音動態渲染 (Remotion) |
| [`cnc_gcode_toolpath`](src/cnc_gcode_toolpath.rs) | G01 3D 直線插補座標步進、G02/G03 圓弧圓心與起訖角度解算、順/逆時針有向角跨度、3軸進給率速度分解 (mm/min -> mm/s)、G41/G42 刀具半徑左右補償法向量偏移、切削線速度轉主軸轉速 (RPM)、G83 啄鑽循環深度步進、每齒進給量轉進給率、梯形速度輪廓即時進給規劃 | CNC 數控雕刻機、PCB 電路板隔離銑削 (PCB Milling)、CAM 刀具路徑規劃、3D 列印 G-Code 解釋器 |
| [`fieldbus_industrial_eth`](src/fieldbus_industrial_eth.rs) | EtherCAT 乙太網訊框頭 (Type 1)/10-byte 數據報頭/WKC 工作計數器驗證/CoE 頭、PROFINET RT 訊框 ID 範圍分類/週期計數器步進、POWERLINK SoC 同步封包/PReq 拆解、Modbus TCP 7-byte MBAP 封包頭封裝與長度校驗 | 工業自動化現場總線、EtherCAT 馬達伺服驅動、PROFINET 工控 PLC 通訊、Modbus TCP 乙太網閘道器 |
| [`can_bus_frame`](src/can_bus_frame.rs) | CAN 2.0/FD ID 拆解、CAN-FD DLC 長度映射 (0..64)、Modbus RTU CRC16、ISO-TP 單幀封裝、MCU 定時器預分頻重載計算 | 車載 CAN 總線通訊、工控 RS485 模組、STM32/NXP/TI/Microchip 韌體 |
| [`can_lin_bus`](src/can_lin_bus.rs) | 車載 LIN 2.x PID 同位計算、Classic/Enhanced 增強校驗和、I2S 32-bit 數位音訊立體聲解包 | 車身車門/後視鏡/車窗 LIN 控制器、I2S 數位麥克風與音訊編解碼晶片 |
| [`comm_controller`](src/comm_controller.rs) | I2C 7-bit 讀寫訊框、SPI Mode 0..3 映射、5 線 SPI D/C 螢幕命令訊框、QSPI 0xEB 快速讀取、UART 波特率定點除數、DMA 跨架構記憶體對齊 | OLED/TFT 螢幕驅動、QSPI Flash 晶片、串口通信、8/16/32/64-bit CPU DMA 對齊 |
| [`comm_phy_bus`](src/comm_phy_bus.rs) | MIPI CSI-2 短封包 (含 6-bit ECC)、長封包 CRC16、MIPI D-PHY LP 低功耗狀態解碼、DSPI 雙通道讀取、QPI 四線啟用、RGMII 延遲抽頭 (Skew Taps) 與速率控制字、SGMII 1.25Gbps 自協商廣告字、跨架構 8/32-bit FIFO 打包解包 | 相機影像感測器 (CSI-2)、千兆以太網 PHY 介面 (RGMII/SGMII)、雙/四線 SPI Flash、跨 CPU 字長記憶體對齊 |
| [`clock_pll_rtc`](src/clock_pll_rtc.rs) | 晶振 PLL 鎖相迴路 (M, N, P) 最優倍頻分頻求解、RTC 32.768kHz 晶振拋物線溫漂補償、NTP 雙向網路時鐘對齊與延遲計算 | MCU 系統時鐘樹初始化、高精度 RTC 溫漂校準、物聯網網際網路系統時鐘同步 |
| [`physics_raycast_3d`](src/physics_raycast_3d.rs) | Möller–Trumbore 3D 射線-三角形高速相交、向量鏡面反射、點面有號距離 | 3D VR/AR 頭顯空間選取、雷射筆射線拾取、遊戲光照彈射物理模擬 |
| [`radar_aoa_mti`](src/radar_aoa_mti.rs) | MTI 雙脈衝/三脈衝延遲消除器、雙天線相位單脈衝測角 (AoA)、和差單脈衝比斜率、雷達盲速解算、Range-Doppler 網格轉實體座標、CSR 雜波訊噪比、FMCW 最大無模糊距離/速度、都卜勒頻心估算 | 雷達動目標顯示 (MTI)、毫米波雷達目標空間角度估計、地物雜波抑制、都卜勒頻譜中心校準 |
| [`radar_biomed_dsp`](src/radar_biomed_dsp.rs) | MIMO 正交 Hadamard CDMA 編碼、TDMA 天線排程、FMCW 距離解析度與拍頻測距、微都卜勒胸腔相位解纏繞、生醫呼吸帶通濾波、Q7/Q31 多位元量化 | 毫米波雷達生命徵象 (心跳/呼吸) 監測、FMCW 測距、MIMO 虛擬孔徑相控陣列、跨位元定點數 DSP |
| [`radar_cfar_doppler`](src/radar_cfar_doppler.rs) | 1D CA-CFAR 恒虛警率目標檢測、都卜勒速度解算、微都卜勒呼吸胸腔微動位移、點雲距離分群、MIMO 虛擬孔徑合成 | 毫米波雷達 (77GHz/60GHz)、智慧醫療呼吸心跳非接觸監測、ADAS 自駕車避障、點雲聚類 |
| [`radar_mimo_tracking`](src/radar_mimo_tracking.rs) | MIMO 虛擬天線陣列等效孔徑索引換算、TDMA 發射通道輪詢排程器、CDMA/PMCW 雷達 Barker 雙相編碼 (7/11/13)、脈衝壓縮匹配濾波步進、Capon/MVDR 自適應波束成形權重、GNN 歐氏距與馬氏距波門檢定、1D 卡爾曼濾波量測創新更新、航跡存活確認與信心衰減評分、雙 PRF 多普勒速度解模糊 | 毫米波雷達 MIMO 虛擬孔徑成像、雷達點雲多目標航跡關聯 (GNN/Kalman)、CDMA/PMCW 正交編碼解調、抗模糊測速 |
| [`fin_apdu_track`](src/fin_apdu_track.rs) | ISO 7816-4 APDU 晶片卡命令封裝與回應解析、磁條 Track 2 卡號效期提取、LRC 校驗 | 金融刷卡機 POS、ATM 讀卡機、晶片卡/磁條卡雙介面支付 |
| [`net_ethernet`](src/net_ethernet.rs) | Ethernet II 訊框解析、802.1Q VLAN Tag 識別、單播/組播/廣播 MAC 位址分類 | 嵌入式網路驅動、以太網閘道器、跨交換機 VLAN 標籤隔離 |
| [`net_tsn_macsec`](src/net_tsn_macsec.rs) | IEEE 802.1AS gPTP 時鐘事件訊息判定、Pdelay 鏈路納秒延遲解算、IEEE 802.1AE MACsec SecTAG 解析與防重放滑動視窗驗證 | 車載時間敏感網路 (TSN / DoIP / SOME/IP)、ADAS 感測器時鐘同步、金融資料中心 Layer 2 硬體加密防禦 |
| [`office_calc_layout`](src/office_calc_layout.rs) | Excel XLOOKUP 查表、PMT 貸款等額本息、NPV 淨現值、PERCENTILE 線性插值、Word Twips/EMUs 換算、貪婪文字折行、PPT 元素等距對齊 | 辦公試算表引擎 (Excel 代碼重現)、Word/PPT 跨平台排版佈局、報表財務精算 |
| [`doc_layout_typography`](src/doc_layout_typography.rs) | 點數 (pt) 與 Twips 雙向換算、TrueType/OpenType UPM 字體單元像素縮放、Word 段落首行縮排/左右邊距、精確行距換算、齊行 (Justified) 空格分攤、Knuth-Plass 排版折行惡化度、表格欄寬等比自動配額、PPT 幻燈片等比自適應居中、斷字連字號懲罰估算 | 專業排版渲染引擎 (Word/PPT 替代)、向量字體度量縮放、雜誌級齊行斷字排版、投影片投映佈局 |
| [`spc_six_sigma_msa`](src/spc_six_sigma_msa.rs) | 六標準差製程能力 (Cp/Cpk)、常態分佈累積概率 (Phi/erf)、規格內良率與 DPPM、雙邊截斷高斯分佈 (Truncated Normal) 統計值與 PDF、AIAG 車規 MSA 量測系統分析 (Gage R&R EV/AV, %GRR, P/T Ratio, ndc 區分度) | 半導體/電子工廠 SPC 品質管制、車用 IATF 16949 / AIAG MSA 量產放行、精密公差不良率精算 |
| [`spc_msa_advanced`](src/spc_msa_advanced.rs) | 長期性能指標 (Pp/Ppk)、樣本偏度與超額峰度、Box-Cox 冪變換、Shewhart X-bar/R 管制界限、Durbin-Watson 自相關、Weibull 可靠度、Anderson-Darling A² 常態檢定、卡方累積概率近似與 %PV 變異佔比 | 車規半導體良率分析、非高斯長尾製程正態轉換、SPC 管制圖即時監控、產品可靠度壽命預估 |
| [`quant_finance_market`](src/quant_finance_market.rs) | 技術面 EMA 步進/RSI 強弱/布林通道三軌/MACD、基本面 PE/PB/ROE/FCF Yield、籌碼面 VWAP/大單淨流入率、指數期貨基差 (Contango/Backwardation)/年化持有成本殖利率、匯率調整報酬率 | 量化高頻交易、股市籌碼與基本面精算、期現套利 (Cash & Carry)、跨市場匯率對沖 |
| [`quant_orderbook_l2`](src/quant_orderbook_l2.rs) | L2 最優買賣報價 (BBO) 中間價與基點價差、盤口訂單微觀不平衡度 (OBI)、交易量加權微觀價格 (Stoikov Micro-Price)、多檔深度加權中間價、有效價差、實現價差 (Realized Spread)、Kyle's Lambda 價格衝擊係數、Amihud 非流動性比率、VPIN 毒性訂單流指標步進、TWAP 時間加權平均價 | 高頻做市商造市報價、訂單流毒性預警、大單演算法拆單 (TWAP/VWAP)、市場衝擊成本估算 |
| [`rpa_screen_control`](src/rpa_screen_control.rs) | 螢幕 RGB 感知色彩距離 (Redmean)、模糊色塊找色 (Find Color Patch)、擬人滑鼠移動軌跡、雙擊狀態機、GPIB IEEE 488.1 3線握手、RS485 傳輸時延 | 土法自動化 (按鍵精靈 / PyAutoGUI 替代)、工控儀表 GPIB 握手、RS485 半雙工方向切換 |
| [`sensor_baro_als`](src/sensor_baro_als.rs) | 巴羅米特氣壓公式國際標準海拔高度解算、OPT3001/BH1750 環境光感測器 Lux 換算 | 智慧手錶/手環爬樓梯氣壓高度計、螢幕自動亮度調節、氣象站 |
| [`sensor_power_meter`](src/sensor_power_meter.rs) | 歐姆定律分流電流計算、庫侖計電量時間積分 (mAh)、單節鋰電池 OCV 開路電壓轉 SOC% | 智慧電池管理系統 (BMS)、可穿戴設備電量百分比顯示、功率計 |
| [`ble_adv_parser`](src/ble_adv_parser.rs) | BLE 4.2/5.0 廣播封包 AD Structure 零拷貝檢索、發射功率、設備名稱、製造商自定義數據解析 | 醫療手環、工廠 Beacon 室內定位、物聯網低功耗廣播解析（消滅 GC 震盪） |
| [`thermal_palette`](src/thermal_palette.rs) | FLIR / MLX90640 攝氏度陣列 5 段式 Ironbow 與 Rainbow 偽彩色映射、矩陣統計 | 無人機電力設備巡檢、紅外線熱斑成像、消滅 Android JNI GC 掉幀 |
| [`dsp_fixed_point`](src/dsp_fixed_point.rs) | Q15 定點數飽和乘法、FIR 濾波器步進卷積、增益調整、浮點互轉 | Cortex-M0/8051 無 FPU 單晶片信號處理、音訊邊緣濾波、整數秒算 |
| [`decision_mcts`](src/decision_mcts.rs) | 蒙地卡羅樹狀搜尋 (MCTS) UCB1 評分、最佳子節點選擇、反向傳播更新、訪問次數溫度策略映射 | 智慧決策系統、棋盤博弈 AI (AlphaGo 核心)、強化學習樹搜索、自主代理多步推理 |
| [`curve_fit_trend`](src/curve_fit_trend.rs) | 物理實驗線性回歸 (斜率/截距/相關係數)、R² 擬合判定係數、Huber 穩健抗噪損失、LOWESS Tricube 局部加權平滑、RBF/Matérn 3/2 核函數、傅立葉特徵映射、SIREN 正弦週期表示、KAN SiLU 活化 | 實驗物理數據擬合、儀表趨勢平滑濾除雜訊、高斯過程回歸核方法、隱式神經表示 (INR)、KAN 神經架構 |
| [`ai_autograd_train`](src/ai_autograd_train.rs) | Micrograd 標量自動求導反向傳播 (加/乘/冪/ReLU)、BPE 詞元頻率統計與合併、GPT-2 因果自注意力遮罩 (Causal Mask)、餘弦退火排程 (Cosine Annealing with Warmup)、全域梯度 L2 Norm 截斷、數值穩定 Log-Sum-Exp | 極簡神經網路訓練迴圈 (Karpathy nanoGPT / llm.c / Raschka)、純 Rust 詞元編碼、微型自求導引擎 |
| [`ai_tensor_gemm`](src/ai_tensor_gemm.rs) | 分塊矩陣乘法 (Tiled GEMM)、Conv2D 卷積前向步進、MaxPool2D 2x2 最大池化、Batch Normalization、Inverted Dropout、Embedding 查找、LeakyReLU、Smooth L1 Loss、餘弦相似度、One-Hot 編碼 | 深度學習神經網路推理與訓練底層算子、邊緣視覺卷積特徵提取、嵌入向量比對 |
| [`dsp_fft_radix2`](src/dsp_fft_radix2.rs) | Cooley-Tukey 基-2 (Radix-2) 快速傅立葉轉換 (FFT/IFFT)、Q15 定點數蝶形運算、位元反轉置換 | 雷達頻譜分析、圖像去噪、音訊壓縮與濾波、無 FPU 單晶片信號處理 |
| [`crypto_kdf`](src/crypto_kdf.rs) | PKCS#7 區塊填充與常數時間去填充（防 Padding Oracle）、PBKDF 加鹽拉伸、KDF 工作密鑰衍生 | 金融 ATM/POS 刷卡機、硬體安全模組 (HSM)、加密通訊韌體 |
| [`crypto_dukpt`](src/crypto_dukpt.rs) | ANSI X9.24 DUKPT 交易金鑰衍生、21-bit 交易計數器管理、PIN/Data 變體金鑰生成 | 金融刷卡機 POS、ATM 提款機每筆交易動態密鑰（防前向破解） |
| [`motor_stepper`](src/motor_stepper.rs) | 步進馬達梯形加減速微秒延遲計算、正交 AB 編碼器 4x 倍頻狀態機、行程換算 | 3D 列印機、CNC 雕刻機、送錫機、工控滑台高精度運動控制 |
| [`robotics_motion_profile`](src/robotics_motion_profile.rs) | 2-Link 平面機械臂逆向/正向運動學 (IK/FK)、雅可比奇異點檢測、梯形與 S-curve 躍度運動曲線、Q15 步進微步電流比、關節軟極限、滾珠螺桿導程 | 工業機械手臂 (SCARA/Delta)、六軸協作機器人逆解、高階 CNC 運動控制器 |
| [`robotics_6dof_slerp`](src/robotics_6dof_slerp.rs) | 6 軸機械手臂腕部球心解耦、單位四元數 SLERP 球面線性插值、Rodrigues 旋轉軸角與 SO(3) 矩陣互轉、五次多項式零衝擊軌跡規劃、單連桿重力負載靜力矩補償、笛卡爾直線插補 (Blend) | 六軸工業手臂 (UR5/Puma 560) 運動學逆解、末端姿態平滑過渡、無衝擊點對點軌跡規劃 |
| [`env_sensor`](src/env_sensor.rs) | SHT3x 溫濕度 CRC8 校驗、Magnus-Tetens 露點溫度計算、美國/台灣 EPA PM2.5 AQI 換算 | 恆溫恆濕環境試驗箱、防凝露結霜控制、智慧家庭環境儀 |
| [`thermal_pid`](src/thermal_pid.rs) | 抗積分飽和 PID 閉環控制器、10k NTC 阻值轉攝氏度、人眼對數視覺 PWM 映射、電功率計算 | 恆溫烙鐵/烘烤箱韌體、智能溫控濕度箱、LED 呼吸燈、電量監控 |
| [`text_cleaner`](src/text_cleaner.rs) | UTF-8 邊界安全截斷、字詞邊界智能折行、CJK 字元計數 | 字幕視線居中 (RSVP)、翻譯文字流緩衝管理 |

---

## 三、快速使用範例

### 1. 引用方式 (在其他專案的 `Cargo.toml`)

```toml
[dependencies]
rust-bricks = { git = "https://github.com/mangala1728/rust-bricks.git" }
# 或本機路徑引入
# rust-bricks = { path = "../rust-bricks" }
```

### 2. 程式碼調用

```rust
use rust-bricks::math_time::clamp_i64;
use rust-bricks::crypto_vault::sha256_hex;
use rust-bricks::audio_vad::cps_chars_per_second;

fn main() {
    // 1. 安全數值邊界夾止
    let safe_val = clamp_i64(150, 0, 100);
    assert_eq!(safe_val, 100);

    // 2. 零依賴 SHA-256
    let hash = sha256_hex(b"hello rust-bricks");
    println!("SHA-256: {}", hash);

    // 3. 字幕 CPS 計算（字數 10，耗時 2000ms -> 5.0 字/秒）
    let cps = cps_chars_per_second(10, 2000).unwrap();
    assert_eq!(cps, 5.0);
}
```

---

## 四、AI 代理專用機器索引（Machine-Readable Catalog）

> **【甩手掌櫃免讀】** 本庫所有函式均已被結構化抽取至索引清單。  
> AI 碼工代理在跨專案開發時，請直接讀取下列檔案即可在 O(1) 時間內獲得可用積木、型別簽章與邊界定義，無需逐一掃描各個 `.rs` 原始檔：

- **YAML 格式（推薦代理檢索）**：[`CATALOG.yaml`](CATALOG.yaml)
- **JSON 格式（程式化讀取）**：[`CATALOG.json`](CATALOG.json)
- **自動更新指令**：新增積木後請執行 `cargo run -- catalog` 秒級同步最新機器索引。

---

## 五、倉庫容量與合約守門員（100% 純 Rust 內建守護）

為防止 AI 代理隨意提交二進位大檔或破壞 09 合約，本庫內建純 Rust 剛性守門員（`cargo run -- guard`）並接線至 Git `pre-commit` 與 `.agents/hooks.json`：

1. **二進位多媒體直擋**：嚴禁提交音訊（.wav/.mp3）、圖像（.png/.jpg）、封裝檔（.zip/.tar）或模型權重檔（.onnx/.pt）。
2. **單檔 500 KB 硬上限**：嚴禁提交超大日誌或龐大硬編碼測試數據，測試數據一律在代碼中以虛擬陣列動態生成。
3. **零外部依賴防線**：`Cargo.toml` 的 `[dependencies]` 必須為空。
4. **09 號合約自動審查**：提交前自動檢查 `## Map`、`@outputs`、`@domain` 守衛，禁止 `static mut` 與遞迴。
5. **單元測試全綠保證**：提交前自動執行 `cargo test`。

```bash
# 執行容量與合約守護檢查（100% 純 Rust）
cargo run -- guard

# 自動產生/更新機器索引（100% 純 Rust）
cargo run -- catalog

# 執行 CLI 自我檢驗程式
cargo run -- selftest

# 執行全部單元測試
cargo test

# 巡檢外部 contrib 貢獻分支（防撞車與體檢驗收）
cargo run -- patrol

# 執行零信任品質總監黑箱混沌抽檢（0 崩潰破壞性測試）
cargo run -- audit

# 瀏覽或建立跨專案 AI 代理許願單
cargo run -- wish list
```

---

## 六、多專案防撞車與核心策展管家協議（Curation & Integration）

為防止跨專案多 AI 代理並行上傳 Rust 代碼造成版本衝突與重複造輪子，本庫建立了「核心策展管家與防撞車協議」（詳見 [`docs/CURATION-AND-INTEGRATION.md`](docs/CURATION-AND-INTEGRATION.md)、[`docs/ZERO-TRUST-QUALITY-DIRECTOR.md`](docs/ZERO-TRUST-QUALITY-DIRECTOR.md)、[`docs/DUAL-AXIS-EVOLUTION-STRATEGY.md`](docs/DUAL-AXIS-EVOLUTION-STRATEGY.md)、[`docs/QUALITY-PATROL-AND-AUTOMATION-SCHEDULE.md`](docs/QUALITY-PATROL-AND-AUTOMATION-SCHEDULE.md)、[`docs/TRANSPILATION-METHODOLOGY.md`](docs/TRANSPILATION-METHODOLOGY.md)、[`docs/HUNTING-HIGH-VALUE-BRICKS.md`](docs/HUNTING-HIGH-VALUE-BRICKS.md) 與 [`CONTRIBUTING.md`](CONTRIBUTING.md)）：

1. **巡檢外部貢獻分支**：執行 `cargo run -- patrol` 主動拉取並檢查 `origin/contrib/*`。
2. **機械防重發明輪子**：`guard.rs` 自動掃描全庫函式名稱，禁止重複命名。
3. **`lib.rs` 字母排序防線**：`guard.rs` 自動審查 `src/lib.rs` 宣告排序，防止分支合併衝突。
4. **分支隔離上線**：本地開 `worker/*` 或 `integrate/*` 分支開發，經 `rebase main` 與五板斧驗收後方可 merge。
5. **缺陷硬鎖（Defect Gate）**：外部代碼有坑開單 `incidents/INC-*.md (OPEN)`，守衛剛性阻擋全庫入庫直至修復。
