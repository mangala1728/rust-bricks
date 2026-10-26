//! # LXI & HiSLIP Instrument Ethernet Protocol Micro-Bricks (實驗室網路儀控協議微積木)
//!
//! 恪守《09 號合約》：純函式、零外部依賴、NASA JPL 60 行原則、執行期邊界保護。
//! 涵蓋 RFC 4506 XDR 編解碼 (4-byte 對齊填充)、ONC RPC (RFC 5531) 呼叫標頭、
//! VXI-11 Create_Link / Device_Read 參數序列化、IVI HiSLIP 16-byte 封包頭、LXI mDNS 服務名稱格式化。

/// ## Map: RFC 4506 XDR 4-byte Big-Endian 整數序列化
/// @outputs: 1
pub fn xdr_encode_u32(val: u32, out_buf: &mut [u8]) -> usize {
    if out_buf.len() < 4 {
        return 0;
    }
    out_buf[0] = (val >> 24) as u8;
    out_buf[1] = (val >> 16) as u8;
    out_buf[2] = (val >> 8) as u8;
    out_buf[3] = (val & 0xFF) as u8;
    4
}

/// ## Map: RFC 4506 XDR 4-byte Big-Endian 整數解碼
/// @outputs: 1
pub fn xdr_decode_u32(buf: &[u8]) -> Option<(u32, usize)> {
    if buf.len() < 4 {
        return None;
    }
    let val = ((buf[0] as u32) << 24)
        | ((buf[1] as u32) << 16)
        | ((buf[2] as u32) << 8)
        | (buf[3] as u32);
    Some((val, 4))
}

/// ## Map: XDR 長度前綴與 4 位元組邊界對齊封裝 (Opaque Data Padded)
/// @outputs: 1
pub fn xdr_encode_opaque_padded(data: &[u8], out_buf: &mut [u8]) -> usize {
    let len = data.len();
    let pad = (4 - (len % 4)) % 4;
    let total_len = 4 + len + pad;
    if out_buf.len() < total_len {
        return 0;
    }
    xdr_encode_u32(len as u32, out_buf);
    out_buf[4..4 + len].copy_from_slice(data);
    for i in 0..pad {
        out_buf[4 + len + i] = 0;
    }
    total_len
}

/// ## Map: XDR 帶對齊邊界的變長資料解碼
/// @outputs: 1
pub fn xdr_decode_opaque_padded(buf: &[u8], out_buf: &mut [u8]) -> Option<(usize, usize)> {
    let (len_u32, _) = xdr_decode_u32(buf)?;
    let len = len_u32 as usize;
    let pad = (4 - (len % 4)) % 4;
    let total_consumed = 4 + len + pad;
    if buf.len() < total_consumed || out_buf.len() < len {
        return None;
    }
    out_buf[..len].copy_from_slice(&buf[4..4 + len]);
    Some((len, total_consumed))
}

/// ## Map: ONC RPC (RFC 5531) 呼叫標頭封裝 (XID, CALL, Version 2, Program, Vers, Proc, AuthNull)
/// @outputs: 1
pub fn vxi11_rpc_header_pack(
    xid: u32,
    prog: u32,
    vers: u32,
    proc_num: u32,
    out_buf: &mut [u8],
) -> usize {
    if out_buf.len() < 40 {
        return 0;
    }
    let mut offset = 0;
    offset += xdr_encode_u32(xid, &mut out_buf[offset..]);
    offset += xdr_encode_u32(0, &mut out_buf[offset..]); // MsgType = 0 (CALL)
    offset += xdr_encode_u32(2, &mut out_buf[offset..]); // RPC Version = 2
    offset += xdr_encode_u32(prog, &mut out_buf[offset..]);
    offset += xdr_encode_u32(vers, &mut out_buf[offset..]);
    offset += xdr_encode_u32(proc_num, &mut out_buf[offset..]);
    // Credentials (AUTH_NONE: flavor=0, len=0)
    offset += xdr_encode_u32(0, &mut out_buf[offset..]);
    offset += xdr_encode_u32(0, &mut out_buf[offset..]);
    // Verifier (AUTH_NONE: flavor=0, len=0)
    offset += xdr_encode_u32(0, &mut out_buf[offset..]);
    offset += xdr_encode_u32(0, &mut out_buf[offset..]);
    offset
}

