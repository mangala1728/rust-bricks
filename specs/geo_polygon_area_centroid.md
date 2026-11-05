# 模組名稱: 多邊形鞋帶公式面積與幾何質心計算器 (Polygon Shoelace Area & Centroid)

## 1. 目標定位
- 目標模組檔: `src/geo_polygon_area_centroid.rs`
- 目標測試檔: `tests/test_geo_polygon_area_centroid.rs`
- 功能簡述: 利用高斯格林公式（鞋帶演算法 Shoelace formula），高數值穩定度計算任意平面簡單多邊形之有向面積（Signed Area）與幾何質心座標（Centroid $C_x, C_y$）。

## 2. 09 合約介面定義
```rust
// ## Map
// Role: 2D polygon shoelace signed area and centroid coordinates
// Inputs: vertices: &[(f64, f64)]
// Outputs: Option<f64> or Option<(f64, f64)>
// Invariants: 0 external dependencies, line 1 domain guard, functions <= 60 lines

/// @outputs: 1
/// @domain: vertices.len() >= 3, coordinates are finite
pub fn geo_polygon_signed_area(vertices: &[(f64, f64)]) -> Option<f64>;

/// @outputs: 2
/// @domain: vertices.len() >= 3, |area| > 1e-12
pub fn geo_polygon_centroid(vertices: &[(f64, f64)]) -> Option<(f64, f64)>;
```

## 3. 邊界與異常條件 (Domain Guards)
- 若 `vertices.len() < 3` 或頂點包含非有限數，立即回傳 `None`。
- 鞋帶公式：
  $$A = \frac{1}{2} \sum_{i=0}^{N-1} (x_i y_{i+1} - x_{i+1} y_i)$$
  $$C_x = \frac{1}{6A} \sum_{i=0}^{N-1} (x_i + x_{i+1}) (x_i y_{i+1} - x_{i+1} y_i)$$
  $$C_y = \frac{1}{6A} \sum_{i=0}^{N-1} (y_i + y_{i+1}) (x_i y_{i+1} - x_{i+1} y_i)$$
  （其中頂點 $N$ 閉合循環回第 $0$ 點）。
- `geo_polygon_centroid`: 若 $|A| \le 10^{-12}$（退化為共線點或無面積多邊形），回傳 `None`。

## 4. 確定性黃金值測試要求
- 單位正方形 `[(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]`（逆時針）：
  - 有向面積 $A = 1.0$
  - 質心 $C = (0.5, 0.5)$
- 直角三角形 `[(0.0, 0.0), (3.0, 0.0), (0.0, 3.0)]`：
  - 有向面積 $A = 4.5$
  - 質心 $C = (1.0, 1.0)$

### 機器可驗證（```golden，由 cargo run -- farm-golden 產生 tests/golden_geo_polygon_area_centroid.rs）
```golden
let sq = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
(geo_polygon_signed_area(&sq).unwrap() - 1.0).abs() < 1e-10
let (cx, cy) = geo_polygon_centroid(&sq).unwrap();
(cx - 0.5).abs() < 1e-10 && (cy - 0.5).abs() < 1e-10
let tri = [(0.0, 0.0), (3.0, 0.0), (0.0, 3.0)];
(geo_polygon_signed_area(&tri).unwrap() - 4.5).abs() < 1e-10
let (tcx, tcy) = geo_polygon_centroid(&tri).unwrap();
(tcx - 1.0).abs() < 1e-10 && (tcy - 1.0).abs() < 1e-10
geo_polygon_signed_area(&[(0.0, 0.0), (1.0, 0.0)]).is_none()
```
