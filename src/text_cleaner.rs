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
