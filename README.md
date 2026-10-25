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

---

## 二、模組方塊圖（Module Map）

| 模組 | 職責 | 典型應用場景 |
|---|---|---|
| [`math_time`](src/math_time.rs) | 高精度數值夾止、滑動均線、儒略日曆法轉換、占星大限 (Dasha) 比例運算 | 股市量化、占星天氣預報、時間序列平滑 |
| [`audio_vad`](src/audio_vad.rs) | PCM16 音訊取樣點能量 (RMS) 計算、靜音幀判定、CPS 字幕字速審查 | 語音活動檢測 (VAD)、即時翻譯 App、多模態字幕對齊 |
| [`crypto_vault`](src/crypto_vault.rs) | 零依賴 SHA-256 摘要、常數時間比較 (防時序側信道攻擊)、XOR 流式混淆 | 機密分片校驗、快照雜湊校驗、敏感字串安全防護 |
| [`sys_path`](src/sys_path.rs) | 跨平台相對路徑防穿越校驗、史實層只增不刪 (Append-Only) 差異校驗 | 檔案系統守衛、日誌保鮮關卡、快照還原校驗 |
| [`text_cleaner`](src/text_cleaner.rs) | UTF-8 邊界安全截斷、字詞邊界智能折行、標點規範化 | 字幕視線居中 (RSVP)、翻譯文字流緩衝管理 |

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

## 四、品質與自測

```bash
# 執行全部單元測試
cargo test

# 執行 CLI 自我檢驗程式
cargo run -- selftest
```
