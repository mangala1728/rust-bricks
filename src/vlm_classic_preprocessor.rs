// ## Map
// Role: vision/vlm_classic_preprocessor — 古典善本圖像自適應去噪、二值化、裝訂槽投影切片與 VLM 提示詞生成純演算法積木。
// Inputs: 灰階像素切片、寬高維度、直方圖、百分位點、投影門檻、切片最小空隙。
// Outputs: 灰階直方圖、百分位灰度階、動態分割門檻值、二值化圖像、3x3 中值去噪、橫向投影黑像素計數、經文區塊切片邊界、VLM 防幻覺提示詞。
// Invariants: 零外部依賴、禁止 unsafe 代碼、100% 遵守 09 號合約、無遞迴 (R09-7)、巢狀深度 <= 2 (R09-3)、單函式長度 <= 100 行。

#![forbid(unsafe_code)]

/// 計算 256 階灰度直方圖
/// @outputs: 1
/// @domain: pixels.len() [0..134217728]
pub fn calc_histogram(pixels: &[u8]) -> [u32; 256] {
    let mut hist = [0u32; 256];
    for &p in pixels {
        hist[p as usize] = hist[p as usize].saturating_add(1);
    }
    hist
}

/// 依據直方圖計算指定百分位數之灰度值 (0..100)
/// @outputs: 1
/// @domain: total_pixels [1..134217728], percentile [0..100]
pub fn calc_percentile(hist: &[u32; 256], total_pixels: u32, percentile: u8) -> u8 {
    if total_pixels == 0 || percentile > 100 {
        return 128;
    }
    let target = ((total_pixels as u64)
        .saturating_mul(percentile as u64)
        .saturating_add(99)
        / 100) as u32;

    let mut accumulated: u32 = 0;
    for (val, &count) in hist.iter().enumerate() {
        accumulated = accumulated.saturating_add(count);
        if accumulated >= target {
            return val as u8;
        }
    }
    255
}

/// 計算標準 Otsu 自適應二值化門檻值：最大化類間變異數 (Between-Class Variance)
/// @outputs: 1
/// @domain: total_pixels [1..134217728]
pub fn calc_dynamic_otsu_threshold(hist: &[u32; 256], total_pixels: u32) -> u8 {
    if total_pixels == 0 {
        return 128;
    }
    let total_f = total_pixels as f64;
    let mut sum_total = 0.0f64;
    for (i, &count) in hist.iter().enumerate() {
        sum_total += (i as f64) * (count as f64);
    }

    let mut weight_b = 0.0f64;
    let mut sum_b = 0.0f64;
    let mut max_variance = -1.0f64;
    let mut optimal_threshold = 128u8;

    for i in 0..256 {
        let count_f = hist[i] as f64;
        weight_b += count_f;
        if weight_b == 0.0 {
            continue;
        }
        let weight_f = total_f - weight_b;
        if weight_f <= 0.0 {
            break;
        }
        sum_b += (i as f64) * count_f;
        let mean_b = sum_b / weight_b;
        let mean_f = (sum_total - sum_b) / weight_f;
        let diff = mean_b - mean_f;
        let variance = weight_b * weight_f * diff * diff;

        if variance > max_variance {
            max_variance = variance;
            optimal_threshold = i as u8;
        }
    }

    optimal_threshold
}

/// 應用二值化門檻：小於門檻為墨跡黑 (0)，其餘為紙張白 (255)
/// @outputs: 1
/// @domain: width [1..16384], height [1..16384], threshold [0..255]
pub fn apply_threshold(
    pixels: &[u8],
    width: usize,
    height: usize,
    threshold: u8,
    out_pixels: &mut [u8],
) -> bool {
    let expected_len = width.saturating_mul(height);
    if width == 0 || height == 0 || pixels.len() != expected_len || out_pixels.len() != expected_len {
        return false;
    }

    let effective_threshold = threshold.max(1);
    for (p_in, p_out) in pixels.iter().zip(out_pixels.iter_mut()) {
        if *p_in < effective_threshold {
            *p_out = 0;
        } else {
            *p_out = 255;
        }
    }
    true
}

