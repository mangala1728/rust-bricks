# 零信任品質總監黑箱抽檢體系（Zero-Trust Quality Director & Adversarial Audit）

> **「不看實作、不信承諾，模擬最刁鑽、最惡意的客戶端黑箱調用；唯有在狂轟濫炸下 0 崩潰且優雅收斂者，方准存活。」**  
> 本體系為 `rust-bricks` 庫專屬的獨立品質監督與破壞性混沌抽檢架構（Chaos Adversarial Audit）。

---

## 一、核心哲學：黑箱視角與反向思維（The Zero-Trust Mindset）

傳統代碼審查（Code Review）最大的盲點是「白箱偏見（Whitebox Bias）」——審查員看著作者寫的代碼，不自覺地順著作者預設的「正常路徑（Happy Path）」去理解，從而漏掉真正致命的邊界與未定義行為。

**零信任品質總監的四大鐵律**：
1. **絕不看內部原始碼實作（Zero-Whitebox Inspection）**：
   - 品質總監把自己當作一個完全不認識代碼作者的外部使用者，只看 `CATALOG.yaml` 的公開簽章與介面合約。
2. **反向思維與惡意攻擊輸入（Adversarial Chaos Fuzzing）**：
   - 專挑開發者「以為沒人會這樣傳」的極限、無效、溢出與畸形參數狂轟濫炸。
3. **崩潰零容忍（Zero Panic Guarantee）**：
   - 在任何輸入下，子函式絕對禁止觸發 `panic!`、陣列越界、或除以零中斷；必須優雅回傳預設值、飽和截斷值、`None` 或 `Err`。
4. **時序與記憶體有界性（Bounded State & Execution）**：
   - 拒絕死迴圈、拒絕無界分配、杜絕退格鍵下溢與記憶體殘留。

---

## 二、黑箱抽檢維度與破壞性測試用例庫（Chaos Test Matrix）

| 檢驗維度 | 模擬攻擊場景 | 代表性惡意輸入 | 品質總監預期收斂標準 |
|---|---|---|---|
| **浮點與幾何極值** | 3D VR/幾何引擎被傳入非數或無限大 | `f32::NAN`, `f32::INFINITY`, 負半徑球體, 倒置包圍盒 | 幾何計算回傳 `None` 或單位四元數，絕不發散 |
| **數值範圍倒置** | 用戶傳入顛倒的邊界區間 | `clamp_i64(50, 100, 0)`（$min > max$） | 拒絕非法夾止，維持原值回傳 |
| **音訊串流中斷** | 麥克風音訊串流瞬斷、取樣率除以零 | 空切片 `&[]`、取樣耗時 `0ms` | 能量回傳 `0.0`，CPS 計算回傳 `None` |
| **物理感測器異常** | GPS 漂移到外太空、負絕對零度 | 緯度 `999.0`、熱敏電阻阻值 `-100Ω`、溫度 `-300°C` | Haversine 與 NTC 計算返回 `None` |
| **金融與智慧卡黑客** | 傳入超長字元、非數字 PIN、畸形 TLV | 卡號包含非法字母、PIN 長度 `<4`、截斷 TLV 長度 | Luhn 回傳 `false`，TLV 返回 `None`，杜絕記憶體越界讀取 |
| **車載與工控總線** | 傳入超標 CAN FD DLC、時鐘為 0 | `DLC = 255`、系統時鐘 `0Hz` | DLC 映射為 `0`，Timer 計算返回 `None`，防硬體 Fault |
| **終端 Harness 炸彈** | 連續退格清空空緩衝區、未閉合 ANSI | 50 個 `\x08` (BS)、未閉合之 `\x1B[31` | 緩衝區指標絕不下溢為負數，ANSI 脫色長度精確歸零 |

---

## 三、自動化巡檢指令與守護防線（CLI Integration）

### 1. 手動觸發品質總監突擊抽查
```bash
cargo run -- audit
# 或
cargo run -- chaos
```
**輸出範例**：
```text
🕵️ [rust-bricks] Running Zero-Trust Quality Director Chaos Audit (Black-Box)...
✨ [rust-bricks] 零信任黑箱巡檢完成！共抽查 4 大維度極限反向輸入，全部安全收斂 (0 崩潰，100% 通過)！
```

### 2. 強制整合於 `guard.rs` 機械防線
本庫的 `cargo run -- guard`（以及 Git Commit Pre-hook）已將零信任品質總監巡檢設為**「防線五（Line 5）」**。任何未通過品質總監反向黑箱狂轟濫炸的代碼，將被強制攔截入庫！
