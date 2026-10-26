//! # Document Layout & Typography Micro-Bricks (Word/PPT 文書排版與字體控制微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 點數 (pt) 與 Twips (1/20 pt) 雙向換算、TrueType/OpenType UPM 字體單元像素縮放、
//! 段落首行縮排與左右邊距、行高倍數換算、齊行 (Justification) 空格分攤、
//! Knuth-Plass 排版折行惡化度 (Badness)、表格欄寬等比自動配額、PPT 幻燈片等比自適應居中、斷字連字號懲罰估算。

/// ## Convert: 印刷點數 (pt) 轉 Word/RTF Twips (1 pt = 20 twips)
/// @outputs: 1
/// @domain: pt [-10000.0, 10000.0]
pub fn doc_pt_to_twips(pt: f64) -> i32 {
    if !pt.is_finite() {
        return 0;
    }
    let clamped_pt = pt.clamp(-10000.0, 10000.0);
    (clamped_pt * 20.0).round() as i32
}

/// ## Convert: Word/RTF Twips 轉印刷點數 (pt)
/// @outputs: 1
/// @domain: twips [-200000, 200000]
pub fn doc_twips_to_pt(twips: i32) -> f64 {
    let clamped_twips = twips.clamp(-200000, 200000);
    (clamped_twips as f64) / 20.0
}

/// ## Scale: 字體設計單元 (UPM: 1000/2048) 依指定磅數與 DPI 縮放為設備像素
/// pixel = glyph_units * (font_size_pt * dpi) / (72.0 * upm)
/// @outputs: 1
/// @domain: glyph_units [-32768.0, 32767.0]
/// @domain: font_size_pt [1.0, 1000.0]
/// @domain: dpi [10.0, 2400.0]
pub fn doc_font_scale_em_px(glyph_units: f64, upm: u16, font_size_pt: f64, dpi: f64) -> f64 {
    if !glyph_units.is_finite() || upm == 0 || !font_size_pt.is_finite() ||
       font_size_pt <= 0.0 || !dpi.is_finite() || dpi <= 0.0 {
        return 0.0;
    }
    let scale = (font_size_pt * dpi) / (72.0 * (upm as f64));
    glyph_units * scale
}

/// ## Map: Word 段落縮排換算為 Twips (首行縮排、左側縮排、右側縮排)
/// @outputs: 3
/// @domain: first_line_indent_pt [-500.0, 500.0]
/// @domain: left_indent_pt [0.0, 1000.0]
/// @domain: right_indent_pt [0.0, 1000.0]
pub fn doc_paragraph_indentation_twips(
    first_line_indent_pt: f64,
    left_indent_pt: f64,
    right_indent_pt: f64,
) -> (i32, i32, i32) {
    if !first_line_indent_pt.is_finite() || !left_indent_pt.is_finite() || !right_indent_pt.is_finite() {
        return (0, 0, 0);
    }
    let first = doc_pt_to_twips(first_line_indent_pt);
    let left = doc_pt_to_twips(left_indent_pt.max(0.0));
    let right = doc_pt_to_twips(right_indent_pt.max(0.0));
    (first, left, right)
}

/// ## Calc: Word 精確行距 (Exact Line Spacing) 換算為 Twips
/// @outputs: 1
/// @domain: font_size_pt [1.0, 500.0]
/// @domain: line_spacing_multiple [0.5, 10.0]
pub fn doc_line_spacing_exact_twips(font_size_pt: f64, line_spacing_multiple: f64) -> i32 {
    if !font_size_pt.is_finite() || !line_spacing_multiple.is_finite() ||
       font_size_pt <= 0.0 || line_spacing_multiple <= 0.0 {
        return 0;
    }
    let total_pt = font_size_pt * line_spacing_multiple;
    doc_pt_to_twips(total_pt)
}

/// ## Distribute: 齊行 (Justified) 剩餘空白寬度均勻分攤到每個空格
/// @outputs: 1
/// @domain: line_width_px [1.0, 10000.0]
/// @domain: words_width_px [0.0, 10000.0]
pub fn doc_text_justified_space_distribution(
    line_width_px: f64,
    words_width_px: f64,
    num_spaces: usize,
) -> Option<f64> {
    if !line_width_px.is_finite() || !words_width_px.is_finite() ||
       line_width_px <= 0.0 || words_width_px < 0.0 || num_spaces == 0 {
        return None;
    }
    let surplus = line_width_px - words_width_px;
    if surplus < 0.0 {
        // 內容超出邊界 (溢出)
        return None;
    }
    Some(surplus / (num_spaces as f64))
}

