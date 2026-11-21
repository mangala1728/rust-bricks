// SPDX-License-Identifier: LicenseRef-Project-License
//! ## Map
//! - Role: Zero-dependency HTTP/1.1 request parser, response serializer & health-check router (`core` brick)
//! - Inputs: Raw HTTP/1.1 request bytes (`&[u8]`) and `app_version` string
//! - Outputs: Parsed `HttpRequest` or serialized HTTP/1.1 response bytes
//! - Invariants:
//!   1. Zero external crates (`std` only) and zero `static mut` (`R09-2`).
//!   2. Zero recursion (`R09-7`), bounded header scanning (`MAX_HTTP_HEADERS = 64`), nesting depth <= 2 (`R09-3`).
//!   3. Explicit `@domain` bounds and runtime guards on numeric parameters (`R09-4`).

const MAX_HTTP_HEADERS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

/// Finds the byte index of the `\r\n\r\n` HTTP header terminator, if present.
// @outputs: 2
fn find_header_end(raw: &[u8]) -> Option<usize> {
    if raw.len() < 4 {
        return None;
    }
    let limit = raw.len().saturating_sub(3).min(65536);
    for i in 0..limit {
        if &raw[i..i + 4] == b"\r\n\r\n" {
            return Some(i);
        }
    }
    None
}

/// Parses an HTTP/1.1 request buffer into `HttpRequest`.
// @outputs: 2
pub fn parse_http_request(raw: &[u8]) -> Result<HttpRequest, String> {
    let head_end = find_header_end(raw).ok_or_else(|| "Incomplete HTTP header".to_string())?;
    let head_str = std::str::from_utf8(&raw[..head_end]).map_err(|_| "Non-UTF8 HTTP header".to_string())?;
    let mut lines = head_str.split("\r\n");
    let req_line = lines.next().ok_or_else(|| "Missing request line".to_string())?;
    let mut parts = req_line.split_whitespace();
    let method = parts.next().ok_or_else(|| "Missing HTTP method".to_string())?.to_string();
    let raw_target = parts.next().ok_or_else(|| "Missing HTTP target".to_string())?;
    let path = raw_target.split('?').next().unwrap_or(raw_target).to_string();

    let mut headers = Vec::new();
    for line in lines.take(MAX_HTTP_HEADERS) {
        let Some((k, v)) = line.split_once(':') else {
            continue;
        };
        headers.push((k.trim().to_ascii_lowercase(), v.trim().to_string()));
    }
    let body = raw[head_end + 4..].to_vec();
    Ok(HttpRequest { method, path, headers, body })
}

/// Serializes an HTTP/1.1 response with `Content-Length` and `Connection: close`.
// @domain: status [100..599]
// @outputs: 1
pub fn format_http_response(status: u16, reason: &str, content_type: &str, body: &[u8]) -> Vec<u8> {
    if status < 100 || status > 599 {
        return b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec();
    }
    let head = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    );
    let mut out = Vec::with_capacity(head.len() + body.len());
    out.extend_from_slice(head.as_bytes());
    out.extend_from_slice(body);
    out
}

/// Routes a parsed request against `/health` parity rules (`200` on `GET /health`, `405` on non-GET `/health`, `404` otherwise).
// @outputs: 3
pub fn route_health_request(req: &HttpRequest, app_version: &str) -> Vec<u8> {
    if req.path == "/health" && req.method == "GET" {
        let body = format!("{{\"status\":\"ok\",\"version\":\"{}\"}}", app_version);
        return format_http_response(200, "OK", "application/json", body.as_bytes());
    }
    if req.path == "/health" {
        return format_http_response(405, "Method Not Allowed", "application/json", b"{\"detail\":\"Method Not Allowed\"}");
    }
    format_http_response(404, "Not Found", "application/json", b"{\"detail\":\"Not Found\"}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // @outputs: 1
    fn test_http_health_routing_200_404_405() {
        let get_raw = b"GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
        // @safety-proof: valid HTTP/1.1 literal
        let req_ok = parse_http_request(get_raw).unwrap();
        let resp_ok = String::from_utf8_lossy(&route_health_request(&req_ok, "0.1.0")).to_string();
        assert!(resp_ok.starts_with("HTTP/1.1 200 OK"));
        assert!(resp_ok.contains("{\"status\":\"ok\",\"version\":\"0.1.0\"}"));

        let post_raw = b"POST /health HTTP/1.1\r\nHost: localhost\r\n\r\n";
        // @safety-proof: valid HTTP/1.1 literal
        let req_405 = parse_http_request(post_raw).unwrap();
        let resp_405 = String::from_utf8_lossy(&route_health_request(&req_405, "0.1.0")).to_string();
        assert!(resp_405.starts_with("HTTP/1.1 405 Method Not Allowed"));

        let nf_raw = b"GET /unknown HTTP/1.1\r\nHost: localhost\r\n\r\n";
        // @safety-proof: valid HTTP/1.1 literal
        let req_404 = parse_http_request(nf_raw).unwrap();
        let resp_404 = String::from_utf8_lossy(&route_health_request(&req_404, "0.1.0")).to_string();
        assert!(resp_404.starts_with("HTTP/1.1 404 Not Found"));
    }
}
