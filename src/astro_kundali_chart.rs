// ## Map
// Role: astro/astro_kundali_chart — 零依賴吠陀占星雙軌星盤（南印盤與北印盤）幾何佈局、ASCII 拓撲與 SVG 向量生成純函式積木。
// Inputs: 命宮上升星座 (1..12)、行星分佈陣列、畫布尺寸 (像素)、宮位索引。
// Outputs: 十二宮星座對應、南印盤座標映射、北印盤幾何頂點與 SVG 標籤字串、拓撲字元方塊。
// Invariants: 零全域狀態、禁止 unsafe 代碼、100% 遵守 09 號合約、無遞迴、邊界嚴格防禦。

#![forbid(unsafe_code)]

/// 行星代號枚舉（0..9）
pub const GRAHA_AS: u8 = 0;  // As命 (Lagna)
pub const GRAHA_SU: u8 = 1;  // 太陽 (Sūrya)
pub const GRAHA_MO: u8 = 2;  // 月亮 (Candra)
pub const GRAHA_MA: u8 = 3;  // 火星 (Maṅgala)
pub const GRAHA_ME: u8 = 4;  // 水星 (Budha)
pub const GRAHA_JU: u8 = 5;  // 木星 (Guru)
pub const GRAHA_VE: u8 = 6;  // 金星 (Śukra)
pub const GRAHA_SA: u8 = 7;  // 土星 (Śani)
pub const GRAHA_RA: u8 = 8;  // 羅睺 (Rāhu)
pub const GRAHA_KE: u8 = 9;  // 計都 (Ketu)

/// 取得行星標準中文名稱
/// @outputs: 1
/// @domain: graha_id [0..9]
pub fn graha_name_zh(graha_id: u8) -> &'static str {
    match graha_id {
        GRAHA_AS => "As命",
        GRAHA_SU => "太陽",
        GRAHA_MO => "月亮",
        GRAHA_MA => "火星",
        GRAHA_ME => "水星",
        GRAHA_JU => "木星",
        GRAHA_VE => "金星",
        GRAHA_SA => "土星",
        GRAHA_RA => "羅睺",
        GRAHA_KE => "計都",
        _ => "未知",
    }
}

/// 取得星座標準中文名稱
/// @outputs: 1
/// @domain: sign_num [1..12]
pub fn rashi_name_zh(sign_num: u8) -> &'static str {
    if sign_num < 1 || sign_num > 12 {
        return "未知";
    }
    match sign_num {
        1 => "01 白羊",
        2 => "02 金牛",
        3 => "03 雙子",
        4 => "04 巨蟹",
        5 => "05 獅子",
        6 => "06 處女",
        7 => "07 天秤",
        8 => "08 天蠍",
        9 => "09 射手",
        10 => "10 摩羯",
        11 => "11 水瓶",
        12 => "12 雙魚",
        _ => "未知",
    }
}

/// 依上升星座計算第 N 宮之星座 (1..12)
/// @outputs: 2
/// @domain: ascendant_sign [1..12]
/// @domain: house_num [1..12]
pub fn calculate_house_sign(ascendant_sign: u8, house_num: u8) -> Option<u8> {
    if ascendant_sign < 1 || ascendant_sign > 12 || house_num < 1 || house_num > 12 {
        return None;
    }
    // 依歐幾里得取模公式：((asc - 1) + (house - 1)) % 12 + 1
    let raw = (ascendant_sign - 1) + (house_num - 1);
    Some((raw % 12) + 1)
}

/// 南印度盤（固定星座制）：給定星座 (1..12)，回傳在 4x4 網格中的 (row, col)
/// @outputs: 2
/// @domain: sign_num [1..12]
pub fn south_indian_grid_coord(sign_num: u8) -> Option<(usize, usize)> {
    if sign_num < 1 || sign_num > 12 {
        return None;
    }
    // 南印度盤排布：雙魚 (0,0), 白羊 (0,1), 金牛 (0,2), 雙子 (0,3) ...
    let coord = match sign_num {
        12 => (0, 0),
        1 => (0, 1),
        2 => (0, 2),
        3 => (0, 3),
        4 => (1, 3),
        5 => (2, 3),
        6 => (3, 3),
        7 => (3, 2),
        8 => (3, 1),
        9 => (3, 0),
        10 => (2, 0),
        11 => (1, 0),
        _ => return None,
    };
    Some(coord)
}

