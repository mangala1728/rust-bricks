//! # Industrial Ethernet Fieldbus Micro-Bricks (工業即時乙太網與現場總線微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 EtherCAT 訊框頭/10-byte 數據報/WKC 工作計數器/CoE 頭、
//! PROFINET RT 訊框分類與週期計數器、POWERLINK SoC/PReq 封包、Modbus TCP 7-byte MBAP 頭。

/// ## Map: EtherCAT 乙太網訊框頭打包 (Length 11-bit, Type 1)
/// @domain: length [0, 1498]
/// @outputs: 1
pub fn ethercat_frame_header_pack(length: u16, out_buf: &mut [u8]) -> usize {
    if length > 1498 || out_buf.len() < 2 {
        return 0;
    }
    // Low 11 bits: length, bit 12..15: type (1 for EtherCAT commands)
    let val = (length & 0x07FF) | (1u16 << 12);
    out_buf[0] = (val & 0xFF) as u8;
    out_buf[1] = (val >> 8) as u8;
    2
}

/// ## Map: EtherCAT 10-byte 數據報頭封裝 (Cmd, Index, Address, Length, More, IRQ)
/// @domain: len [0, 1486]
/// @outputs: 1
pub fn ethercat_datagram_header_pack(
    cmd: u8,
    idx: u8,
    addr: u32,
    len: u16,
    more: bool,
    irq: u16,
    out_buf: &mut [u8],
) -> usize {
    if len > 1486 || out_buf.len() < 10 {
        return 0;
    }
    out_buf[0] = cmd;
    out_buf[1] = idx;
    out_buf[2] = (addr & 0xFF) as u8;
    out_buf[3] = ((addr >> 8) & 0xFF) as u8;
    out_buf[4] = ((addr >> 16) & 0xFF) as u8;
    out_buf[5] = ((addr >> 24) & 0xFF) as u8;
    // 依據 ETG.1000.4 規範，EtherCAT 數據報標頭 More 旗標位於 Bit 15 (0x8000)
    let len_more = (len & 0x07FF) | if more { 0x8000 } else { 0 };
    out_buf[6] = (len_more & 0xFF) as u8;
    out_buf[7] = (len_more >> 8) as u8;
    out_buf[8] = (irq & 0xFF) as u8;
    out_buf[9] = (irq >> 8) as u8;
    10
}

/// ## Map: EtherCAT 工作計數器 (WKC: Working Counter) 正確性驗證
/// @outputs: 1
pub fn ethercat_working_counter_verify(actual_wkc: u16, expected_wkc: u16) -> bool {
    actual_wkc == expected_wkc && expected_wkc > 0
}

/// ## Map: EtherCAT CoE (CAN Application Protocol over EtherCAT) 2-byte 頭封裝
/// @domain: service [0, 15]
/// @outputs: 1
pub fn ethercat_mailbox_coe_header_pack(service: u8, out_buf: &mut [u8]) -> usize {
    if service > 15 || out_buf.len() < 2 {
        return 0;
    }
    // High 4 bits: service (e.g. 2 for SDO Request, 3 for SDO Response)
    let val = (service as u16) << 12;
    out_buf[0] = (val & 0xFF) as u8;
    out_buf[1] = (val >> 8) as u8;
    2
}

/// ## Map: PROFINET RT 即時訊框 FrameID 分類
/// @outputs: 1
pub fn profinet_rt_frame_id_classify(frame_id: u16) -> u8 {
    if (0x0100..=0x0FFF).contains(&frame_id) {
        3 // RTC3 (Isochronous Synchronous)
    } else if (0x8000..=0xBBFF).contains(&frame_id) {
        1 // RTC1 (Real-Time Cyclic Unsynchronized)
    } else if (0xC000..=0xFBFF).contains(&frame_id) {
        2 // RTC2 (Real-Time Cyclic Synchronized)
    } else if (0xFC01..=0xFEFF).contains(&frame_id) {
        4 // Acyclic Data / Alarm
    } else {
        0 // Invalid or Non-RT
    }
}

/// ## Map: PROFINET IO 週期計數器步進差解算 (含 16-bit 溢位處理)
/// @outputs: 1
pub fn profinet_io_cycle_counter_step(last_counter: u16, current_counter: u16) -> u16 {
    current_counter.wrapping_sub(last_counter)
}

/// ## Map: Ethernet POWERLINK SoC (Start of Cycle) 網路同步封包打包
/// @outputs: 1
pub fn powerlink_soc_packet_pack(rel_time_us: u32, out_buf: &mut [u8]) -> usize {
    if out_buf.len() < 8 {
        return 0;
    }
    out_buf[0] = 0x01; // Message Type 0x01 (SoC)
    out_buf[1] = 0x00; // Destination Node (Broadcast)
    out_buf[2] = 0xF0; // Source Node (Managing Node MN = 240)
    out_buf[3] = 0x00; // Reserved
    out_buf[4] = (rel_time_us >> 24) as u8;
    out_buf[5] = (rel_time_us >> 16) as u8;
    out_buf[6] = (rel_time_us >> 8) as u8;
    out_buf[7] = (rel_time_us & 0xFF) as u8;
    8
}

/// ## Map: Ethernet POWERLINK PReq / PRes 封包解析 (Dest, Src, ServiceId)
/// @outputs: 1
pub fn powerlink_preq_pres_parse(payload: &[u8]) -> Option<(u8, u8, u16)> {
    if payload.len() < 6 {
        return None;
    }
    let msg_type = payload[0];
    if msg_type != 0x03 && msg_type != 0x04 {
        return None; // 0x03=PReq, 0x04=PRes
    }
    let dst = payload[1];
    let src = payload[2];
    let srv = ((payload[4] as u16) << 8) | (payload[5] as u16);
    Some((dst, src, srv))
}

/// ## Map: Modbus TCP MBAP 7-byte 封包頭封裝 (TransID, ProtoID 0, UnitID, Length)
/// @outputs: 1
pub fn modbus_tcp_mbap_header_pack(
    trans_id: u16,
    unit_id: u8,
    pdu_len: u16,
    out_buf: &mut [u8],
) -> usize {
    if out_buf.len() < 7 || pdu_len == 0 {
        return 0;
    }
    out_buf[0] = (trans_id >> 8) as u8;
    out_buf[1] = (trans_id & 0xFF) as u8;
    out_buf[2] = 0x00; // Protocol ID (0 for Modbus TCP)
    out_buf[3] = 0x00;
    let total_len = pdu_len.saturating_add(1); // Length includes unit_id
    out_buf[4] = (total_len >> 8) as u8;
    out_buf[5] = (total_len & 0xFF) as u8;
    out_buf[6] = unit_id;
    7
}

/// ## Map: Modbus TCP MBAP 7-byte 封包頭解碼與長度驗證
/// @outputs: 1
pub fn modbus_tcp_mbap_header_unpack(buf: &[u8]) -> Option<(u16, u8, u16)> {
    if buf.len() < 7 {
        return None;
    }
    let trans_id = ((buf[0] as u16) << 8) | (buf[1] as u16);
    let proto_id = ((buf[2] as u16) << 8) | (buf[3] as u16);
    if proto_id != 0 {
        return None; // 非 Modbus 協議
    }
    let len_field = ((buf[4] as u16) << 8) | (buf[5] as u16);
    if len_field < 2 {
        return None;
    }
    let unit_id = buf[6];
    let pdu_len = len_field - 1;
    Some((trans_id, unit_id, pdu_len))
}
