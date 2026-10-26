// ## Map
// Role: core/fin_apdu_track — 金融晶片卡 ISO 7816-4 APDU 命令/回應封包與磁條 Track 2 解碼微積木。
// Inputs: APDU 標頭參數 (CLA, INS, P1, P2)、指令負載、磁條字串、LRC 數據切片。
// Outputs: APDU 二進位訊框長度、回應 (Data, SW1, SW2) 元組、卡號 (PAN) 與效期切片、LRC 校驗碼。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、長度邊界防禦。

/// @outputs: 2
/// @domain: data_len [0..255]
pub fn apdu_encode_command(
    cla: u8,
    ins: u8,
    p1: u8,
    p2: u8,
    data: &[u8],
    le: Option<u8>,
    out: &mut [u8],
) -> Option<usize> {
    if data.len() > 255 {
        return None;
    }
    let lc = data.len();
    let needed = 4 + (if lc > 0 { 1 + lc } else { 0 }) + (if le.is_some() { 1 } else { 0 });
    if out.len() < needed {
        return None;
    }
    out[0] = cla;
    out[1] = ins;
    out[2] = p1;
    out[3] = p2;
    let mut cursor = 4;
    if lc > 0 {
        out[cursor] = lc as u8;
        cursor += 1;
        out[cursor..cursor + lc].copy_from_slice(data);
        cursor += lc;
    }
    if let Some(expected_le) = le {
        out[cursor] = expected_le;
        cursor += 1;
    }
    Some(cursor)
}

/// @outputs: 2
/// @domain: resp_len [2..258]
pub fn apdu_parse_response<'a>(resp: &'a [u8]) -> Option<(&'a [u8], u8, u8)> {
    if resp.len() < 2 || resp.len() > 258 {
        return None;
    }
    let n = resp.len();
    let sw1 = resp[n - 2];
    let sw2 = resp[n - 1];
    let data = &resp[..n - 2];
    Some((data, sw1, sw2))
}

/// @outputs: 2
pub fn track2_parse_pan_and_exp<'a>(track2_str: &'a str) -> Option<(&'a str, &'a str)> {
    if track2_str.len() < 16 || track2_str.len() > 40 {
        return None;
    }
    let clean = track2_str.trim_start_matches(';').trim_end_matches('?');
    let sep_pos = clean.find('=')?;
    if sep_pos < 12 || sep_pos > 19 {
        return None;
    }
    let pan = &clean[..sep_pos];
    let after_sep = &clean[sep_pos + 1..];
    if after_sep.len() < 4 {
        return None;
    }
    let exp_yymm = &after_sep[..4];
    Some((pan, exp_yymm))
}

/// @outputs: 1
/// @domain: data_len [0..1024]
pub fn lrc_checksum(data: &[u8]) -> u8 {
    if data.len() > 1024 {
        return 0;
    }
    let mut lrc = 0u8;
    for &b in data {
        lrc ^= b;
    }
    lrc
}
