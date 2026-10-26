// ## Map
// Role: core/comm_controller — I2C、SPI (3/4/5線)、QSPI、UART 通信控制器時序訊框與多架構 DMA 記憶體對齊微積木。
// Inputs: 7-bit 地址、讀寫標記、CPOL/CPHA 極性、QSPI 位址、系統時脈、目標波特率、DMA 記憶體位址。
// Outputs: I2C 讀寫首字節、SPI 模式 (0..3)、QSPI 0xEB 訊框長度、UART 整數與小數分頻係數、DMA 補齊長度。
// Invariants: 零外部依賴、零 Heap 分配、無遞迴、零全域狀態、控制流巢狀深度 <= 2、除以零防禦。

/// @outputs: 1
/// @domain: addr_7bit [0..127]
pub fn i2c_address_frame(addr_7bit: u8, is_read: bool) -> u8 {
    if addr_7bit > 0x7F {
        return 0x00;
    }
    (addr_7bit << 1) | (is_read as u8)
}

/// @outputs: 1
pub fn spi_clock_mode(cpol: bool, cpha: bool) -> u8 {
    ((cpol as u8) << 1) | (cpha as u8)
}

/// @outputs: 1
pub fn spi_5wire_dc_frame(is_command: bool, data: u8) -> (bool, u8) {
    // 5 線 SPI (CS, SCK, MOSI, RESET, D/C): is_command=true 時 D/C 電位為 Low (0)
    (is_command, data)
}

/// @outputs: 2
/// @domain: addr_24bit [0..16777215]
/// @domain: dummy_cycles [0..16]
pub fn qspi_flash_fast_read_cmd(addr_24bit: u32, dummy_cycles: u8, out_frame: &mut [u8; 8]) -> usize {
    if addr_24bit > 0x00FF_FFFF || dummy_cycles > 16 {
        return 0;
    }
    // Quad I/O Fast Read (0xEB)
    out_frame[0] = 0xEB;
    out_frame[1] = ((addr_24bit >> 16) & 0xFF) as u8;
    out_frame[2] = ((addr_24bit >> 8) & 0xFF) as u8;
    out_frame[3] = (addr_24bit & 0xFF) as u8;
    let dummy_bytes = ((dummy_cycles + 1) / 2) as usize;
    let total_len = 4 + dummy_bytes;
    for i in 4..total_len {
        out_frame[i] = 0x00;
    }
    total_len
}

/// @outputs: 2
/// @domain: sys_clock_hz [1000..1000000000]
/// @domain: baud_rate [300..10000000]
pub fn uart_baud_rate_divisor(sys_clock_hz: u32, baud_rate: u32, oversampling: u8) -> Option<(u16, u8)> {
    if baud_rate == 0 || sys_clock_hz < baud_rate || (oversampling != 8 && oversampling != 16) {
        return None;
    }
    let over = oversampling as u64;
    let baud = baud_rate as u64;
    let sys = sys_clock_hz as u64;
    let denom = over * baud;
    // 加入 denom / 2 進行四捨五入，使波特率誤差最小化
    let div_scaled = (sys * 16 + denom / 2) / denom;
    let mantissa = (div_scaled >> 4) as u16;
    let fraction = (div_scaled & 0x0F) as u8;
    if mantissa == 0 {
        return None;
    }
    Some((mantissa, fraction))
}

/// @outputs: 1
/// @domain: align_bytes [1..64]
pub fn dma_align_padding_bytes(addr: usize, align_bytes: usize) -> usize {
    if align_bytes == 0 || align_bytes > 64 || (align_bytes & (align_bytes - 1)) != 0 {
        return 0;
    }
    let rem = addr % align_bytes;
    if rem == 0 {
        0
    } else {
        align_bytes - rem
    }
}
