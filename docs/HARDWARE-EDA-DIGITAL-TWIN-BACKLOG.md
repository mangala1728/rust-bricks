# 個人硬體公司：EDA 幾何、電磁、熱模擬與 3D 數位雙生作戰大綱 (Hardware EDA & Digital Twin Backlog)

> 本文件為個人硬體工作室（Solo Hardware Venture）專屬之長期技術儲備藍圖。  
> 結合 **Signex (純 Rust EDA)**、**Fyrox (純 Rust 3D 工業數位雙生引擎)** 與 **Blender (開源 3D DCC 之王)**，透過 `rust-bricks` 純 Rust、零外部依賴、09 號合約與 NASA JPL 60 行微積木體系，逐步實現一人全閉環硬體研發製造能力。

---

## 一、宏觀架構與五大技術支柱

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    個人硬體工作室（Solo Hardware Venture）                   │
│         一人即團隊：規格書秒讀 -> 線路設計 -> 物理模擬 -> 3D驗配 -> 製造交付      │
└──────────────────────────────────────┬───────────────────────────────────────┘
                                       ▼
┌──────────────────────────────────────────────────────────────────────────────┐
│       電子設計自動化 (EDA) 與幾何佈局 (Signex / KiCad + Rust 微積木)          │
│ • 微帶線/共面波導阻抗 (Z0)   • 差分對蛇形等長匹配   • 即時幾何 DRC 碰撞檢驗   │
│ • 淚滴 (Teardrop) 圓弧生成   • IPC-7351 焊盤尺寸    • Gerber / Drill 解析    │
└───────────────┬──────────────────────┬──────────────────────────┬────────────┘
                │                      │                          │
                ▼                      ▼                          ▼
