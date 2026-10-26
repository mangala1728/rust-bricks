// ## Map
// Role: core/net_tsn_macsec — IEEE 802.1AS gPTP 時鐘事件觸發、鏈路延遲解算與 IEEE 802.1AE MACsec 硬體加密 SecTAG 解析微積木。
// Inputs: 以太網 PTP 封包切片、PTP 四元時間戳 (t1..t4)、MACsec 訊框切片、防重放封包序號。
// Outputs: PTP 事件訊息判別、Pdelay 鏈路納秒延遲、MACsec SecTAG 資訊結構、防重放滑動視窗驗證結果。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、長度邊界防禦。

pub const ETHER_TYPE_PTP: u16 = 0x88F7;
pub const ETHER_TYPE_MACSEC: u16 = 0x88E5;

/// @outputs: 1
/// @domain: msg_type [0..15]
pub fn ptp_is_event_message(msg_type: u8) -> bool {
    // 802.1AS / 1588 Event 訊息: Sync(0), Delay_Req(1), Pdelay_Req(2), Pdelay_Resp(3)
    msg_type <= 3
}

pub struct PtpHeaderSummary {
    pub message_type: u8,
    pub version: u8,
    pub domain_number: u8,
    pub sequence_id: u16,
    pub correction_ns: f64,
}

/// @outputs: 2
/// @domain: ptp_len [34..1500]
pub fn ptp_parse_header(ptp_payload: &[u8]) -> Option<PtpHeaderSummary> {
    if ptp_payload.len() < 34 || ptp_payload.len() > 1500 {
        return None;
    }
    let msg_type = ptp_payload[0] & 0x0F;
    let version = ptp_payload[1] & 0x0F;
    let domain = ptp_payload[4];
    let seq_id = u16::from_be_bytes([ptp_payload[30], ptp_payload[31]]);

    // CorrectionField 位於字節 8..16 (64-bit signed integer，以 2^16 為單位的納秒)
    let raw_corr = i64::from_be_bytes([
        ptp_payload[8], ptp_payload[9], ptp_payload[10], ptp_payload[11],
        ptp_payload[12], ptp_payload[13], ptp_payload[14], ptp_payload[15],
    ]);
    let correction_ns = (raw_corr as f64) / 65536.0;

    Some(PtpHeaderSummary {
        message_type: msg_type,
        version,
        domain_number: domain,
        sequence_id: seq_id,
        correction_ns,
    })
}

/// @outputs: 2
pub fn ptp_pdelay_mean_path_delay_ns(
    t1_ns: i64,
    t2_ns: i64,
    t3_ns: i64,
    t4_ns: i64,
    correction_ns: f64,
) -> Option<f64> {
    if t4_ns < t1_ns || t3_ns < t2_ns || correction_ns.is_nan() {
        return None;
    }
    // IEEE 802.1AS Pdelay 核心公式: ((t4 - t1) - (t3 - t2) - correction) / 2
    let round_trip = (t4_ns - t1_ns) as f64;
    let peer_turnaround = (t3_ns - t2_ns) as f64;
    let delay = (round_trip - peer_turnaround - correction_ns) * 0.5;
    if delay < 0.0 {
        Some(0.0)
    } else {
        Some(delay)
    }
}

pub struct MacsecTagInfo {
    pub tci_an: u8,
    pub encryption_enabled: bool,
    pub has_sci: bool,
    pub association_number: u8,
    pub short_length: u8,
    pub packet_number: u32,
    pub sectag_len: usize,
}

/// @outputs: 2
/// @domain: frame_len [8..1518]
pub fn macsec_parse_sectag(sectag_slice: &[u8]) -> Option<MacsecTagInfo> {
    if sectag_slice.len() < 8 || sectag_slice.len() > 1518 {
        return None;
    }
    let tci_an = sectag_slice[0];
    let short_len = sectag_slice[1] & 0x3F;
    let pn = u32::from_be_bytes([
        sectag_slice[2], sectag_slice[3], sectag_slice[4], sectag_slice[5],
    ]);

    let enc_enabled = (tci_an & 0x08) != 0;
    let has_sci = (tci_an & 0x20) != 0;
    let an = tci_an & 0x03;
    let tag_len = if has_sci { 16 } else { 8 };

    if sectag_slice.len() < tag_len {
        return None;
    }
    Some(MacsecTagInfo {
        tci_an,
        encryption_enabled: enc_enabled,
        has_sci,
        association_number: an,
        short_length: short_len,
        packet_number: pn,
        sectag_len: tag_len,
    })
}

/// @outputs: 1
/// @domain: window_size [0..65536]
pub fn macsec_replay_window_validate(last_valid_pn: u32, incoming_pn: u32, window_size: u32) -> bool {
    if incoming_pn == 0 {
        return false; // MACsec PN 必大於 0
    }
    if window_size == 0 {
        return incoming_pn > last_valid_pn;
    }
    if incoming_pn > last_valid_pn {
        return true;
    }
    // 處於滑動視窗下界之內: incoming_pn + window_size >= last_valid_pn
    let diff = last_valid_pn.saturating_sub(incoming_pn);
    diff <= window_size
}
