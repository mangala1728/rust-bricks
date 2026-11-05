// ## Map
// Role: core/json_repair — 純 Rust 零外部依賴之大模型輸出 JSON 容錯修復微積木。
// Inputs: raw LLM JSON 字串切片 (&str)。
// Outputs: 依序完成去籬笆、引號合併、補逗點、內部引號跳脫、尾逗點丟棄、溢出括號清理之修復字串。
// Invariants: 零外部依賴、禁止 unsafe、無遞迴 (R09-7)、有界迴圈 (R09-8)、UTF-8 安全。

const MAX_JSON_CHARS: usize = 65_536;

#[inline]
fn is_ws(c: char) -> bool {
    c == ' ' || c == '\t' || c == '\n' || c == '\r'
}

fn next_visible(chars: &[(usize, char)], from: usize) -> usize {
    let mut i = from;
    while i < chars.len() && is_ws(chars[i].1) {
        i += 1;
    }
    i
}

/// 截取首個 `{` 至末個 `}` 之間的內容，去除 markdown 標籤 (```json) 與對話前言/後語。
// @outputs: 1
pub fn repair_json_trimmed_braces(text: &str) -> &str {
    let mut first = None;
    let mut last = None;

    for (i, c) in text.char_indices().take(MAX_JSON_CHARS) {
        if c == '{' && first.is_none() {
            first = Some(i);
        }
        if c == '}' {
            last = Some(i);
        }
    }

    match (first, last) {
        (Some(start), Some(end_idx)) if end_idx >= start => {
            let end_char_len = text[end_idx..].chars().next().map_or(1, |c| c.len_utf8());
            &text[start..end_idx + end_char_len]
        }
        _ => text,
    }
}

/// 修復鍵值冒號後多餘的前導引號，如 `: " "Does` 或 `: ""text"` -> `: "Does`。
// @outputs: 1
pub fn repair_json_merge_leading_quotes(text: &str) -> String {
    let chars: Vec<(usize, char)> = text.char_indices().take(MAX_JSON_CHARS).collect();
    let len = chars.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;

    while i < len {
        if chars[i].1 == ':' {
            out.push(':');
            i += 1;
            while i < len && is_ws(chars[i].1) {
                out.push(chars[i].1);
                i += 1;
            }
            if i < len && chars[i].1 == '"' {
                out.push('"');
                i += 1;
                let mut ws_start = i;
                while ws_start < len && (chars[ws_start].1 == ' ' || chars[ws_start].1 == '\t') {
                    ws_start += 1;
                }
                if ws_start < len && chars[ws_start].1 == '"' {
                    let next_idx = ws_start + 1;
                    if next_idx < len && !is_ws(chars[next_idx].1) && chars[next_idx].1 != ',' && chars[next_idx].1 != '}' && chars[next_idx].1 != ']' && chars[next_idx].1 != ':' {
                        i = ws_start + 1;
                        continue;
                    }
                }
            }
            continue;
        }

        out.push(chars[i].1);
        i += 1;
    }

    out
}

fn line_ends_value(line: &str) -> bool {
    let trimmed = line.trim_end();
    if trimmed.is_empty() {
        return false;
    }
    let last = trimmed.chars().last().unwrap_or(' ');
    if last == '"' || last == '}' || last == ']' || last.is_ascii_digit() {
        return true;
    }
    trimmed.ends_with("true") || trimmed.ends_with("false") || trimmed.ends_with("null")
}

fn line_starts_key(line: &str) -> bool {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('"') {
        return false;
    }
    let chars: Vec<char> = trimmed.chars().collect();
    let mut i = 1;
    while i < chars.len() {
        if chars[i] == '\\' {
            i += 2;
            continue;
        }
        if chars[i] == '"' {
            let mut next = i + 1;
            while next < chars.len() && is_ws(chars[next]) {
                next += 1;
            }
            return next < chars.len() && chars[next] == ':';
        }
        i += 1;
    }
    false
}

/// 補齊多行欄位之間遺漏的逗號。
// @outputs: 1
pub fn repair_json_add_missing_commas(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out = String::with_capacity(text.len() + 64);

    for (idx, line) in lines.iter().enumerate() {
        let has_next = idx + 1 < lines.len();
        if has_next && line_ends_value(line) && line_starts_key(lines[idx + 1]) {
            let trimmed_end_len = line.trim_end().len();
            out.push_str(&line[..trimmed_end_len]);
            out.push(',');
            out.push_str(&line[trimmed_end_len..]);
        } else {
            out.push_str(line);
        }
        if has_next {
            out.push('\n');
        }
    }

    out
}

fn ends_string(chars: &[(usize, char)], from: usize) -> bool {
    let next = next_visible(chars, from);
    if next >= chars.len() {
        return true;
    }
    match chars[next].1 {
        ':' | '}' | ']' => true,
        ',' => {
            let after_comma = next_visible(chars, next + 1);
            if after_comma >= chars.len() {
                return true;
            }
            matches!(chars[after_comma].1, '"' | '{' | '[' | '}' | ']')
        }
        _ => false,
    }
}

/// 轉義字串內部未跳脫的雙引號（如 `"He said "yes", then left"`）。
// @outputs: 1
pub fn repair_json_escape_inner_quotes(text: &str) -> String {
    let chars: Vec<(usize, char)> = text.char_indices().take(MAX_JSON_CHARS).collect();
    let len = chars.len();
    let mut out = String::with_capacity(text.len() + 32);

    let mut in_string = false;
    let mut escaped = false;

    for i in 0..len {
        let c = chars[i].1;
        if in_string {
            if escaped {
                escaped = false;
                out.push(c);
                continue;
            }
            if c == '\\' {
                escaped = true;
                out.push('\\');
                continue;
            }
            if c == '"' {
                if ends_string(&chars, i + 1) {
                    in_string = false;
                    out.push('"');
                } else {
                    out.push('\\');
                    out.push('"');
                }
                continue;
            }
            out.push(c);
        } else {
            if c == '"' {
                in_string = true;
            }
            out.push(c);
        }
    }

    out
}

