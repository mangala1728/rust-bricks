// ## Map
// Role: core/net_ethernet — 以太網 Ethernet II / 802.1Q 訊框標頭零拷貝解析與 MAC 位址分類微積木。
// Inputs: 原始以太網封包位元組切片、6 位元組 MAC 陣列。
// Outputs: MAC 類型判別 (單播/組播/廣播)、EtherType、VLAN ID、負載切片位移。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、長度邊界防禦。

/// @outputs: 1
pub fn is_broadcast_mac(mac: &[u8; 6]) -> bool {
    mac[0] == 0xFF && mac[1] == 0xFF && mac[2] == 0xFF && mac[3] == 0xFF && mac[4] == 0xFF && mac[5] == 0xFF
}

/// @outputs: 1
pub fn is_multicast_mac(mac: &[u8; 6]) -> bool {
    if is_broadcast_mac(mac) {
        return false;
    }
    // MAC 第一字節最低位為 1 代表組播
    (mac[0] & 0x01) == 0x01
}

pub struct EthernetFrameInfo {
    pub dest_mac: [u8; 6],
    pub src_mac: [u8; 6],
    pub ether_type: u16,
    pub vlan_id: Option<u16>,
    pub payload_offset: usize,
}

/// @outputs: 1
/// @domain: packet_len [14..1518]
pub fn parse_ethernet_frame_header(packet: &[u8]) -> Option<EthernetFrameInfo> {
    if packet.len() < 14 || packet.len() > 1518 {
        return None;
    }
    let mut dest = [0u8; 6];
    let mut src = [0u8; 6];
    dest.copy_from_slice(&packet[0..6]);
    src.copy_from_slice(&packet[6..12]);

    let raw_type = u16::from_be_bytes([packet[12], packet[13]]);
    if raw_type == 0x8100 {
        // 802.1Q VLAN Tag 訊框 (至少需 18 字節)
        if packet.len() < 18 {
            return None;
        }
        let tci = u16::from_be_bytes([packet[14], packet[15]]);
        let vlan_id = tci & 0x0FFF; // 低 12 位為 VLAN ID
        let inner_type = u16::from_be_bytes([packet[16], packet[17]]);
        Some(EthernetFrameInfo {
            dest_mac: dest,
            src_mac: src,
            ether_type: inner_type,
            vlan_id: Some(vlan_id),
            payload_offset: 18,
        })
    } else {
        Some(EthernetFrameInfo {
            dest_mac: dest,
            src_mac: src,
            ether_type: raw_type,
            vlan_id: None,
            payload_offset: 14,
        })
    }
}
