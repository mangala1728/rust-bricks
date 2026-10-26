//! # Office Document DrawingML & Shape Vector Layout Micro-Bricks (辦公文書向量圖形與排版微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 OpenXML DrawingML EMU (English Metric Units) 與點數 pt 雙向高精度換算、
//! 60000分之1度旋轉角轉弧度、形狀 2D 仿射矩陣與旋轉包圍盒 (AABB)、
//! 流程圖正交連接符 (Orthogonal Connector) 轉折點解算、漸層停靠點 (Gradient Stop) 顏色插值、
//! 文字框自適應字體縮放、多形狀等距均分間距計算、投射陰影偏移解算。

/// ## Map: OpenXML DrawingML EMU (English Metric Units) 轉點數 (Points, pt)
/// 標準：1 inch = 914400 EMUs = 72 pt -> 1 pt = 12700 EMUs
/// @outputs: 1
/// @domain: emu [-1e14, 1e14]
pub fn doc_emu_to_points(emu: i64) -> f64 {
    (emu as f64) / 12700.0
}

/// ## Map: 點數 (Points, pt) 轉 OpenXML DrawingML EMU
/// @outputs: 1
/// @domain: pt [-1e9, 1e9]
pub fn doc_points_to_emu(pt: f64) -> i64 {
    if !pt.is_finite() {
        return 0;
    }
    (pt * 12700.0).round() as i64
}

/// ## Convert: DrawingML 60000分之一度整數旋轉角轉弧度
/// OpenXML 規範 rot 屬性為 60000th of a degree (360 度 = 21600000)
/// @outputs: 1
/// @domain: rot_60000th_deg [-216000000, 216000000]
pub fn doc_drawing_rotation_to_radians(rot_60000th_deg: i32) -> f64 {
    const PI: f64 = 3.141592653589793;
    let deg = (rot_60000th_deg as f64) / 60000.0;
    deg * (PI / 180.0)
}