/// 刪除位於 `}` 或 `]` 前多餘的尾隨逗點。
// @outputs: 1
pub fn repair_json_drop_trailing_commas(text: &str) -> String {
    let chars: Vec<(usize, char)> = text.char_indices().take(MAX_JSON_CHARS).collect();
    let len = chars.len();
    let mut out = String::with_capacity(text.len());

    let mut in_string = false;
    let mut escaped = false;

    for i in 0..len {
        let c = chars[i].1;
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            out.push(c);
            continue;
        }

        if c == '"' {
            in_string = true;
            out.push('"');
            continue;
        }

        if c == ',' {
            let next = next_visible(&chars, i + 1);
            if next < len && (chars[next].1 == '}' || chars[next].1 == ']') {
                continue;
            }
        }

        out.push(c);
    }

    out
}

/// 刪除字串外部結尾多餘的閉合大括號 `}`。
// @outputs: 1
pub fn repair_json_drop_excess_braces(text: &str) -> String {
    let chars: Vec<(usize, char)> = text.char_indices().take(MAX_JSON_CHARS).collect();

    let mut in_string = false;
    let mut escaped = false;
    let mut opens = 0usize;
    let mut close_indices = Vec::new();

    for (idx, &(_, c)) in chars.iter().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
        } else if c == '{' {
            opens += 1;
        } else if c == '}' {
            close_indices.push(idx);
        }
    }

    if close_indices.len() <= opens {
        return text.to_string();
    }

    let excess = close_indices.len() - opens;
    let drop_set: std::collections::BTreeSet<usize> = close_indices.iter().rev().take(excess).copied().collect();

    let mut out = String::with_capacity(text.len());
    for (idx, &(_, c)) in chars.iter().enumerate() {
        if !drop_set.contains(&idx) {
            out.push(c);
        }
    }

    out
}

/// 綜合六步順序修復 LLM 輸出的損毀 JSON 字串。
// @outputs: 1
pub fn repair_json_stream(raw: &str) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let s1 = repair_json_trimmed_braces(raw);
    let s2 = repair_json_merge_leading_quotes(s1);
    let s3 = repair_json_add_missing_commas(&s2);
    let s4 = repair_json_escape_inner_quotes(&s3);
    let s5 = repair_json_drop_trailing_commas(&s4);
    repair_json_drop_excess_braces(&s5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repair_json_trimmed_braces() {
        let fenced = "```json\n{\"key\": \"val\"}\n```";
        assert_eq!(repair_json_trimmed_braces(fenced), "{\"key\": \"val\"}");

        let preamble = "Here is the response:\n{\"status\": 200}\nHope it helps!";
        assert_eq!(repair_json_trimmed_braces(preamble), "{\"status\": 200}");

        let plain = "no braces here";
        assert_eq!(repair_json_trimmed_braces(plain), "no braces here");
    }

    #[test]
    fn test_repair_json_merge_leading_quotes() {
        let bad = "{\"msg\": \" \"Does it work?\"}";
        assert_eq!(repair_json_merge_leading_quotes(bad), "{\"msg\": \"Does it work?\"}");

        let empty = "{\"empty\": \"\"}";
        assert_eq!(repair_json_merge_leading_quotes(empty), "{\"empty\": \"\"}");
    }

    #[test]
    fn test_repair_json_add_missing_commas() {
        let missing = "{\n  \"zh-TW\": \"你好\"\n  \"en\": \"hello\"\n}";
        let fixed = repair_json_add_missing_commas(missing);
        assert!(fixed.contains("\"zh-TW\": \"你好\","));
    }

    #[test]
    fn test_repair_json_escape_inner_quotes() {
        let unescaped = "{\"en\": \"He said \"yes\", then left\"}";
        assert_eq!(repair_json_escape_inner_quotes(unescaped), "{\"en\": \"He said \\\"yes\\\", then left\"}");

        let valid = "{\"en\": \"Where is the pharmacy?\"}";
        assert_eq!(repair_json_escape_inner_quotes(valid), valid);
    }

    #[test]
    fn test_repair_json_drop_trailing_commas() {
        let trailing = "{\"a\": 1, \"b\": [2, 3,],}";
        assert_eq!(repair_json_drop_trailing_commas(trailing), "{\"a\": 1, \"b\": [2, 3]}");

        let inside_str = "{\"a\": \"x, }\", \"b\": 2}";
        assert_eq!(repair_json_drop_trailing_commas(inside_str), inside_str);
    }

    #[test]
    fn test_repair_json_drop_excess_braces() {
        let excess = "{\"a\": \"b\"}}}";
        assert_eq!(repair_json_drop_excess_braces(excess), "{\"a\": \"b\"}");
    }

    #[test]
    fn test_repair_json_stream() {
        let messy = "```json\n{\n  \"text\": \"He said \"hi\"\"\n  \"val\": 10,\n}}\n```";
        let repaired = repair_json_stream(messy);
        assert!(repaired.starts_with('{'));
        assert!(repaired.ends_with('}'));
        assert!(repaired.contains("\\\"hi\\\""));
        assert!(repaired.contains("\"text\": \"He said \\\"hi\\\"\","));
        assert!(!repaired.contains("10,"));
        assert!(!repaired.ends_with("}}"));
    }
}
