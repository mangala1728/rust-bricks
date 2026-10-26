# 熱成像感測器 (FLIR / MLX90640) 攝氏度陣列轉 Ironbow 偽彩色 RGB 映射微積木

wish_id: WISH-20261026-001
requester: 巡檢無人機-邊緣視覺代理 (drone-vision-agent)
status: FULFILLED
priority: HIGH
created_at: 2026-10-26
fulfilled_at: 2026-10-26
implemented_in: src/thermal_palette.rs

## 1. 需求背景與場景（Context）
- 使用專案：高空巡檢無人機太陽能板與電力設備熱斑檢測 Android App。
- 遇到瓶頸：目前使用 Python OpenCV `applyColorMap` 或 Java 手刻雙層迴圈，每秒 30fps 時在手機端引發頻繁 GC 停頓，導致畫面掉幀。

## 2. 原語言代碼或演算法本質（Source Algorithm）
- 原語言：Python NumPy + Matplotlib / C++ FLIR SDK
- 演算法本質：將低解析度 (如 32x24 或 160x120) 的 `f32` 溫度浮點數矩陣，依照自定義或標準鐵紅色彩帶 (Ironbow / Rainbow Palette) 線性或分段內插為 RGB24 像素陣列。

## 3. 期望的 Rust 微積木簽章（Target Signature）
```rust
/// @outputs: 1
/// @domain: min_celsius [-50.0..500.0]
/// @domain: max_celsius [-50.0..500.0]
pub fn thermal_ironbow_rgb(
    temp_celsius: f32,
    min_celsius: f32,
    max_celsius: f32,
) -> (u8, u8, u8);
```

## 4. 效益與指標（ROI）
- 記憶體優化：零 Heap 分配，消滅 JNI 跨語言物件生成，待機功耗預期降低 18%。
- 速度提升：預計從 Python/Java 的 12ms/幀 降低至 Rust 純記憶體 < 0.2ms/幀。

## 5. 履約驗收成果（Fulfillment Receipt）
- 已收錄於 `src/thermal_palette.rs`：
  - `thermal_ironbow_rgb`：5 段線性插值標準 FLIR Ironbow 色帶。
  - `thermal_rainbow_rgb`：標準彩虹高對比偽彩色映射。
  - `thermal_matrix_stats`：零依賴單遍矩陣統計（Min/Max/Avg）。
- 驗證結果：全單元測試與黑箱混沌審計通過，零崩潰、零 Heap 分配。
