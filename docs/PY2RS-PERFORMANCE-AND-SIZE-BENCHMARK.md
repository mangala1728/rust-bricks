# `/py2rs` 純 Rust 轉寫效能、容量、記憶體與 Android 實機實測報告

> **量測日期**：2026-11-21  
> **來源專案**：`ai-interpreter` 全專案 Python → 純 Rust（09 號合約邊界）轉寫工程  
> **對應積木模組**：
> - [`src/encoding_base64_json.rs`](../src/encoding_base64_json.rs)（零依賴 Base64 / JSON 編解碼）
> - [`src/archive_zip_store.rs`](../src/archive_zip_store.rs)（零依賴 ZIP Store / PNG 編碼）
> - [`src/net_http_micro.rs`](../src/net_http_micro.rs)（標準庫阻塞式微 HTTP 客戶端與 `/health` 閘道骨架）
> - [`src/code_kotlin_lint.rs`](../src/code_kotlin_lint.rs)（Kotlin 09 合約與跨層靜態掃描器）
> - [`src/code_python_structure.rs`](../src/code_python_structure.rs)（Python 結構與 `@outputs` 檢查器）
> - [`src/decision_local_edge.rs`](../src/decision_local_edge.rs)（本地邊緣決策路由與邊界護欄，詳見 [`LOCAL-DECISION-MODEL-BENCHMARK.md`](LOCAL-DECISION-MODEL-BENCHMARK.md)）
>
> **量測環境**：Windows 11（Python 3.14.x / Rust `x86_64-pc-windows-gnullvm` + `aarch64-linux-android`）＋ Samsung `SM-S9470`（`R5GL22PP9SA` / Qualcomm `qcom` / Android 16, API 36 / `arm64-v8a`）

---

## 一、程式容量對照（Python vs Rust 預設 Release vs `[profile.release]` 瘦身版）

透過在 `Cargo.toml` 啟用 `[profile.release]`（`opt-level = "s"`、`lto = "thin"`、`codegen-units = 1`、`strip = true`，保留 unwind 語意），桌機 13 支執行檔與 Android ARM64 `.so` 體積皆縮減 **31% ～ 42%**：

| 項目 | Python 基準／執行期 | Rust 預設 Release | Rust `[profile.release]` 瘦身版 | 差異說明 |
|---|---:|---:|---:|---|
| 原始碼總量 | 35 檔 / `399,638 B` | 72 檔 / `473,795 B` | 72 檔 / `473,795 B` | Rust 多 `18.6%`（含 09 合約註解、邊界守衛與單元測試） |
| 執行期環境體積 | `.venv`: `49.98 MB`<br>（`Python314`: `1,991.74 MB`） | 13 支 `.exe` 合計 `7.73 MB` | **13 支 `.exe` 合計 `5.17 MB`**（`5,423,104 B`） | 部署零依賴；13 支總減幅 **-33.1%** |
| Android ARM64 `.so` | 無法於 Android 直接執行 CPython | `588,184 B`（`574.4 KB`） | **`388,800 B`（`379.7 KB`）** | 減幅 **-33.9%**，維持 16KB ELF 頁對齊 |
| Android ARM64 `e2e-bench` | — | `615,816 B`（`601.4 KB`） | **`355,792 B`（`347.5 KB`）** | 減幅 **-42.2%** |

### 桌機單支二進位瘦身幅度（代表性工具）
- `qgates.exe`（全套 13 道品質關卡）：`947,200 B` → **`610,304 B`（-35.6%）**
- `backend-server.exe`（`/health` HTTP/1.1 閘道）：`537,600 B` → **`350,208 B`（-34.9%）**
- `dhooks.exe`（PreToolUse 攔截守衛）：`700,928 B` → **`471,552 B`（-32.7%）**
- `kt-lint.exe`（對應 `code_kotlin_lint`）：`568,832 B` → **`384,512 B`（-32.4%）**
- `structure-check.exe`（對應 `code_python_structure`）：`514,560 B` → **`348,160 B`（-32.3%）**

---

## 二、桌機執行時間與記憶體占用對照（Win32 `GetProcessMemoryInfo` 5 輪中位數）

