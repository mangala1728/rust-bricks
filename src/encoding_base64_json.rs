// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Zero-dependency RFC 4648 Base64 codec & iterative bounded-stack RFC 8259 JSON parser/serializer (`core` brick)
//! - Inputs: Raw byte slices (`&[u8]`) or UTF-8 JSON strings (`&str`)
//! - Outputs: Encoded Base64/JSON strings, decoded byte vectors, or `JsonVal` AST
//! - Invariants:
//!   1. Zero recursion (`R09-7`): JSON parsing and serialization use explicit bounded stacks (`MAX_JSON_DEPTH = 32`).
//!   2. Zero external crates (`std` only) and zero `static mut` (`R09-2`).
//!   3. Nesting depth <= 2 (`R09-3`) and explicit `@domain` bounds on all numeric parameters.

const B64_ALPHA: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const MAX_JSON_DEPTH: usize = 32;
const MAX_JSON_TOKENS: usize = 131_072;

#[derive(Debug, Clone, PartialEq)]
pub enum JsonVal {
    Null,
    Bool(bool),
    Num(f64),
    Str(String),
    Arr(Vec<JsonVal>),
    Obj(Vec<(String, JsonVal)>),
}

impl JsonVal {
    /// Returns string slice if `self` is `JsonVal::Str`.
    // @outputs: 2
    pub fn as_json_str(&self) -> Option<&str> {
        match self {
            JsonVal::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns `f64` if `self` is `JsonVal::Num`.
    // @outputs: 2
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            JsonVal::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns `i64` if `self` is `JsonVal::Num`.
    // @outputs: 2
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonVal::Num(n) => Some(*n as i64),
            _ => None,
        }
    }