/// ## Map: VXI-11 Create_Link Procedure 呼叫參數序列化
/// @outputs: 1
pub fn vxi11_create_link_params_pack(
    client_id: u32,
    lock_device: bool,
    lock_timeout_ms: u32,
    device: &str,
    out_buf: &mut [u8],
) -> usize {
    let dev_bytes = device.as_bytes();
    let pad = (4 - (dev_bytes.len() % 4)) % 4;
    let req_size = 12 + 4 + dev_bytes.len() + pad;
    if out_buf.len() < req_size {
        return 0;
    }
    let mut offset = 0;
    offset += xdr_encode_u32(client_id, &mut out_buf[offset..]);
    offset += xdr_encode_u32(if lock_device { 1 } else { 0 }, &mut out_buf[offset..]);
    offset += xdr_encode_u32(lock_timeout_ms, &mut out_buf[offset..]);
    offset += xdr_encode_opaque_padded(dev_bytes, &mut out_buf[offset..]);
    offset
}

/// ## Map: VXI-11 Device_Read Procedure 呼叫參數序列化
/// @outputs: 1
/// ## Map: VXI-11 Device_Read Procedure 呼叫參數序列化 (依據 VXI-11 規範 Section B.2: lid, requestSize, io_timeout, lock_timeout, flags, termChar)
pub fn vxi11_device_read_params_pack(
    link_id: u32,
    req_size: u32,
    io_timeout_ms: u32,
    lock_timeout_ms: u32,
    flags: u32,
    term_char: u8,
    out_buf: &mut [u8],
) -> usize {
    if out_buf.len() < 24 {
        return 0;
    }
    let mut offset = 0;
    offset += xdr_encode_u32(link_id, &mut out_buf[offset..]);
    offset += xdr_encode_u32(req_size, &mut out_buf[offset..]);
    offset += xdr_encode_u32(io_timeout_ms, &mut out_buf[offset..]);
    offset += xdr_encode_u32(lock_timeout_ms, &mut out_buf[offset..]);
    offset += xdr_encode_u32(flags, &mut out_buf[offset..]);
    offset += xdr_encode_u32(term_char as u32, &mut out_buf[offset..]);
    offset
}

/// ## Map: IVI HiSLIP 16-byte 協議封包頭封裝 (Magic 0x4853 "HS")
/// @domain: msg_type [0, 255]
/// @outputs: 1
pub fn hislip_packet_header_pack(
    msg_type: u8,
    control_code: u8,
    parameter: u32,
    payload_len: u64,
    out_buf: &mut [u8],
) -> usize {
    if out_buf.len() < 16 {
        return 0;
    }
    out_buf[0] = 0x48; // 'H'
    out_buf[1] = 0x53; // 'S'
    out_buf[2] = msg_type;
    out_buf[3] = control_code;
    out_buf[4] = (parameter >> 24) as u8;
    out_buf[5] = (parameter >> 16) as u8;
    out_buf[6] = (parameter >> 8) as u8;
    out_buf[7] = (parameter & 0xFF) as u8;
    for i in 0..8 {
        out_buf[8 + i] = ((payload_len >> (56 - i * 8)) & 0xFF) as u8;
    }
    16
}

/// ## Map: IVI HiSLIP 16-byte 協議封包頭解碼與幻數驗證
/// @outputs: 1
pub fn hislip_packet_header_unpack(buf: &[u8]) -> Option<(u8, u8, u32, u64)> {
    if buf.len() < 16 || buf[0] != 0x48 || buf[1] != 0x53 {
        return None;
    }
    let msg_type = buf[2];
    let control_code = buf[3];
    let parameter = ((buf[4] as u32) << 24)
        | ((buf[5] as u32) << 16)
        | ((buf[6] as u32) << 8)
        | (buf[7] as u32);
    let mut payload_len = 0u64;
    for i in 0..8 {
        payload_len = (payload_len << 8) | (buf[8 + i] as u64);
    }
    Some((msg_type, control_code, parameter, payload_len))
}

/// ## Map: LXI 儀器 mDNS / Zeroconf 服務註冊名稱格式化
/// @outputs: 1
pub fn lxi_mdns_service_name_format(
    instrument_model: &str,
    serial_num: &str,
    out_buf: &mut [u8],
) -> usize {
    let m = instrument_model.as_bytes();
    let s = serial_num.as_bytes();
    let suffix = b"._lxi._tcp.local";
    let total_len = m.len() + 1 + s.len() + suffix.len();
    if m.is_empty() || s.is_empty() || out_buf.len() < total_len {
        return 0;
    }
    let mut idx = 0;
    out_buf[idx..idx + m.len()].copy_from_slice(m);
    idx += m.len();
    out_buf[idx] = b'-';
    idx += 1;
    out_buf[idx..idx + s.len()].copy_from_slice(s);
    idx += s.len();
    out_buf[idx..idx + suffix.len()].copy_from_slice(suffix);
    idx += suffix.len();
    idx
}