┌────────────────────────┐ ┌────────────────────────┐ ┌────────────────────────┐
│  支柱一：電磁與射頻    │ │  支柱二：熱設計與散熱   │ │ 支柱三：3D機構與數位雙生 │
│  (SI / PI / RF / EMC)  │ │  (Thermal Simulation)  │ │ (Fyrox + Blender)      │
│ • S 參數 / VSWR / 反射 │ │ • 熱阻網路與結溫 TJ    │ │ • glTF / STEP 網格解析 │
│ • 史密斯圓圖阻抗匹配   │ │ • PCB 散熱過孔並聯等效 │ │ • 元件與機殼 AABB 碰撞 │
│ • 趨膚效應與銅箔粗糙度 │ │ • 2D 穩態熱傳導 FDM    │ │ • 2D 板框拉伸 3D 實體  │
└────────────────────────┘ └────────────────────────┘ └────────────────────────┘
```

---

## 二、候選微積木清單（待逐步納入規格池）

### 支柱一：電磁相容 (EMC)、訊號完整性 (SI) 與射頻微波
| 模組名稱 | 目標功能與工程價值 | 預計輸出契約 |
| :--- | :--- | :--- |
| `eda_impedance_microstrip` | IPC-2141 表面微帶線特性阻抗 $Z_0$ 與傳播延遲 $t_{pd}$ 計算 | `Option<(f64, f64)>` |
| `eda_impedance_stripline` | 對稱帶狀線內層高速走線阻抗與有效介電常數解析解 | `Option<f64>` |
| `eda_impedance_cpw` | 接地共面波導 (CPW with Ground) 射頻阻抗多項式估算 | `Option<f64>` |
| `eda_rf_s_parameters` | 散射矩陣 $S_{11}$ 轉反射損耗 (Return Loss)、插入損耗與駐波比 (VSWR) | `Option<(f64, f64, f64)>` |
| `eda_rf_smith_chart` | 史密斯圓圖複數反射係數 $\Gamma$ 與正規化複數阻抗互轉 | `Option<(f64, f64)>` |
| `eda_rf_skin_depth` | 高頻交流趨膚深度 $\delta$ 與銅箔表面粗糙度修正阻抗增加比率 | `Option<(f64, f64)>` |

### 支柱二：PCB 佈局、即時幾何 DRC 與生產製造
| 模組名稱 | 目標功能與工程價值 | 預計輸出契約 |
| :--- | :--- | :--- |
| `eda_drc_point_to_segment` | 點到走線線段之最近幾何距離（即時 DRC 走線間距核查底層） | `Option<f64>` |
| `eda_trace_serpentine_meander` | 高速差分對蛇形繞線 (Meander Trace) 延遲匹配折點幾何生成 | `Option<Vec<[f64; 2]>>` |
| `eda_pad_teardrop_curve` | 焊盤/過孔淚滴補強 (Teardrop) 平滑圓弧幾何切點推導 | `Option<([f64; 2], [f64; 2])>` |
| `eda_ipc7351_footprint_calc` | IPC-7351B 表面貼裝元件焊盤尺寸推導 (Least/Nominal/Most) | `Option<([f64; 2], [f64; 2])>` |
| `eda_gerber_rs274x_tokenizer` | Gerber RS-274X 指令輕量流式 Tokenizer (D01, D02, D03, Flash) | `Option<GerberToken>` |
| `eda_drill_excellon_parser` | Excellon 數控鑽孔代碼 (Tool, X, Y 座標) 串流解析器 | `Option<(u8, f64, f64)>` |
| `eda_centroid_smd_align` | SMT 貼片座標 (Pick & Place Centroid) 旋轉矩陣與原點對齊 | `Option<[f64; 2]>` |

### 支柱三：熱設計、功耗與熱模擬
| 模組名稱 | 目標功能與工程價值 | 預計輸出契約 |
| :--- | :--- | :--- |
| `eda_thermal_junction_calc` | 晶片結溫 $T_J$ 估算與等效熱阻網路 ($\theta_{JA} = \theta_{JC} + \theta_{CS} + \theta_{SA}$) | `Option<f64>` |
| `eda_thermal_vias_parallel` | 散熱過孔陣列 (Thermal Vias) 銅箔電鍍等效固體熱阻與導熱係數 | `Option<f64>` |
| `eda_ipc2152_trace_current` | IPC-2152 走線電流承載力、銅厚 (1oz/2oz) 與允許溫升計算 | `Option<f64>` |
| `eda_thermal_fdm_2d_step` | PCB 銅箔二維穩態熱傳導有限差分 (FDM) 五點差分單步步進 | `Option<f64>` |

### 支柱四：3D 機構、數位雙生與渲染橋接 (Fyrox & Blender)
| 模組名稱 | 目標功能與工程價值 | 預計輸出契約 |
| :--- | :--- | :--- |
| `eda_3d_aabb_interference` | PCB 元件與 3D 機殼內壁 AABB/OBB 碰撞干涉即時檢測 | `Option<bool>` |
| `eda_polygon_extrusion_mesh` | 2D 板框多邊形拉伸成 3D 棱柱體 Mesh（餵給 Fyrox 物理與渲染） | `Option<Mesh3D>` |
| `eda_gltf_chunk_reader` | glTF 2.0 / GLB 二進位網格頂點與索引緩衝區輕量解讀器 | `Option<GltfChunk>` |

### 支柱五：規格書視覺多模態與 OCR 轉換工具鏈 (Datasheet Intelligence)
| 模組名稱 | 目標功能與工程價值 | 預計輸出契約 |
| :--- | :--- | :--- |
| `datasheet_pin_table_cleaner` | 晶片引腳表格 OCR 後 OCR 混淆字元清洗與引腳編號連續性驗證 | `Option<PinRow>` |
| `datasheet_reg_bitfield_packer` | 暫存器位元欄位非重疊、邊界覆蓋與讀寫屬性結構化驗證 | `Option<RegisterDef>` |
| `datasheet_elec_limits_guard` | 電氣極限邊界提取並自動生成 09 合約 `// @domain:` 執行期守衛 | `Option<String>` |

---

## 三、執行時程與階段目標

1. **Phase A（底層物理與幾何演算法先行儲備）**：
   - 伴隨日常規格推進，將阻抗、DRC 距離、熱阻、S 參數等演算法納入規格佇列，交由雲端農夫自動產出測試與實作。
2. **Phase B（規格書智慧解讀多模態腳本）**：
   - 建立 Python / Rust 混合工具，讀取 PDF 晶片手冊，自動萃取 Pinout 與 Register Map 生成 Markdown / YAML。
3. **Phase C（Signex + Fyrox + Blender 無頭串接原型）**：
   - 編寫自動化腳本，將 PCB 走線與元件 3D 模型匯出，無縫置入 Fyrox 進行數位雙生互動與 Blender 拟真產品渲染。