    /// Returns `bool` if `self` is `JsonVal::Bool`.
    // @outputs: 2
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonVal::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns slice if `self` is `JsonVal::Arr`.
    // @outputs: 2
    pub fn as_arr(&self) -> Option<&[JsonVal]> {
        match self {
            JsonVal::Arr(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    /// Looks up key in `JsonVal::Obj` iteratively.
    // @outputs: 2
    pub fn get(&self, key: &str) -> Option<&JsonVal> {
        let JsonVal::Obj(pairs) = self else {
            return None;
        };
        for (k, v) in pairs {
            if k == key {
                return Some(v);
            }
        }
        None
    }

    /// Serializes `JsonVal` to compact JSON without recursion (`R09-7`).
    // @outputs: 1
    pub fn to_json_string(&self) -> String {
        serialize_json_iterative(self)
    }
}

enum SerTask<'a> {
    Val(&'a JsonVal),
    Raw(&'static str),
    Key(&'a str),
}

/// Pushes JSON array element tasks onto serialization `stack` in reverse order.
// @outputs: 1
fn push_arr_tasks<'a>(items: &'a [JsonVal], out: &mut String, stack: &mut Vec<SerTask<'a>>) {
    out.push('[');
    stack.push(SerTask::Raw("]"));
    for i in (0..items.len()).rev() {
        if i + 1 < items.len() {
            stack.push(SerTask::Raw(","));
        }
        stack.push(SerTask::Val(&items[i]));
    }
}

/// Pushes JSON object key-value tasks onto serialization `stack` in reverse order.
// @outputs: 1
fn push_obj_tasks<'a>(pairs: &'a [(String, JsonVal)], out: &mut String, stack: &mut Vec<SerTask<'a>>) {
    out.push('{');
    stack.push(SerTask::Raw("}"));
    for i in (0..pairs.len()).rev() {
        if i + 1 < pairs.len() {
            stack.push(SerTask::Raw(","));
        }
        stack.push(SerTask::Val(&pairs[i].1));
        stack.push(SerTask::Raw(":"));
        stack.push(SerTask::Key(&pairs[i].0));
    }
}

/// Appends a single `JsonVal` or pushes child serialization tasks onto `stack`.
// @outputs: 1
fn push_ser_val<'a>(val: &'a JsonVal, out: &mut String, stack: &mut Vec<SerTask<'a>>) {
    match val {
        JsonVal::Null => out.push_str("null"),
        JsonVal::Bool(true) => out.push_str("true"),
        JsonVal::Bool(false) => out.push_str("false"),
        JsonVal::Num(n) => push_json_number(*n, out),
        JsonVal::Str(s) => {
            out.push('"');
            out.push_str(&json_escape_str(s));
            out.push('"');
        }
        JsonVal::Arr(items) => push_arr_tasks(items, out, stack),
        JsonVal::Obj(pairs) => push_obj_tasks(pairs, out, stack),
    }
}

/// Formats finite `f64` cleanly for JSON output.
// @outputs: 3
// @domain: n [-1e308..1e308]
fn push_json_number(n: f64, out: &mut String) {
    if !n.is_finite() || n.abs() > 1e308 {
        out.push_str("null");
        return;
    }
    if n.fract() == 0.0 && n.abs() < 9_007_199_254_740_992.0 {
        out.push_str(&(n as i64).to_string());
        return;
    }
    out.push_str(&n.to_string());
}

/// Iteratively serializes a `JsonVal` tree using an explicit bounded stack (`R09-7`).
// @outputs: 1
pub fn serialize_json_iterative(root: &JsonVal) -> String {
    let mut out = String::with_capacity(128);
    let mut stack: Vec<SerTask<'_>> = vec![SerTask::Val(root)];
    let mut steps = 0usize;
    while let Some(task) = stack.pop() {
        if steps >= MAX_JSON_TOKENS {
            break;
        }
        steps += 1;
        match task {
            SerTask::Raw(s) => out.push_str(s),
            SerTask::Key(k) => {
                out.push('"');
                out.push_str(&json_escape_str(k));
                out.push('"');
            }
            SerTask::Val(v) => push_ser_val(v, &mut out, &mut stack),
        }
    }
    out
}

/// Escapes special characters for RFC 8259 JSON strings.
// @outputs: 1
pub fn json_escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Encodes raw bytes to standard RFC 4648 Base64 string.
// @outputs: 1
pub fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(((input.len() + 2) / 3) * 4);
    let mut i = 0usize;
    while i < input.len() {
        let b0 = input[i] as u32;
        let b1 = if i + 1 < input.len() { input[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < input.len() { input[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(B64_ALPHA[((triple >> 18) & 0x3F) as usize] as char);
        out.push(B64_ALPHA[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if i + 1 < input.len() { B64_ALPHA[((triple >> 6) & 0x3F) as usize] as char } else { '=' });
        out.push(if i + 2 < input.len() { B64_ALPHA[(triple & 0x3F) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

/// Maps a single Base64 ASCII byte to its 6-bit value (`0..63`), or `255` if invalid.
// @outputs: 2
// @domain: byte [0..255]
fn b64_val(byte: u8) -> u8 {
    if byte > 127 {
        return 255;
    }
    match byte {
        b'A'..=b'Z' => byte - b'A',
        b'a'..=b'z' => byte - b'a' + 26,
        b'0'..=b'9' => byte - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => 255,
    }
}

/// Decodes an RFC 4648 Base64 string (ignoring ASCII whitespace) into raw bytes.
// @outputs: 2
pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
    let clean: Vec<u8> = input.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    if clean.len() % 4 != 0 {
        return Err("Invalid base64 length".to_string());
    }
    let mut out = Vec::with_capacity((clean.len() / 4) * 3);
    let mut i = 0usize;
    while i < clean.len() {
        let a = b64_val(clean[i]);
        let b = b64_val(clean[i + 1]);
        if a == 255 || b == 255 {
            return Err("Invalid base64 character".to_string());
        }
        let c_pad = clean[i + 2] == b'=';
        let d_pad = clean[i + 3] == b'=';
        let c = if c_pad { 0 } else { b64_val(clean[i + 2]) };
        let d = if d_pad { 0 } else { b64_val(clean[i + 3]) };
        if (!c_pad && c == 255) || (!d_pad && d == 255) {
            return Err("Invalid base64 character".to_string());
        }
        let triple = ((a as u32) << 18) | ((b as u32) << 12) | ((c as u32) << 6) | (d as u32);
        out.push(((triple >> 16) & 0xFF) as u8);
        if !c_pad {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if !d_pad {
            out.push((triple & 0xFF) as u8);
        }
        i += 4;
    }
    Ok(out)
}

enum ContainerFrame {
    Arr(Vec<JsonVal>),
    Obj(Vec<(String, JsonVal)>, Option<String>),
}

/// Skips JSON whitespace starting at `pos` up to `bytes.len()`.
// @outputs: 1
// @domain: pos [0..16777216]
fn skip_ws(bytes: &[u8], pos: usize) -> usize {
    if pos > 16_777_216 {
        return bytes.len();
    }
    let mut idx = pos;
    while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t' | b'\r' | b'\n') {
        idx += 1;
    }
    idx
}

/// Parses a JSON string literal starting at the opening `"` byte index `pos`.
// @outputs: 2
// @domain: pos [0..16777216]
fn parse_json_string(bytes: &[u8], pos: usize) -> Result<(String, usize), String> {
    if pos >= bytes.len() || pos > 16_777_216 || bytes[pos] != b'"' {
        return Err("Expected '\"'".to_string());
    }
    let mut i = pos + 1;
    let mut out = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            return Ok((out, i + 1));
        }
        if b == b'\\' {
            let (ch, next_i) = decode_escape_byte(bytes, i + 1)?;
            out.push(ch);
            i = next_i;
            continue;
        }
        if b < 0x80 {
            out.push(b as char);
            i += 1;
            continue;
        }
        let chunk_end = (i + 4).min(bytes.len());
        let ch_str = std::str::from_utf8(&bytes[i..chunk_end]).unwrap_or_else(|e| {
            let valid = e.valid_up_to().max(1);
            std::str::from_utf8(&bytes[i..i + valid]).unwrap_or("")
        });
        let Some(ch) = ch_str.chars().next() else {
            return Err("Invalid UTF-8".to_string());
        };
        out.push(ch);
        i += ch.len_utf8();
    }
    Err("Unterminated JSON string".to_string())
}

/// Decodes a single escape character starting at `idx` (right after `\`) and returns `(char, next_idx)`.
// @outputs: 2
// @domain: idx [0..16777216]
fn decode_escape_byte(bytes: &[u8], idx: usize) -> Result<(char, usize), String> {
    if idx >= bytes.len() || idx > 16_777_216 {
        return Err("Unterminated escape".to_string());
    }
    let esc = bytes[idx];
    let next = idx + 1;
    match esc {
        b'"' => Ok(('"', next)),
        b'\\' => Ok(('\\', next)),
        b'/' => Ok(('/', next)),
        b'b' => Ok(('\x08', next)),
        b'f' => Ok(('\x0c', next)),
        b'n' => Ok(('\n', next)),
        b'r' => Ok(('\r', next)),
        b't' => Ok(('\t', next)),
        b'u' => {
            if next + 4 > bytes.len() {
                return Err("Truncated \\u escape".to_string());
            }
            let hex = std::str::from_utf8(&bytes[next..next + 4]).map_err(|_| "Invalid \\u".to_string())?;
            let code = u32::from_str_radix(hex, 16).map_err(|_| "Invalid hex in \\u".to_string())?;
            Ok((char::from_u32(code).unwrap_or('\u{FFFD}'), next + 4))
        }
        _ => Err("Unknown JSON escape".to_string()),
    }
}

/// Parses a JSON number literal starting at `pos`.
// @outputs: 2
// @domain: pos [0..16777216]
fn parse_json_number(bytes: &[u8], pos: usize) -> Result<(JsonVal, usize), String> {
    if pos >= bytes.len() || pos > 16_777_216 {
        return Err("Unexpected EOF in number".to_string());
    }
    let mut end = pos;
    while end < bytes.len() && matches!(bytes[end], b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E') {
        end += 1;
    }
    let s = std::str::from_utf8(&bytes[pos..end]).map_err(|_| "Invalid UTF-8 number".to_string())?;
    let val = s.parse::<f64>().map_err(|_| format!("Invalid number: {}", s))?;
    Ok((JsonVal::Num(val), end))
}

/// Parses a scalar JSON value (`null`, `true`, `false`, string, or number).
// @outputs: 2
// @domain: pos [0..16777216]
fn parse_json_scalar(bytes: &[u8], pos: usize) -> Result<(JsonVal, usize), String> {
    if pos >= bytes.len() || pos > 16_777_216 {
        return Err("Unexpected EOF".to_string());
    }
    if bytes[pos] == b'"' {
        let (s, next) = parse_json_string(bytes, pos)?;
        return Ok((JsonVal::Str(s), next));
    }
    if bytes[pos..].starts_with(b"null") {
        return Ok((JsonVal::Null, pos + 4));
    }
    if bytes[pos..].starts_with(b"true") {
        return Ok((JsonVal::Bool(true), pos + 4));
    }
    if bytes[pos..].starts_with(b"false") {
        return Ok((JsonVal::Bool(false), pos + 5));
    }
    parse_json_number(bytes, pos)
}

/// Attaches a completed `JsonVal` to the top container frame or sets `root`.
// @outputs: 2
fn attach_json_val(
    val: JsonVal,
    frames: &mut [ContainerFrame],
    root: &mut Option<JsonVal>,
) -> Result<(), String> {
    let Some(top) = frames.last_mut() else {
        *root = Some(val);
        return Ok(());
    };
    match top {
        ContainerFrame::Arr(items) => items.push(val),
        ContainerFrame::Obj(pairs, pending_key) => {
            let key = pending_key.take().ok_or_else(|| "Missing object key".to_string())?;
            pairs.push((key, val));
        }
    }
    Ok(())
}

/// Opens `{` or `[` onto `frames`, enforcing `MAX_JSON_DEPTH`.
// @outputs: 2
// @domain: ch [0..255]
fn open_json_container(ch: u8, frames: &mut Vec<ContainerFrame>) -> Result<bool, String> {
    if ch != b'{' && ch != b'[' {
        return Err("Expected '{' or '['".to_string());
    }
    if frames.len() >= MAX_JSON_DEPTH {
        return Err("JSON nesting depth exceeded".to_string());
    }
    if ch == b'{' {
        frames.push(ContainerFrame::Obj(Vec::new(), None));
        return Ok(true);
    }
    frames.push(ContainerFrame::Arr(Vec::new()));
    Ok(false)
}

/// Closes top container on `frames` and attaches it to parent or `root`.
// @outputs: 2
// @domain: ch [0..255]
fn close_json_container(
    ch: u8,
    frames: &mut Vec<ContainerFrame>,
    root: &mut Option<JsonVal>,
) -> Result<(), String> {
    if ch != b'}' && ch != b']' {
        return Err("Expected '}' or ']'".to_string());
    }
    let frame = frames.pop().ok_or_else(|| "Unmatched closing bracket".to_string())?;
    let closed = match frame {
        ContainerFrame::Arr(items) if ch == b']' => JsonVal::Arr(items),
        ContainerFrame::Obj(pairs, None) if ch == b'}' => JsonVal::Obj(pairs),
        _ => return Err("Mismatched JSON bracket".to_string()),
    };
    attach_json_val(closed, frames, root)
}

/// Parses an object key and trailing `:` at `pos`, storing it in top `Obj` frame.
// @outputs: 2
// @domain: pos [0..16777216]
fn parse_obj_key(bytes: &[u8], pos: usize, frames: &mut [ContainerFrame]) -> Result<usize, String> {
    if pos >= bytes.len() || pos > 16_777_216 {
        return Err("Unexpected EOF in object key".to_string());
    }
    let (k, next) = parse_json_string(bytes, pos)?;
    let colon_pos = skip_ws(bytes, next);
    if colon_pos >= bytes.len() || bytes[colon_pos] != b':' {
        return Err("Expected ':' after object key".to_string());
    }
    if let Some(ContainerFrame::Obj(_, pk)) = frames.last_mut() {
        *pk = Some(k);
    }
    Ok(colon_pos + 1)
}

/// Advances `pos` past optional `,` after a value and returns whether next token is an object key.
// @outputs: 3
// @domain: pos [0..16777216]
fn consume_comma_next(bytes: &[u8], pos: usize, frames: &[ContainerFrame]) -> (usize, bool) {
    if pos > 16_777_216 {
        return (bytes.len(), false);
    }
    let p = skip_ws(bytes, pos);
    if p < bytes.len() && bytes[p] == b',' {
        let ek = matches!(frames.last(), Some(ContainerFrame::Obj(_, _)));
        return (p + 1, ek);
    }
    (p, false)
}

/// Parses an RFC 8259 JSON string iteratively with bounded depth (`MAX_JSON_DEPTH = 32`).
// @outputs: 2
pub fn parse_json(input: &str) -> Result<JsonVal, String> {
    let bytes = input.as_bytes();
    let mut pos = skip_ws(bytes, 0);
    let mut frames: Vec<ContainerFrame> = Vec::new();
    let mut root: Option<JsonVal> = None;
    let mut expect_key = false;
    let mut steps = 0usize;

    while pos < bytes.len() && steps < MAX_JSON_TOKENS {
        steps += 1;
        pos = skip_ws(bytes, pos);
        if pos >= bytes.len() {
            break;
        }
        let b = bytes[pos];
        if expect_key && b != b'}' {
            pos = parse_obj_key(bytes, pos, &mut frames)?;
            expect_key = false;
            continue;
        }
        if b == b'{' || b == b'[' {
            expect_key = open_json_container(b, &mut frames)?;
            pos += 1;
            continue;
        }
        if b == b'}' || b == b']' {
            close_json_container(b, &mut frames, &mut root)?;
            let (np, ek) = consume_comma_next(bytes, pos + 1, &frames);
            pos = np;
            expect_key = ek;
            continue;
        }
        let (val, next) = parse_json_scalar(bytes, pos)?;
        attach_json_val(val, &mut frames, &mut root)?;
        let (np, ek) = consume_comma_next(bytes, next, &frames);
        pos = np;
        expect_key = ek;
    }
    if !frames.is_empty() {
        return Err("Unclosed JSON container".to_string());
    }
    root.ok_or_else(|| "Empty JSON input".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // @outputs: 1
    fn test_base64_roundtrip() {
        let raw = b"ai-interpreter-rust-brick\x00\xff";
        let enc = base64_encode(raw);
        assert_eq!(enc, "YWktaW50ZXJwcmV0ZXItcnVzdC1icmljawD/");
        // @safety-proof: valid base64 literal
        let dec = base64_decode(&enc).unwrap();
        assert_eq!(dec, raw);
    }

    #[test]
    // @outputs: 1
    fn test_json_parse_and_serialize_roundtrip() {
        let sample = "{\"status\":\"ok\",\"count\":3,\"items\":[true,null,\"hi\\n\"]}";
        // @safety-proof: valid RFC 8259 JSON literal
        let parsed = parse_json(sample).unwrap();
        assert_eq!(parsed.get("status").and_then(|v| v.as_json_str()), Some("ok"));
        assert_eq!(parsed.get("count").and_then(|v| v.as_i64()), Some(3));
        assert_eq!(parsed.to_json_string(), sample);
    }
}