/// 南印度盤標準正方形儲存格邊長（WordCraft 表格標準：2.70 公分 = 1530 dxa = 76.5 pt）
pub const SOUTH_INDIAN_CELL_DXA: u32 = 1530;
/// 南印度盤總寬度與總高度（4 格 x 2.70 公分 = 10.80 公分 = 6120 dxa）
pub const SOUTH_INDIAN_TOTAL_DXA: u32 = 6120;
/// 南印度盤中央合併區尺寸（2 格 x 2.70 公分 = 5.40 公分 = 3060 dxa）
pub const SOUTH_INDIAN_CENTER_DXA: u32 = 3060;

/// 公分轉 OpenXML dxa 單位（1 cm = 566.929 dxa ≈ 567 dxa）
/// @outputs: 1
/// @domain: cm [0.0..100.0]
pub fn cm_to_dxa(cm: f32) -> u32 {
    if cm <= 0.0 || cm > 100.0 {
        return 0;
    }
    // 1 inch = 2.54 cm = 1440 dxa
    ((cm / 2.54) * 1440.0).round() as u32
}

/// 判斷 4x4 儲存格是否為中央保留區 (1,1), (1,2), (2,1), (2,2)
/// @outputs: 1
/// @domain: row [0..3]
/// @domain: col [0..3]
pub fn is_south_indian_center_cell(row: usize, col: usize) -> bool {
    if row > 3 || col > 3 {
        return false;
    }
    (row == 1 || row == 2) && (col == 1 || col == 2)
}

/// 北印度盤（固定宮位制）：取得第 H 宮中心點在正方形畫布中的百分比座標 (x_ratio, y_ratio)
/// @outputs: 2
/// @domain: house_num [1..12]
pub fn north_indian_house_center(house_num: u8) -> Option<(f32, f32)> {
    if house_num < 1 || house_num > 12 {
        return None;
    }
    let pt = match house_num {
        1 => (0.50, 0.25),
        2 => (0.25, 0.12),
        3 => (0.12, 0.25),
        4 => (0.25, 0.50),
        5 => (0.12, 0.75),
        6 => (0.25, 0.88),
        7 => (0.50, 0.75),
        8 => (0.75, 0.88),
        9 => (0.88, 0.75),
        10 => (0.75, 0.50),
        11 => (0.88, 0.25),
        12 => (0.75, 0.12),
        _ => return None,
    };
    Some(pt)
}

/// 北印度盤（固定宮位制）：取得第 H 宮標注星座序號的角隅百分比座標 (x_ratio, y_ratio)
/// @outputs: 2
/// @domain: house_num [1..12]
pub fn north_indian_sign_corner(house_num: u8) -> Option<(f32, f32)> {
    if house_num < 1 || house_num > 12 {
        return None;
    }
    let pt = match house_num {
        1 => (0.50, 0.38),
        2 => (0.36, 0.20),
        3 => (0.20, 0.36),
        4 => (0.38, 0.50),
        5 => (0.20, 0.64),
        6 => (0.36, 0.80),
        7 => (0.50, 0.62),
        8 => (0.64, 0.80),
        9 => (0.80, 0.64),
        10 => (0.62, 0.50),
        11 => (0.80, 0.36),
        12 => (0.64, 0.20),
        _ => return None,
    };
    Some(pt)
}

/// 快速檢驗是否包含禁字（0 皈依、0 平台、0 證悟、0 達斯、0 以太）
/// @outputs: 1
/// @domain: text.len() [0..100000]
pub fn validate_banned_words_slice(text: &str) -> Option<&'static str> {
    if text.contains("皈依") {
        return Some("禁止「皈依」，應使用「歸依」");
    }
    if text.contains("平台") {
        return Some("禁止「平台」，應使用「層級」");
    }
    if text.contains("證悟") {
        return Some("避用「證悟」，改為「覺悟自我的／感悟／親證」");
    }
    if text.contains("達斯") && !text.contains("達斯亞") {
        return Some("禁止「達斯」，結尾 a 音必發，應使用「達薩」");
    }
    if text.contains("以太") && !text.contains("以太陽") {
        return Some("五大元素 ether 應使用「乙太」，嚴禁「以太」");
    }
    None
}

