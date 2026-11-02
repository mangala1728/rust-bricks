# 規格書：二維單調鏈凸包交叉積掃描 (2D Convex Hull Monotone Chain)

## 模組名稱
`geo_convex_hull_2d`

## 業務目標與適用場景
在無人機航路碰撞檢測、雷射雷達 (LiDAR) 點雲邊界框擬合與地理資訊系統 (GIS) 中，Andrew's Monotone Chain 凸包演算法以 $O(N \log N)$ 複雜度穩健求取最小凸多邊形外輪廓。
本模組提供零外部依賴、純 `std` 實作的二維叉積旋向判定與單調鏈掃描微積木。

## 09 號合約邊界規範

### 函式 1：二維向量叉積旋向判定 (Cross Product Orientation)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn cross_product_2d(p0: (f64, f64), p1: (f64, f64), p2: (f64, f64)) -> f64
  ```
- **數學公式**：
  - $\vec{u} = (p_1.x - p_0.x, \; p_1.y - p_0.y)$
  - $\vec{v} = (p_2.x - p_0.x, \; p_2.y - p_0.y)$
  - $\text{cross} = \vec{u}.x \cdot \vec{v}.y - \vec{u}.y \cdot \vec{v}.x$
  - 若 $\text{cross} > 0$ 表示逆時針轉向 (Left turn)；$\text{cross} < 0$ 表示順時針轉向 (Right turn)；$= 0$ 表示共線。

### 函式 2：單調鏈凸包構建 (Monotone Chain Convex Hull)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn convex_hull_2d(mut points: Vec<(f64, f64)>) -> Option<Vec<(f64, f64)>>
  ```
- **運算過程**：
  - 若 `points.len() < 3`，若非空直接回傳原點列表或 `None`。
  - 按 X 座標升序排序，若 X 相等則按 Y 座標升序排序。
  - 構建下凸包 (Lower Hull)：
    歷遍點 $P$，若當前凸包長度 $\ge 2$ 且 `cross_product_2d(hull[k-2], hull[k-1], P) <= 0.0`，則彈出頂部點。
  - 構建上凸包 (Upper Hull)：
    同理逆向歷遍。
  - 合併上下凸包（去除重複首尾點）。

## 黃金測試向量 (Golden Vectors)
- 點集：四邊形內含一個中心點
  $[(0.0, 0.0), (4.0, 0.0), (4.0, 4.0), (0.0, 4.0), (2.0, 2.0)]$
  - 凸包外頂點應為 4 個角點：$[(0, 0), (4, 0), (4, 4), (0, 4)]$。
  - 中心點 $(2.0, 2.0)$ 應被排除在凸包之外。
- 測試斷言頂點數為 4，且各點座標正確匹配。