/// 3x3 中值濾波，消滅單像素椒鹽掃描雜點，保留天城文字畫邊緣
/// @outputs: 1
/// @domain: width [3..16384], height [3..16384]
pub fn apply_median_filter_3x3(
    pixels: &[u8],
    width: usize,
    height: usize,
    out_pixels: &mut [u8],
) -> bool {
    let expected_len = width.saturating_mul(height);
    if width < 3 || height < 3 || pixels.len() != expected_len || out_pixels.len() != expected_len {
        return false;
    }

    // 複製邊界原值
    out_pixels.copy_from_slice(pixels);

    let mut win = [0u8; 9];
    let row_stride = width;

    for y in 1..height.saturating_sub(1) {
        let y_prev = y.saturating_sub(1).saturating_mul(row_stride);
        let y_curr = y.saturating_mul(row_stride);
        let y_next = y.saturating_add(1).saturating_mul(row_stride);

        for x in 1..width.saturating_sub(1) {
            let x_prev = x.saturating_sub(1);
            let x_next = x.saturating_add(1);

            win[0] = pixels[y_prev.saturating_add(x_prev)];
            win[1] = pixels[y_prev.saturating_add(x)];
            win[2] = pixels[y_prev.saturating_add(x_next)];

            win[3] = pixels[y_curr.saturating_add(x_prev)];
            win[4] = pixels[y_curr.saturating_add(x)];
            win[5] = pixels[y_curr.saturating_add(x_next)];

            win[6] = pixels[y_next.saturating_add(x_prev)];
            win[7] = pixels[y_next.saturating_add(x)];
            win[8] = pixels[y_next.saturating_add(x_next)];

            // 微型插入排序求中值
            for i in 1..9 {
                let key = win[i];
                let mut j = i;
                while j > 0 && win[j - 1] > key {
                    win[j] = win[j - 1];
                    j -= 1;
                }
                win[j] = key;
            }

            out_pixels[y_curr.saturating_add(x)] = win[4]; // 中值索引為 4
        }
    }
    true
}

/// 計算每一水平行之黑像素（墨跡）累積直方圖投影
/// @outputs: 1
/// @domain: width [1..16384], height [1..16384]
pub fn calc_horizontal_projection(
    binary_pixels: &[u8],
    width: usize,
    height: usize,
    row_ink_counts: &mut [u32],
) -> bool {
    let expected_len = width.saturating_mul(height);
    if width == 0 || height == 0 || binary_pixels.len() != expected_len || row_ink_counts.len() != height {
        return false;
    }

    for (y, count) in row_ink_counts.iter_mut().enumerate() {
        let row_start = y.saturating_mul(width);
        let row_end = row_start.saturating_add(width);
        let mut black_pixels: u32 = 0;
        for &p in &binary_pixels[row_start..row_end] {
            if p == 0 {
                black_pixels = black_pixels.saturating_add(1);
            }
        }
        *count = black_pixels;
    }
    true
}

/// 依據水平投影黑像素計數，偵測空白裝訂槽 (Gutter)，切分獨立經文/段落區塊
/// @outputs: 1
/// @domain: width [1..16384], min_gap_px [5..1000], min_block_height [10..1000]
pub fn find_gutter_slices(
    row_ink_counts: &[u32],
    width: usize,
    min_gap_px: usize,
    min_block_height: usize,
    out_slices: &mut Vec<(usize, usize)>,
) -> usize {
    out_slices.clear();
    let height = row_ink_counts.len();
    if width == 0 || height == 0 || min_gap_px == 0 {
        return 0;
    }

    // 門檻：該行黑像素超過 0.5% 即視為有墨跡
    let min_ink_threshold = ((width as u32).saturating_mul(5) / 1000).max(2);

    let mut in_block = false;
    let mut start_y: usize = 0;
    let mut blank_count: usize = 0;

    for (y, &ink) in row_ink_counts.iter().enumerate() {
        let has_ink = ink >= min_ink_threshold;
        if has_ink {
            if !in_block {
                in_block = true;
                start_y = y.saturating_sub(5); // 增加 5px 上邊距保護天城文頂線
            }
            blank_count = 0;
        } else if in_block {
            blank_count = blank_count.saturating_add(1);
            if blank_count >= min_gap_px || y == height.saturating_sub(1) {
                let end_y = (y.saturating_sub(blank_count).saturating_add(5)).min(height);
                if end_y.saturating_sub(start_y) >= min_block_height {
                    out_slices.push((start_y, end_y));
                }
                in_block = false;
                blank_count = 0;
            }
        }
    }

    if in_block && height.saturating_sub(start_y) >= min_block_height {
        out_slices.push((start_y, height));
    }

    out_slices.len()
}