/// ## Calc: 向量圖形 2D 仿射變換矩陣解算 (Shape Affine Matrix [a, b, c, d, e, f])
/// 包含旋轉 (rot_rad)、水平翻轉 (flip_h)、垂直翻轉 (flip_v) 及平移 (x, y)
/// @outputs: 1
/// @domain: x_pt [-1e6, 1e6], y_pt [-1e6, 1e6], rot_rad [-100.0, 100.0]
pub fn doc_shape_affine_matrix_step(
    x_pt: f64,
    y_pt: f64,
    rot_rad: f64,
    flip_h: bool,
    flip_v: bool,
) -> [f64; 6] {
    if !x_pt.is_finite() || !y_pt.is_finite() || !rot_rad.is_finite() {
        return [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    }
    let cos_t = rot_rad.cos();
    let sin_t = rot_rad.sin();
    let sx = if flip_h { -1.0 } else { 1.0 };
    let sy = if flip_v { -1.0 } else { 1.0 };

    [
        sx * cos_t,
        sx * sin_t,
        sy * -sin_t,
        sy * cos_t,
        x_pt,
        y_pt,
    ]
}

/// ## Calc: 旋轉矩形圖形之軸對齊外接包圍盒 (Rotated Shape AABB)
/// 回傳 (min_x, min_y, max_x, max_y)
/// @outputs: 1
/// @domain: cx [-1e6, 1e6], cy [-1e6, 1e6], w [0.0, 1e6], h [0.0, 1e6], rot_rad [-100.0, 100.0]
pub fn doc_shape_rotated_aabb(
    cx: f64,
    cy: f64,
    w: f64,
    h: f64,
    rot_rad: f64,
) -> (f64, f64, f64, f64) {
    if !cx.is_finite() || !cy.is_finite() || !w.is_finite() || !h.is_finite() || !rot_rad.is_finite() || w <= 0.0 || h <= 0.0 {
        return (cx, cy, cx, cy);
    }
    let cos_t = rot_rad.cos().abs();
    let sin_t = rot_rad.sin().abs();
    let half_bb_w = 0.5 * (w * cos_t + h * sin_t);
    let half_bb_h = 0.5 * (w * sin_t + h * cos_t);

    (cx - half_bb_w, cy - half_bb_h, cx + half_bb_w, cy + half_bb_h)
}

/// ## Route: 流程圖正交連接線折點解算 (Orthogonal Connector Midpoint)
/// 給定起點與終點，回傳 L 型折線轉折點座標
/// @outputs: 1
pub fn doc_orthogonal_connector_corner(
    start: (f64, f64),
    end: (f64, f64),
    is_horizontal_first: bool,
) -> (f64, f64) {
    if !start.0.is_finite() || !start.1.is_finite() || !end.0.is_finite() || !end.1.is_finite() {
        return start;
    }
    if is_horizontal_first {
        (end.0, start.1) // 先水平移動至 end.0，再垂直移動至 end.1
    } else {
        (start.0, end.1) // 先垂直移動至 end.1，再水平移動至 end.0
    }
}

/// ## Lerp: 漸層填色停止點線性顏色插值 (Gradient Stop sRGB Linear Lerp)
/// @outputs: 1
/// @domain: t [0.0, 1.0]
pub fn doc_gradient_stop_lerp_rgb(c1: [u8; 3], c2: [u8; 3], t: f64) -> [u8; 3] {
    if !t.is_finite() {
        return c1;
    }
    let clamped_t = t.clamp(0.0, 1.0);
    let r = (c1[0] as f64) + ((c2[0] as f64) - (c1[0] as f64)) * clamped_t;
    let g = (c1[1] as f64) + ((c2[1] as f64) - (c1[1] as f64)) * clamped_t;
    let b = (c1[2] as f64) + ((c2[2] as f64) - (c1[2] as f64)) * clamped_t;

    [r.round() as u8, g.round() as u8, b.round() as u8]
}

/// ## Scale: 文字框超界自動縮小字體縮放比 (Text Box Auto-fit Font Scale)
/// 依據估算文字總面積與文字框面積計算字型縮小比 (0.1..1.0)
/// @outputs: 1
/// @domain: box_w [1.0, 1e6], box_h [1.0, 1e6], base_font_size [1.0, 1000.0]
pub fn doc_textbox_autofit_font_scale(
    char_count: usize,
    box_w: f64,
    box_h: f64,
    base_font_size: f64,
) -> f64 {
    if char_count == 0 || !box_w.is_finite() || !box_h.is_finite() || !base_font_size.is_finite() || box_w <= 0.0 || box_h <= 0.0 || base_font_size <= 0.0 {
        return 1.0;
    }
    // 經驗幾何估計：單字符平均寬度約為 0.55 * font_size，高度約為 1.2 * font_size
    let char_area = 0.55 * 1.2 * base_font_size * base_font_size;
    let total_text_area = (char_count as f64) * char_area;
    let box_area = box_w * box_h;

    if total_text_area <= box_area {
        1.0
    } else {
        let scale = (box_area / total_text_area).sqrt();
        scale.clamp(0.2, 1.0)
    }
}

/// ## Distribute: 多圖形水平均勻間距分攤計算 (Distribute Elements Spacing)
/// @outputs: 1
/// @domain: total_span [0.0, 1e6]
pub fn doc_distribute_elements_spacing(total_span: f64, element_sizes: &[f64]) -> Option<f64> {
    if !total_span.is_finite() || element_sizes.len() < 2 || total_span <= 0.0 {
        return None;
    }
    let mut sum_sizes = 0.0;
    for &sz in element_sizes {
        if !sz.is_finite() || sz < 0.0 { return None; }
        sum_sizes += sz;
    }
    if sum_sizes > total_span {
        return None; // 圖形總寬度超出跨度
    }
    let gap_count = (element_sizes.len() - 1) as f64;
    Some((total_span - sum_sizes) / gap_count)
}

/// ## Calc: 投射陰影位移量解算 (Drop Shadow Offset)
/// 公式：dx = distance * cos(angle), dy = distance * sin(angle)
/// @outputs: 1
/// @domain: distance_pt [0.0, 1000.0], angle_rad [-100.0, 100.0]
pub fn doc_drop_shadow_offset(distance_pt: f64, angle_rad: f64) -> (f64, f64) {
    if !distance_pt.is_finite() || !angle_rad.is_finite() || distance_pt < 0.0 {
        return (0.0, 0.0);
    }
    (distance_pt * angle_rad.cos(), distance_pt * angle_rad.sin())
}
