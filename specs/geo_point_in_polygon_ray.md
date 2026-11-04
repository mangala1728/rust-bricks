# 規格書：射線交叉法任意多邊形內外點判別 (Point in Polygon Ray-Casting)

## 模組名稱
`geo_point_in_polygon_ray`

## 業務目標與適用場景
在電子圍欄 (Geofencing)、無人機禁航區檢查、GIS 地圖點位查詢與碰撞邊界測試中，判斷任意 2D 平面多邊形（凸多邊形或凹多邊形）是否包含指定點是核心演算法。
本模組提供零外部依賴、純 `std` 實作的射線法 (Ray-Casting Algorithm) 點在多邊形內外判定微積木。

## 09 號合約邊界規範

### 函式 1：射線法判定二維點是否在多邊形內部 (Point in Polygon)
- **函式簽名**：
  ```rust
  /// @outputs: 1
  pub fn point_in_polygon_ray(
      point: (f64, f64),
      polygon: &[(f64, f64)],
  ) -> Option<bool>
  ```
- **運算過程**：
  - 若 `polygon.len() < 3` 回傳 `None`（無效多邊形）。
  - 邊界檢查：若 `point` 或 `polygon` 中任意座標為 NaN 或非有限數，回傳 `None`。
  - 令 `inside = false`。
  - 歷遍多邊形各邊 $(i, j)$，其中 $j = (i + 1) \% N$：
    - 令 $(x_i, y_i) = \text{polygon}[i]$, $(x_j, y_j) = \text{polygon}[j]$。
    - 若水平射線穿過邊：`((y_i > py) != (y_j > py))`
      - 且交點位於點右側：$px < \frac{(x_j - x_i)(py - y_i)}{y_j - y_i} + x_i$
      - 則反轉 `inside = !inside`。
  - 回傳 `Some(inside)`。

## 黃金測試向量 (Golden Vectors)
- 凹多邊形（L 型區域）：
  $[(0.0, 0.0), (6.0, 0.0), (6.0, 3.0), (3.0, 3.0), (3.0, 6.0), (0.0, 6.0)]$
  - 點 $(1.0, 1.0)$：位於 L 型內部下底，應回傳 `Some(true)`。
  - 點 $(4.0, 4.0)$：位於 L 型外部缺口，應回傳 `Some(false)`。
  - 點 $(10.0, 10.0)$：遠離多邊形，應回傳 `Some(false)`。
- 邊界防禦：
  - 點數小於 3（退化線段）：回傳 `None`。

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_geo_point_in_polygon_ray.rs）
```golden
// 出處：規格向量，L 型凹多邊形；射線法 Python 獨立驗算
let l = [(0.0, 0.0), (6.0, 0.0), (6.0, 3.0), (3.0, 3.0), (3.0, 6.0), (0.0, 6.0)];
point_in_polygon_ray((1.0, 1.0), &l) == Some(true)
point_in_polygon_ray((4.0, 4.0), &l) == Some(false)
point_in_polygon_ray((10.0, 10.0), &l) == Some(false)
// 出處：規格邊界防禦（點數 < 3、NaN／非有限座標回傳 None）
point_in_polygon_ray((1.0, 1.0), &[(0.0, 0.0), (6.0, 0.0)]).is_none()
point_in_polygon_ray((f64::NAN, 1.0), &l).is_none()
point_in_polygon_ray((1.0, 1.0), &[(0.0, 0.0), (6.0, f64::INFINITY), (0.0, 6.0)]).is_none()
```