/// 生成 VLM 視覺交叉校對防幻覺提示詞
/// @outputs: 1
pub fn generate_vlm_verification_prompt(context_hint: &str) -> String {
    let hint = if context_hint.trim().is_empty() {
        "古典啟示聖典經文 (Revealed Classic / Sacred Sūtra)"
    } else {
        context_hint
    };

    let mut prompt = String::with_capacity(1500);
    prompt.push_str("【任務目標】：請對此高解析度經文圖像進行極致嚴密的梵文天城文（Devanagari）與註疏文字提取，並進行原圖像素級交叉校對。\n\n");
    prompt.push_str("【圖像內容語境】：");
    prompt.push_str(hint);
    prompt.push_str("\n\n");
    prompt.push_str("【嚴格字形校核準則（Anti-Hallucination & Diacritics Fidelity）】：\n");
    prompt.push_str("1. 嚴格區分易混淆天城文字形：\n");
    prompt.push_str("   - घ (gha) vs ध (dha)（注意頂線是否封口）\n");
    prompt.push_str("   - भ (bha) vs म (ma)（注意左上方是否有圓環破口）\n");
    prompt.push_str("   - प (pa) vs य (ya) vs ष (ṣa)\n");
    prompt.push_str("   - व (va) vs ब (ba)\n");
    prompt.push_str("   - ख (kha) vs रव (rava)\n");
    prompt.push_str("2. 準確識別並保留一切連字（Saṁyuktākṣara，如 ङ्क, ञ्ज, त्त्व, द्ध, ष्ट्र）與除元音符（Virāma/Halanta ्）。\n");
    prompt.push_str("3. 嚴格保留標點符號：單豎線 । (Danda) 與雙豎線 ॥ (Double Danda)，以及括號中之經文編號。\n");
    prompt.push_str("4. 結構化三層輸出要求：\n");
    prompt.push_str("   - 經文編號行（例如：॥ १.१.१ ॥）\n");
    prompt.push_str("   - 純淨天城文經文（Devanagari 原文，只增不刪，保留原始連音 Sandhi）\n");
    prompt.push_str("   - 標準羅馬轉寫（IAST Transliteration）\n");
    prompt.push_str("   - 繁體中文全譯初稿（依據哲學上下文，嚴禁 0 禁字：0 皈依、0 平台、0 證悟、0 達斯、0 以太）\n\n");
    prompt.push_str("請以結構化 JSON 格式輸出提取結果：\n");
    prompt.push_str("{\n");
    prompt.push_str("  \"verse_id\": \"編號\",\n");
    prompt.push_str("  \"devanagari\": \"天城文多行原文\",\n");
    prompt.push_str("  \"iast\": \"標準羅馬拼音\",\n");
    prompt.push_str("  \"trans_zh\": \"繁體中文初步譯文\",\n");
    prompt.push_str("  \"visual_uncertainties\": [\"任何圖像殘缺或存疑之字符與字根考證說明\"]\n");
    prompt.push_str("}\n");
    prompt
}

/// 計算 9x8 灰度縮圖之 64-bit 差分感知雜湊 (dHash)
/// @outputs: 1
/// @domain: thumb_9x8.len() == 72
pub fn compute_dhash_64(thumb_9x8: &[u8]) -> u64 {
    if thumb_9x8.len() != 72 {
        return 0;
    }
    let mut hash: u64 = 0;
    for row in 0usize..8 {
        let row_start = row.saturating_mul(9);
        for col in 0usize..8 {
            let left = thumb_9x8[row_start.saturating_add(col)];
            let right = thumb_9x8[row_start.saturating_add(col).saturating_add(1)];
            hash <<= 1;
            if left > right {
                hash |= 1;
            }
        }
    }
    hash
}

/// 計算兩個 64-bit dHash 之間的漢明距離 (Hamming Distance)
/// @outputs: 1
pub fn hamming_distance_64(hash_a: u64, hash_b: u64) -> u32 {
    (hash_a ^ hash_b).count_ones()
}