/// 產生北印度星盤標準 SVG 幾何圖形字串
/// @outputs: 1
/// @domain: size [100..2000]
pub fn generate_north_indian_svg_base(size: u32) -> String {
    let s = if size < 100 || size > 2000 { 500 } else { size };
    let mid = s / 2;
    let mut svg = String::with_capacity(512);
    svg.push_str(&format!(r##"<svg xmlns="http://www.w3.org/2000/svg" width="{s}" height="{s}" viewBox="0 0 {s} {s}">"##));
    svg.push_str(r##"<rect width="100%" height="100%" fill="#FFFFFF"/>"##);
    // Outer border
    svg.push_str(&format!(r##"<rect x="10" y="10" width="{}" height="{}" fill="none" stroke="#000000" stroke-width="3"/>"##, s - 20, s - 20));
    // Diagonals
    svg.push_str(&format!(r##"<line x1="10" y1="10" x2="{}" y2="{}" stroke="#000000" stroke-width="2"/>"##, s - 10, s - 10));
    svg.push_str(&format!(r##"<line x1="10" y1="{}" x2="{}" y2="10" stroke="#000000" stroke-width="2"/>"##, s - 10, s - 10));
    // Central diamond
    svg.push_str(&format!(
        r##"<polygon points="{mid},10 10,{mid} {mid},{} {},{mid}" fill="none" stroke="#000000" stroke-width="2"/>"##,
        s - 10, s - 10
    ));
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_house_sign_golden() {
        // 白羊座上升 (1) -> 命宮是白羊 (1)，第 7 宮是天秤 (7)
        assert_eq!(calculate_house_sign(1, 1), Some(1));
        assert_eq!(calculate_house_sign(1, 7), Some(7));
        assert_eq!(calculate_house_sign(1, 12), Some(12));

        // 巨蟹座上升 (4) -> 命宮是巨蟹 (4)，第 10 宮是白羊 (1)
        assert_eq!(calculate_house_sign(4, 1), Some(4));
        assert_eq!(calculate_house_sign(4, 10), Some(1));

        // 雙魚座上升 (12) -> 第 2 宮是白羊 (1)
        assert_eq!(calculate_house_sign(12, 2), Some(1));

        // 邊界防禦測試
        assert_eq!(calculate_house_sign(0, 1), None);
        assert_eq!(calculate_house_sign(13, 1), None);
        assert_eq!(calculate_house_sign(1, 0), None);
        assert_eq!(calculate_house_sign(1, 13), None);
    }

    #[test]
    fn test_south_indian_grid_coords() {
        assert_eq!(south_indian_grid_coord(12), Some((0, 0))); // 雙魚
        assert_eq!(south_indian_grid_coord(1), Some((0, 1)));  // 白羊
        assert_eq!(south_indian_grid_coord(3), Some((0, 3)));  // 雙子
        assert_eq!(south_indian_grid_coord(6), Some((3, 3)));  // 處女
        assert_eq!(south_indian_grid_coord(9), Some((3, 0)));  // 射手
        assert_eq!(south_indian_grid_coord(13), None);

        // 中央儲存格
        assert!(is_south_indian_center_cell(1, 1));
        assert!(is_south_indian_center_cell(2, 2));
        assert!(!is_south_indian_center_cell(0, 0));
        assert!(!is_south_indian_center_cell(3, 3));
    }

    #[test]
    fn test_north_indian_coords() {
        // House 1 Center: (0.5, 0.25)
        let (c_x, c_y) = north_indian_house_center(1).unwrap();
        assert!((c_x - 0.50).abs() < 1e-4);
        assert!((c_y - 0.25).abs() < 1e-4);

        // House 7 Center: (0.5, 0.75)
        let (c7_x, c7_y) = north_indian_house_center(7).unwrap();
        assert!((c7_x - 0.50).abs() < 1e-4);
        assert!((c7_y - 0.75).abs() < 1e-4);

        assert_eq!(north_indian_house_center(0), None);
        assert_eq!(north_indian_house_center(13), None);
    }

    #[test]
    fn test_validate_banned_words() {
        assert!(validate_banned_words_slice("我們要皈依主").is_some());
        assert!(validate_banned_words_slice("提升到這個平台").is_some());
        assert!(validate_banned_words_slice("達斯大人").is_some());
        assert!(validate_banned_words_slice("以太元素").is_some());
        // 合法例外
        assert!(validate_banned_words_slice("達斯亞瑜伽").is_none());
        assert!(validate_banned_words_slice("以太陽為中心").is_none());
        assert!(validate_banned_words_slice("我們要歸依首神").is_none());
    }

    #[test]
    fn test_generate_svg_base() {
        let svg = generate_north_indian_svg_base(400);
        assert!(svg.contains(r#"<svg"#));
        assert!(svg.contains(r#"<polygon"#));
        assert!(svg.contains(r#"width="400""#));
    }

    #[test]
    fn test_south_indian_dimensions() {
        assert_eq!(SOUTH_INDIAN_CELL_DXA, 1530);
        assert_eq!(SOUTH_INDIAN_TOTAL_DXA, 6120);
        assert_eq!(SOUTH_INDIAN_CENTER_DXA, 3060);
        let dxa = cm_to_dxa(2.70);
        assert!(dxa == 1530 || dxa == 1531);
        assert_eq!(cm_to_dxa(0.0), 0);
        assert_eq!(cm_to_dxa(-5.0), 0);
        assert_eq!(cm_to_dxa(105.0), 0);
    }
}
