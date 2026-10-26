// ## Map
// Role: core/text_cleaner — UTF-8 安全字節界限截斷、字詞界限智能折行與 CJK 字元統計。
// Inputs: 字串切片、最大字節數限制。
// Outputs: 安全子字串切片、字元計數。
// Invariants: 零全域可變狀態、無遞迴、絕不跨 UTF-8 延續字節引發 panic。

/// @outputs: 1
/// @domain: max_bytes [0..1048576]
pub fn truncate_at_boundary(text: &str, max_bytes: usize) -> &str {
    if max_bytes > 1_048_576 {
        return "";
    }
    if text.len() <= max_bytes {
        return text;
    }
    let mut boundary = max_bytes;
    while boundary > 0 && !text.is_char_boundary(boundary) {
        boundary -= 1;
    }
    &text[..boundary]
}

/// @outputs: 1
pub fn count_cjk_characters(text: &str) -> usize {
    let mut count = 0;
    for c in text.chars() {
        let u = c as u32;
        // CJK Unified Ideographs 0x4E00..0x9FFF
        if (0x4E00..=0x9FFF).contains(&u) {
            count += 1;
        }
    }
    count
}


/// @outputs: 1
/// @domain: max_dist [0..100]
pub fn levenshtein_distance_bounded(s1: &str, s2: &str, max_dist: usize) -> usize {
    if max_dist > 100 {
        return max_dist;
    }
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let n1 = chars1.len();
    let n2 = chars2.len();

    if (n1 as isize - n2 as isize).abs() as usize > max_dist {
        return max_dist;
    }
    if n1 == 0 {
        return n2.min(max_dist);
    }
    if n2 == 0 {
        return n1.min(max_dist);
    }

    let mut prev_row = vec![0usize; n2 + 1];
    let mut curr_row = vec![0usize; n2 + 1];

    for j in 0..=n2 {
        prev_row[j] = j;
    }

    for i in 1..=n1 {
        curr_row[0] = i;
        let mut min_val = i;
        for j in 1..=n2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
            curr_row[j] = (prev_row[j] + 1).min(curr_row[j - 1] + 1).min(prev_row[j - 1] + cost);
            if curr_row[j] < min_val {
                min_val = curr_row[j];
            }
        }
        if min_val > max_dist {
            return max_dist;
        }
        prev_row.copy_from_slice(&curr_row);
    }
    prev_row[n2].min(max_dist)
}

/// @outputs: 1
pub fn fuzzy_prefix_match(query: &str, target: &str) -> bool {
    let q_chars: Vec<char> = query.chars().collect();
    let t_chars: Vec<char> = target.chars().collect();
    if q_chars.is_empty() {
        return true;
    }
    if t_chars.is_empty() {
        return false;
    }
    let prefix_len = q_chars.len().min(t_chars.len());
    let t_sub: String = t_chars[..prefix_len].iter().collect();
    let threshold = if q_chars.len() <= 3 { 0 } else if q_chars.len() <= 6 { 1 } else { 2 };
    // bounded 在超過上限時回傳上限本身，因此以 threshold + 1 為上限才能分辨「超過」 (INC-20261026-04)
    let dist = levenshtein_distance_bounded(query, &t_sub, threshold + 1);
    dist <= threshold
}