/// ## Measure: Knuth-Plass 排版折行惡化度評分 (Badness Rating)
/// 評分越低排版越均勻美觀，溢出則回傳 100,000 (不可接受)
/// @outputs: 1
/// @domain: line_width [1.0, 10000.0]
/// @domain: content_width [0.0, 10000.0]
pub fn doc_knuth_plass_badness(line_width: f64, content_width: f64) -> u32 {
    if !line_width.is_finite() || !content_width.is_finite() || line_width <= 0.0 || content_width < 0.0 {
        return 100_000;
    }
    if content_width > line_width {
        return 100_000;
    }
    let ratio = (line_width - content_width) / line_width;
    let badness = 100.0 * ratio * ratio * ratio;
    badness.round().clamp(0.0, 10000.0) as u32
}

/// ## Fit: 表格欄寬依內容等比分配 (Proportional Column Width Auto-Fit)
/// @outputs: 1
/// @domain: table_available_width [1.0, 100000.0]
pub fn doc_table_column_proportional_width(
    col_contents_width: &[f64],
    table_available_width: f64,
    out_widths: &mut [f64],
) -> bool {
    if col_contents_width.is_empty() || out_widths.len() < col_contents_width.len() ||
       !table_available_width.is_finite() || table_available_width <= 0.0 {
        return false;
    }
    let mut sum_content = 0.0f64;
    for &w in col_contents_width {
        if !w.is_finite() || w < 0.0 { return false; }
        sum_content += w;
    }
    let n = col_contents_width.len();
    if sum_content < 1e-6 {
        // 若全部欄位寬度為零，均等分配
        let equal_w = table_available_width / (n as f64);
        for i in 0..n {
            out_widths[i] = equal_w;
        }
        return true;
    }
    let scale = table_available_width / sum_content;
    for i in 0..n {
        out_widths[i] = col_contents_width[i] * scale;
    }
    true
}

/// ## Fit: PPT 幻燈片母片物件等比自適應縮放與居中 (Aspect Fit & Centering)
/// 回傳 (target_x, target_y, target_width, target_height)
/// @outputs: 4
/// @domain: orig_w [1.0, 10000.0]
/// @domain: orig_h [1.0, 10000.0]
/// @domain: slide_w [1.0, 10000.0]
/// @domain: slide_h [1.0, 10000.0]
pub fn doc_ppt_slide_fit_aspect_ratio(
    orig_w: f64,
    orig_h: f64,
    slide_w: f64,
    slide_h: f64,
) -> (f64, f64, f64, f64) {
    if !orig_w.is_finite() || !orig_h.is_finite() || !slide_w.is_finite() || !slide_h.is_finite() ||
       orig_w <= 0.0 || orig_h <= 0.0 || slide_w <= 0.0 || slide_h <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let scale_w = slide_w / orig_w;
    let scale_h = slide_h / orig_h;
    let scale = scale_w.min(scale_h);

    let target_w = orig_w * scale;
    let target_h = orig_h * scale;
    let target_x = (slide_w - target_w) * 0.5;
    let target_y = (slide_h - target_h) * 0.5;
    (target_x, target_y, target_w, target_h)
}

/// ## Penalty: 排版斷字連字號斷裂懲罰成本 (Hyphenation Break Penalty)
/// @outputs: 1
pub fn doc_hyphenation_break_cost(
    word_len: usize,
    break_pos: usize,
    is_consonant_cluster: bool,
) -> u32 {
    if word_len < 4 || break_pos < 2 || break_pos > word_len.saturating_sub(2) {
        // 拒絕過短單詞或斷裂點靠近詞首/詞尾 (< 2 字元)
        return 10000;
    }
    let mut cost = 50u32;
    // 若拆分在子音叢集邊界，更符合語音自然停頓，降低懲罰
    if is_consonant_cluster {
        cost = cost.saturating_sub(20);
    }
    // 偏向中段斷字
    let mid = word_len / 2;
    let dist_from_mid = (break_pos as isize - mid as isize).abs() as u32;
    cost += dist_from_mid * 5;
    cost
}