| 任務（相同輸入） | Python 耗時 | Rust 預設 Release | Rust 瘦身版 | **對 Python 加速** | Python Peak RSS / Commit | Rust 瘦身版 Peak RSS / Commit | **Commit 節省** |
|---|---:|---:|---:|---:|---:|---:|---:|
| **`dhooks pre-edit`**（Track 1 攔截） | 194.7 ms | 7.1 ms | **6.4 ms** | **30.4×** | 26.54 / 18.16 MB | **4.80 / 0.78 MB** | **小 23.3×** |
| **`kt-lint`**（235 個 `.kt`） | 1,680.6 ms | 84.2 ms | **99.9 ms** | **16.8×** | 23.51 / 15.82 MB | **6.31 / 3.27 MB** | **小 4.8×** |
| **`structure-check`**（`backend/app`） | 96.4 ms | — | **6.5 ms** | **14.8×** | 19.12 / 14.04 MB | **4.46 / 0.69 MB** | **小 20.3×** |
| **`summarize-stress`**（壓力測試彙整） | 60.7 ms | 7.0 ms | **6.1 ms** | **10.0×** | 12.93 / 7.68 MB | **4.55 / 0.72 MB** | **小 10.7×** |
| **`callgraph`**（Android 呼叫圖 JSON） | 110.8 ms | 22.2 ms | **23.0 ms** | **4.8×** | 22.48 / 17.97 MB | **11.93 / 9.38 MB** | **小 1.9×** |
| **`e2e-bench`**（12 WAV 音訊 DSP） | — | — | **11.6 ms** | — | — | **4.68 / 1.41 MB** | — |

### 常駐 HTTP 閘道（`FastAPI + uvicorn` vs `backend-server.exe`）
- **Python (`FastAPI + uvicorn` 於 Python 3.14)**：載入即占用 **43.18 MB Peak RSS / 27.77 MB Private**（15 秒內未完成綁定）。
- **Rust 瘦身版 (`backend-server.exe`)**：啟動就緒 **42.6 ms**，50 次 `GET /health` 全數成功（`50/50`、`11.91 ms/req`），峰值僅 **4.51 MB Peak RSS / 0.70 MB Private**（私有記憶體省下約 **39.7×**）。

---

## 三、Android 實機 (`SM-S9470`) 運行速度與 App 內 Rust 比重實測

1. **原始碼行數比重**：Rust `core+audio` **5,531 行**（21 檔） vs Android main Kotlin **50,575 行**（236 檔）→ **9.86%**。
2. **APK 體積比重**（`base.apk` 壓縮 `61 MB` / 解壓 `83.83 MB`）：`other .so`（`liblitertlm_jni.so` 等）占 **54.39%**（`45.59 MB`），`classes*.dex` 占 **43.58%**（`36.54 MB`），`res/` 占 **1.34%**（`1.12 MB`），`libai_interpreter_tools.so` 僅占 **0.56%**（瘦身後進一步降至 `388,800 B` = `0.37 MiB`）。
3. **實機冷啟動與記憶體**：冷啟動中位數 **`307 ms`**（`350 / 307 / 302 ms`）；`dumpsys meminfo` 總 `TOTAL PSS = 84,314 KB`、`TOTAL RSS = 216,072 KB`（`.so mmap` 僅占 `2,565 KB` PSS）。
4. **ARM64 原生算力 vs 目前 App 接線狀態**：
   - `SM-S9470` 實機 `verify_jni_arm64` **11/11 JNI 通過**；機上 `e2e-bench` 執行 12 支 WAV 音檔降噪＋VAD＋雙向說話者判斷全程僅 **`45 / 45 / 34 ms`**（中位數 **`45 ms`**）。
   - 目前 Android 非測試 UI 中僅 `MainActivity.kt` 與 `AppScreens.kt` 呼叫了 `NativeCoreAdapter.coreVersion()`，其餘音訊/搜尋熱路徑仍走 Kotlin fallback，因此現階段對 App 使用者體感速度改變接近 **0%**；後續將 Kotlin 熱路徑切入 `NativeCoreAdapter` 後即可直接釋放毫秒級 ARM64 原生 DSP 效能。
