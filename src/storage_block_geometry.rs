//! # 儲存陣列、磁碟分區表與區塊設備拓撲 (Storage Geometry, Partition Table & Block Layer)
//!
//! 提供 MBR / GPT 磁碟分區表解析、CHS 與 LBA 定址雙向換算、
//! RAID 5 異或同位與 RAID 6 GF(2^8) Q 同位步進、4K 對齊偏移、
//! NVMe LBA 格式解碼與 Flash 抹除壽命百分比純微積木。
//! 零外部依賴，符合 NASA JPL 規範與合約 09。

/// ## Map: CHS (磁柱-磁頭-磁區) 轉 LBA 邏輯區塊位址 (CHS to LBA Conversion)
/// 計算 LBA = (C * max_heads + H) * max_sectors + (S - 1)
/// @outputs: 1
/// @domain: sector >= 1 && sector <= max_sectors && max_heads > 0 && max_sectors > 0
pub fn mbr_chs_to_lba(
    cylinder: u16,
    head: u8,
    sector: u8,
    max_heads: u8,
    max_sectors: u8,
) -> Option<u32> {
    if sector < 1 || max_heads == 0 || max_sectors == 0 || sector > max_sectors || head >= max_heads {
        return None;
    }
    let track = (cylinder as u32).checked_mul(max_heads as u32)?.checked_add(head as u32)?;
    let lba = track.checked_mul(max_sectors as u32)?.checked_add((sector - 1) as u32)?;
    Some(lba)
}

/// ## Step: MBR 16 位元組分區表項目解析 (MBR Partition Entry Parse)
/// 回傳 (是否可開機, 分區類型 ID, 起始 LBA, 磁區總數)
/// @outputs: 1
/// @domain: entry.len() == 16
pub fn mbr_partition_entry_parse(entry: &[u8; 16]) -> Option<(bool, u8, u32, u32)> {
    let boot_flag = entry[0];
    if boot_flag != 0x00 && boot_flag != 0x80 {
        return None; // 0x80 開機, 0x00 非開機
    }
    let is_bootable = boot_flag == 0x80;
    let part_type = entry[4];
    let lba_start = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]);
    let sector_count = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]);
    Some((is_bootable, part_type, lba_start, sector_count))
}

/// ## Step: GPT (GUID Partition Table) 標頭簽名與版本有效性驗證 (GPT Signature Check)
/// 驗證 8-byte 簽名 "EFI PART" (0x5452415020494645) 與版本 0x00010000
/// @outputs: 1
/// @domain: bytes.len() >= 16 && bytes.len() <= 512
pub fn gpt_header_signature_valid(bytes: &[u8]) -> bool {
    if bytes.len() < 16 || bytes.len() > 512 {
        return false;
    }
    let sig = &bytes[0..8];
    if sig != b"EFI PART" {
        return false;
    }
    let revision = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    revision == 0x0001_0000
}

/// ## Calc: GPT 分區跨度磁區數解算 (GPT Partition Span Sectors)
/// 計算 span = end_lba - start_lba + 1
/// @outputs: 1
/// @domain: end_lba >= start_lba
pub fn gpt_partition_span_sectors(start_lba: u64, end_lba: u64) -> Option<u64> {
    if end_lba < start_lba {
        return None;
    }
    end_lba.checked_sub(start_lba)?.checked_add(1)
}

/// ## Calc: RAID 5 區塊異或同位原地累加步進 (RAID 5 Parity Block XOR Step)
/// @outputs: 1
/// @domain: p_acc.len() == data_block.len() && p_acc.len() > 0 && p_acc.len() <= 1_048_576
pub fn raid5_parity_block_xor_step(p_acc: &mut [u8], data_block: &[u8]) -> bool {
    if p_acc.is_empty() || p_acc.len() != data_block.len() || p_acc.len() > 1_048_576 {
        return false;
    }
    for (p, &d) in p_acc.iter_mut().zip(data_block.iter()) {
        *p ^= d;
    }
    true
}

/// ## Calc: RAID 6 Galois Field GF(2^8) Q-同位乘累加單步 (RAID 6 GF256 Q-Parity Step)
/// 依據多項式 0x11D (x^8 + x^4 + x^3 + x^2 + 1)
/// @outputs: 1
/// @domain: q_acc.len() == data_block.len() && q_acc.len() <= 1_048_576
pub fn raid6_syndrome_q_step(q_acc: &mut [u8], data_block: &[u8], gf_mult: u8) -> bool {
    if q_acc.is_empty() || q_acc.len() != data_block.len() || q_acc.len() > 1_048_576 {
        return false;
    }
    for (q, &d) in q_acc.iter_mut().zip(data_block.iter()) {
        // GF(2^8) 乘法
        let mut a = d;
        let mut b = gf_mult;
        let mut p: u8 = 0;
        for _ in 0..8 {
            if (b & 1) != 0 {
                p ^= a;
            }
            let hi = (a & 0x80) != 0;
            a <<= 1;
            if hi {
                a ^= 0x1D;
            }
            b >>= 1;
        }
        *q ^= p;
    }
    true
}

/// ## Calc: 儲存設備 4K / 邊界對齊偏移字節 (Storage Sector Alignment Offset)
/// @outputs: 1
/// @domain: sector_size_bytes > 0 && align_boundary_bytes > 0
pub fn storage_sector_alignment_offset(
    lba: u64,
    sector_size_bytes: u32,
    align_boundary_bytes: u32,
) -> Option<u32> {
    if sector_size_bytes == 0 || align_boundary_bytes == 0 {
        return None;
    }
    let byte_offset = lba.checked_mul(sector_size_bytes as u64)?;
    let rem = byte_offset % (align_boundary_bytes as u64);
    Some(rem as u32)
}

/// ## Calc: NVMe LBA 格式指數轉磁區大小 (NVMe LBAD to Sector Size Bytes)
/// 計算 2^lbad_exp (例如 9 -> 512, 12 -> 4096)
/// @outputs: 1
/// @domain: lbad_exp >= 9 && lbad_exp <= 16
pub fn nvme_lbad_to_sector_size(lbad_exp: u8) -> Option<u32> {
    if lbad_exp < 9 || lbad_exp > 16 {
        return None;
    }
    Some(1u32 << lbad_exp)
}

/// ## Calc: 快閃記憶體 (NAND Flash) 抹除壽命健康度百分比 (Flash Wear Leveling Health %)
/// @outputs: 1
/// @domain: max_erase_cycles > 0
pub fn flash_wear_leveling_health_pct(erase_count: u32, max_erase_cycles: u32) -> Option<f32> {
    if max_erase_cycles == 0 {
        return None;
    }
    if erase_count >= max_erase_cycles {
        return Some(0.0);
    }
    let remaining = (max_erase_cycles - erase_count) as f32;
    Some((remaining / max_erase_cycles as f32) * 100.0)
}

/// ## Calc: 儲存設備磁區總量轉二進位容量 (Storage Capacity in GiB)
/// 計算 GiB = (sectors * sector_size) / (1024^3)
/// @outputs: 1
/// @domain: sector_size_bytes >= 512 && sector_size_bytes <= 65536
pub fn storage_capacity_gib_from_lba(total_sectors: u64, sector_size_bytes: u32) -> Option<f64> {
    if sector_size_bytes < 512 || sector_size_bytes > 65536 {
        return None;
    }
    let bytes = (total_sectors as u128).checked_mul(sector_size_bytes as u128)?;
    let gib_bytes = 1024u128 * 1024 * 1024;
    let gib = bytes as f64 / gib_bytes as f64;
    Some(gib)
}
