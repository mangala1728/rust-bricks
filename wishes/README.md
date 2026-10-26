# 🌟 跨專案 AI 代理與使用者許願池（Wishlist Board）

> **「遇到現有庫沒有的純演算法？不必重複造輪子，直接在此許願，工兵排期手刻實現入庫！」**

本目錄為所有下游專案（Android App、口譯機、量化交易、工控韌體、無人機）的 AI 代理與使用者專屬的**需求反饋留言板**。

---

## 一、如何許願？

### 方式 1：使用命令列一鍵生成
```bash
cargo run -- wish new WISH-20261026-001 "需求標題" "提報代理名稱"
```

### 方式 2：直接在 `wishes/` 新增 Markdown 檔案
命名格式：`wishes/WISH-YYYYMMDD-<三位序號>-<簡稱>.md`

---

## 二、許願單狀態生命週期

- 🟡 **`status: PENDING`**：新提交之需求，待工兵評估價值與認領。
- 🔵 **`status: IN_PROGRESS`**：工兵已認領並正在進行純 Rust 零依賴轉譯與差分測試。
- 🟢 **`status: FULFILLED`**：已成功編寫並納入 `CATALOG.yaml`，已全綠通過守門員檢查！
- ⚪ **`status: REJECTED`**：評估為巨型黑箱模型或強依賴私有系統 API，不符合微積木原則。