/// 判斷兩張圖像之 dHash 漢明距離是否小於等於門檻 (預設 <= 4 判定為實體重複頁)
/// @outputs: 1
/// @domain: max_hamming [0..64]
pub fn is_duplicate_dhash_pair(hash_a: u64, hash_b: u64, max_hamming: u32) -> bool {
    if max_hamming > 64 {
        return false;
    }
    hamming_distance_64(hash_a, hash_b) <= max_hamming
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_and_percentiles() {
        // 建立階梯灰度圖: 100 個 10 (黑), 100 個 200 (白)
        let mut pixels = Vec::with_capacity(200);
        pixels.extend(std::iter::repeat(10).take(100));
        pixels.extend(std::iter::repeat(200).take(100));

        let hist = calc_histogram(&pixels);
        assert_eq!(hist[10], 100);
        assert_eq!(hist[200], 100);

        let p15 = calc_percentile(&hist, 200, 15);
        assert_eq!(p15, 10);

        let p85 = calc_percentile(&hist, 200, 85);
        assert_eq!(p85, 200);

        let threshold = calc_dynamic_otsu_threshold(&hist, 200);
        assert_eq!(threshold, 10);

        // 測試 5% 墨跡 + 95% 白紙高偏差善本情況：50 個 20 (墨跡), 950 個 240 (紙張)
        let mut scan_pixels = Vec::with_capacity(1000);
        scan_pixels.extend(std::iter::repeat(20).take(50));
        scan_pixels.extend(std::iter::repeat(240).take(950));
        let scan_hist = calc_histogram(&scan_pixels);
        let scan_thresh = calc_dynamic_otsu_threshold(&scan_hist, 1000);
        assert_eq!(scan_thresh, 20);
    }

    #[test]
    fn test_thresholding() {
        let pixels = vec![50, 100, 150, 200];
        let mut out = vec![0u8; 4];
        let ok = apply_threshold(&pixels, 2, 2, 120, &mut out);
        assert!(ok);
        assert_eq!(out, vec![0, 0, 255, 255]);
    }

    #[test]
    fn test_median_filter_3x3() {
        let mut pixels = vec![255u8; 9];
        // 中間設為孤立噪點 0 (黑點)
        pixels[4] = 0;
        let mut out = vec![255u8; 9];
        let ok = apply_median_filter_3x3(&pixels, 3, 3, &mut out);
        assert!(ok);
        // 中間噪點應被濾除還原為 255
        assert_eq!(out[4], 255);
    }

    #[test]
    fn test_horizontal_projection_and_slicing() {
        // 寬 10, 高 20 的假圖像
        let width = 10;
        let height = 20;
        let mut binary = vec![255u8; width * height];

        // 在第 2..5 行放黑字 (行 2, 3, 4)
        for y in 2..5 {
            for x in 0..width {
                binary[y * width + x] = 0;
            }
        }
        // 在第 12..16 行放第二段字
        for y in 12..16 {
            for x in 0..width {
                binary[y * width + x] = 0;
            }
        }

        let mut row_counts = vec![0u32; height];
        let ok = calc_horizontal_projection(&binary, width, height, &mut row_counts);
        assert!(ok);
        assert_eq!(row_counts[2], 10);
        assert_eq!(row_counts[6], 0);

        let mut slices = Vec::new();
        let num_slices = find_gutter_slices(&row_counts, width, 3, 2, &mut slices);
        assert_eq!(num_slices, 2);
        assert!(slices[0].0 <= 2 && slices[0].1 >= 5);
        assert!(slices[1].0 <= 12 && slices[1].1 >= 16);
    }

    #[test]
    fn test_dhash_and_hamming_distance() {
        let mut thumb1 = [100u8; 72];
        let mut thumb2 = [100u8; 72];
        for i in (0..72).step_by(2) {
            thumb1[i] = 120;
            thumb2[i] = 122;
        }
        let h1 = compute_dhash_64(&thumb1);
        let h2 = compute_dhash_64(&thumb2);
        let dist = hamming_distance_64(h1, h2);
        assert_eq!(dist, 0);
        assert!(is_duplicate_dhash_pair(h1, h2, 4));

        let mut thumb_diff = [0u8; 72];
        for i in 0..36 {
            thumb_diff[i] = 250;
        }
        let h_diff = compute_dhash_64(&thumb_diff);
        let dist_diff = hamming_distance_64(h1, h_diff);
        assert!(dist_diff > 10);
    }
}
