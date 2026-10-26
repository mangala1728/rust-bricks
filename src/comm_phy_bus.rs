// ## Map
// Role: hw/comm_phy_bus — 高速串列通信實體層 (MIPI D-PHY/CSI-2, RGMII, SGMII, DSPI, QPI) 與跨架構 FIFO 對齊微積木。
// Inputs: 虛擬通道、資料型別、長度與位址、時延皮秒 (ps)、鏈路速率、8/32-bit 暫存器緩衝區。
// Outputs: MIPI 短封包 (含 6-bit ECC)、長封包 CRC16、RGMII/SGMII 控制字、跨字長對齊字組數。
// Invariants: 零外部依賴、純 Rust 實現、除零剛性防護、無溢位、單一函式 <= 60 行。

/// @outputs: 1
/// @domain: virtual_channel [0..3]
/// @domain: data_type [0..63]
pub fn mipi_csi2_short_packet_header(data_type: u8, virtual_channel: u8, data_field: u16) -> [u8; 4] {
    let vc = virtual_channel & 0x03;
    let dt = data_type & 0x3F;
    let di = (vc << 6) | dt;
    let b1 = (data_field & 0xFF) as u8;
    let b2 = ((data_field >> 8) & 0xFF) as u8;

    // 計算 MIPI CSI-2 6-bit ECC (針對 24-bit DI + DataField)
    let d = (di as u32) | ((b1 as u32) << 8) | ((b2 as u32) << 16);
    let p0 = (d & 0x074355).count_ones() as u8 & 1;
    let p1 = (d & 0x0DA699).count_ones() as u8 & 1;
    let p2 = (d & 0x0EE8E6).count_ones() as u8 & 1;
    let p3 = (d & 0x1B1F0F).count_ones() as u8 & 1;
    let p4 = (d & 0x2E2F30).count_ones() as u8 & 1;
    let p5 = (d & 0x3FFC00).count_ones() as u8 & 1;
    let ecc = p0 | (p1 << 1) | (p2 << 2) | (p3 << 3) | (p4 << 4) | (p5 << 5);

    [di, b1, b2, ecc]
}

/// @outputs: 1
pub fn mipi_csi2_packet_crc16(payload: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in payload {
        let mut cur = byte;
        for _ in 0..8 {
            let bit = (cur & 0x01) ^ ((crc & 0x0001) as u8);
            crc >>= 1;
            if bit != 0 {
                crc ^= 0x8408; // 反向多項式 (X^16 + X^12 + X^5 + 1)
            }
            cur >>= 1;
        }
    }
    crc
}

/// @outputs: 1
pub fn mipi_dphy_lp_state_decode(dp: bool, dn: bool) -> u8 {
    match (dp, dn) {
        (false, false) => 0, // LP-00
        (false, true) => 1,  // LP-01
        (true, false) => 2,  // LP-10
        (true, true) => 3,   // LP-11 (Stop State)
    }
}

/// @outputs: 1
/// @domain: addr_24bit [0..16777215]
pub fn dspi_dual_read_command_frame(cmd: u8, addr_24bit: u32, dummy_clocks: u8) -> [u8; 5] {
    let a = addr_24bit & 0x00FF_FFFF;
    let a2 = ((a >> 16) & 0xFF) as u8;
    let a1 = ((a >> 8) & 0xFF) as u8;
    let a0 = (a & 0xFF) as u8;
    [cmd, a2, a1, a0, dummy_clocks]
}

/// @outputs: 1
pub fn qpi_quad_enable_frame() -> [u8; 2] {
    [0x38, 0x01] // 0x38: QPI Enable Opcode, 0x01: Mode Flag
}

/// @outputs: 1
/// @domain: delay_ps [0..10000]
/// @domain: tap_resolution_ps [1..1000]
pub fn rgmii_tx_delay_skew_taps(delay_ps: u32, tap_resolution_ps: u32) -> Option<u8> {
    if tap_resolution_ps == 0 || delay_ps > 10000 {
        return None;
    }
    let taps = (delay_ps + tap_resolution_ps / 2) / tap_resolution_ps;
    if taps <= 63 {
        Some(taps as u8)
    } else {
        None
    }
}

/// @outputs: 1
/// @domain: speed_mbps [10..1000]
pub fn rgmii_speed_duplex_control_byte(speed_mbps: u16, duplex_full: bool) -> Option<u8> {
    let speed_bits = match speed_mbps {
        10 => 0b00,
        100 => 0b01,
        1000 => 0b10,
        _ => return None,
    };
    let duplex_bit = if duplex_full { 0b0100 } else { 0b0000 };
    let link_up_bit = 0b1000;
    Some(link_up_bit | duplex_bit | speed_bits)
}

/// @outputs: 1
/// @domain: speed_code [0..3]
pub fn sgmii_auto_negotiation_advertisement(link_up: bool, duplex_full: bool, speed_code: u8) -> u16 {
    let mut reg: u16 = 0x0001; // SGMII 規範標準 Bit 0 恆為 1
    if link_up {
        reg |= 1 << 15;
    }
    if duplex_full {
        reg |= 1 << 12;
    }
    let code = (speed_code & 0x03) as u16;
    reg |= code << 10;
    reg
}

/// @outputs: 1
pub fn bus_pack_8bit_to_32bit_words(
    src_bytes: &[u8],
    dst_words: &mut [u32],
    big_endian: bool,
) -> usize {
    let word_count = src_bytes.len() / 4;
    let actual_words = word_count.min(dst_words.len());
    for i in 0..actual_words {
        let b0 = src_bytes[i * 4] as u32;
        let b1 = src_bytes[i * 4 + 1] as u32;
        let b2 = src_bytes[i * 4 + 2] as u32;
        let b3 = src_bytes[i * 4 + 3] as u32;
        if big_endian {
            dst_words[i] = (b0 << 24) | (b1 << 16) | (b2 << 8) | b3;
        } else {
            dst_words[i] = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
        }
    }
    actual_words
}

/// @outputs: 1
pub fn bus_unpack_32bit_to_8bit_bytes(
    src_words: &[u32],
    dst_bytes: &mut [u8],
    big_endian: bool,
) -> usize {
    let max_words = (dst_bytes.len() / 4).min(src_words.len());
    for i in 0..max_words {
        let w = src_words[i];
        if big_endian {
            dst_bytes[i * 4] = ((w >> 24) & 0xFF) as u8;
            dst_bytes[i * 4 + 1] = ((w >> 16) & 0xFF) as u8;
            dst_bytes[i * 4 + 2] = ((w >> 8) & 0xFF) as u8;
            dst_bytes[i * 4 + 3] = (w & 0xFF) as u8;
        } else {
            dst_bytes[i * 4] = (w & 0xFF) as u8;
            dst_bytes[i * 4 + 1] = ((w >> 8) & 0xFF) as u8;
            dst_bytes[i * 4 + 2] = ((w >> 16) & 0xFF) as u8;
            dst_bytes[i * 4 + 3] = ((w >> 24) & 0xFF) as u8;
        }
    }
    max_words * 4
}
