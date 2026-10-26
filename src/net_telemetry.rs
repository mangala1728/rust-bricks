// ## Map
// Role: core/net_telemetry — 手機網路遙測與快速網段比對微積木：RTT 延遲平滑、零配置 IPv4/IPv6 CIDR 子網比對與私有 IP 判定。
// Inputs: RTT (ms)、平滑因數 alpha、IPv4/IPv6 位元組陣列、前綴長度。
// Outputs: 平滑後 RTT (ms)、是否屬於 CIDR 子網 (bool)、是否為私有網段 (bool)。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、純位元遮罩運算（零正則、零 Heap 配置）、純標準庫 100% 零依賴。

/// @outputs: 1
/// @domain: rtt_ms [0.0..60000.0]
/// @domain: prev_smoothed [0.0..60000.0]
/// @domain: alpha [0.0..1.0]
pub fn rtt_jitter_smooth(rtt_ms: f32, prev_smoothed: f32, alpha: f32) -> f32 {
    if rtt_ms.is_nan() || prev_smoothed.is_nan() || alpha.is_nan() {
        return prev_smoothed;
    }
    if rtt_ms < 0.0 || alpha < 0.0 || alpha > 1.0 {
        return prev_smoothed;
    }
    alpha * rtt_ms + (1.0 - alpha) * prev_smoothed
}

/// @outputs: 2
/// @domain: prefix_len [0..32]
pub fn ipv4_in_cidr(ip: [u8; 4], net_ip: [u8; 4], prefix_len: u8) -> bool {
    if prefix_len > 32 {
        return false;
    }
    if prefix_len == 0 {
        return true;
    }
    let ip_u32 = u32::from_be_bytes(ip);
    let net_u32 = u32::from_be_bytes(net_ip);
    let mask = !0u32 << (32 - prefix_len);
    (ip_u32 & mask) == (net_u32 & mask)
}

/// @outputs: 2
/// @domain: prefix_len [0..128]
pub fn ipv6_in_cidr(ip: [u8; 16], net_ip: [u8; 16], prefix_len: u8) -> bool {
    if prefix_len > 128 {
        return false;
    }
    if prefix_len == 0 {
        return true;
    }
    let ip_u128 = u128::from_be_bytes(ip);
    let net_u128 = u128::from_be_bytes(net_ip);
    let mask = if prefix_len == 128 {
        !0u128
    } else {
        !0u128 << (128 - prefix_len)
    };
    (ip_u128 & mask) == (net_u128 & mask)
}

/// @outputs: 2
pub fn is_private_ipv4(ip: [u8; 4]) -> bool {
    // 10.0.0.0/8
    if ipv4_in_cidr(ip, [10, 0, 0, 0], 8) {
        return true;
    }
    // 172.16.0.0/12
    if ipv4_in_cidr(ip, [172, 16, 0, 0], 12) {
        return true;
    }
    // 192.168.0.0/16
    if ipv4_in_cidr(ip, [192, 168, 0, 0], 16) {
        return true;
    }
    // 127.0.0.0/8 (Loopback)
    if ipv4_in_cidr(ip, [127, 0, 0, 0], 8) {
        return true;
    }
    false
}
