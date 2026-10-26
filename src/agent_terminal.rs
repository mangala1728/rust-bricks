// ## Map
// Role: core/agent_terminal — AI 代理馬具 (Agent Harness)、PTY 終端串流淨化與非遞迴 Glob 檔案檢索微積木。
// Inputs: 終端原始字串、Glob 萬用字元樣式、目標文字、PTY 位元組串流、目標緩衝區。
// Outputs: 脫色後字元數、萬用字元比對布林值、退格編輯後有效字元數。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、雙指針有界回溯、控制流巢狀深度 <= 2。

/// @outputs: 1
/// @domain: input_len [0..65536]
pub fn strip_ansi_codes(input: &str, out_buf: &mut [u8]) -> usize {
    let bytes = input.as_bytes();
    let len = bytes.len();
    if len > 65536 {
        return 0;
    }
    let buf_cap = out_buf.len();
    let mut out_len = 0usize;
    let mut in_escape = false;

    let mut i = 0usize;
    while i < len && out_len < buf_cap {
        let b = bytes[i];
        if in_escape {
            // ANSI 終止字元通常在 0x40..=0x7E 之間 (如 'm', 'h', 'J', 'K')
            if b >= 0x40 && b <= 0x7E && b != b'[' {
                in_escape = false;
            }
        } else if b == 0x1B && i + 1 < len && bytes[i + 1] == b'[' {
            in_escape = true;
            i += 1; // 跳過 '['
        } else {
            out_buf[out_len] = b;
            out_len += 1;
        }
        i += 1;
    }
    out_len
}

/// @outputs: 2
/// @domain: pattern_len [0..1024]
/// @domain: text_len [0..1024]
pub fn simple_glob_match(pattern: &str, text: &str) -> bool {
    let p_bytes = pattern.as_bytes();
    let t_bytes = text.as_bytes();
    let p_len = p_bytes.len();
    let t_len = t_bytes.len();
    if p_len > 1024 || t_len > 1024 {
        return false;
    }

    let mut p_idx = 0usize;
    let mut t_idx = 0usize;
    let mut star_idx = usize::MAX;
    let mut match_idx = 0usize;

    while t_idx < t_len {
        if p_idx < p_len && (p_bytes[p_idx] == b'?' || p_bytes[p_idx] == t_bytes[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p_len && p_bytes[p_idx] == b'*' {
            star_idx = p_idx;
            match_idx = t_idx;
            p_idx += 1;
        } else if star_idx != usize::MAX {
            p_idx = star_idx + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_len && p_bytes[p_idx] == b'*' {
        p_idx += 1;
    }
    p_idx == p_len
}

/// @outputs: 1
/// @domain: input_len [0..65536]
pub fn pty_line_buffer_edit(input: &[u8], out_buf: &mut [u8]) -> usize {
    let len = input.len();
    if len > 65536 {
        return 0;
    }
    let buf_cap = out_buf.len();
    let mut out_len = 0usize;

    for &b in input {
        if b == 0x08 || b == 0x7F {
            // 退格鍵 (Backspace / Delete)
            if out_len > 0 {
                out_len -= 1;
            }
        } else if b == b'\r' {
            continue; // 忽略 Carriage Return，由 \n 換行
        } else if out_len < buf_cap {
            out_buf[out_len] = b;
            out_len += 1;
        }
    }
    out_len
}
