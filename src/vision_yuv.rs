// ## Map
// Role: core/vision_yuv — 手機相機 Camera2 (NV21 / YUV420sp) 預覽幀零依賴灰階抽取、降採樣與亮度直方圖計算微積木。
// Inputs: 相機 NV21/YUV 二進位位元組切片、寬高維度 (width, height)。
// Outputs: 8-bit 灰階二進位位元組、2x2 平均降採樣灰階影像、256-bin 亮度直方圖。
// Invariants: 零外部依賴、無遞迴、零全域可變狀態、寬高有界守衛（最大 8192x8192）、記憶體越界邊界安全、純標準庫 100% 零依賴。

const MAX_IMAGE_DIM: usize = 8192;

/// @outputs: 2
/// @domain: width [1..8192]
/// @domain: height [1..8192]
pub fn nv21_extract_grayscale(nv21_buffer: &[u8], width: usize, height: usize) -> Result<Vec<u8>, &'static str> {
    if width == 0 || width > MAX_IMAGE_DIM || height == 0 || height > MAX_IMAGE_DIM {
        return Err("影像寬高超出有界範圍 [1..8192]");
    }
    let y_plane_size = width.saturating_mul(height);
    if nv21_buffer.len() < y_plane_size {
        return Err("NV21 緩衝區長度不足以容納 Y 灰階平面");
    }
    Ok(nv21_buffer[0..y_plane_size].to_vec())
}

/// @outputs: 2
/// @domain: width [2..8192]
/// @domain: height [2..8192]
pub fn downsample_2x2_grayscale(
    gray: &[u8],
    width: usize,
    height: usize,
) -> Result<(Vec<u8>, usize, usize), &'static str> {
    if width < 2 || width > MAX_IMAGE_DIM || height < 2 || height > MAX_IMAGE_DIM {
        return Err("影像寬高不可小於 2 或超過 8192");
    }
    let expected_len = width.saturating_mul(height);
    if gray.len() < expected_len {
        return Err("灰階緩衝區長度與寬高不符");
    }
    let new_width = width / 2;
    let new_height = height / 2;
    let mut out = Vec::with_capacity(new_width * new_height);

    for y in 0..new_height {
        let row0 = (y * 2) * width;
        let row1 = (y * 2 + 1) * width;
        for x in 0..new_width {
            let col0 = x * 2;
            let col1 = x * 2 + 1;
            let p00 = gray[row0 + col0] as u32;
            let p01 = gray[row0 + col1] as u32;
            let p10 = gray[row1 + col0] as u32;
            let p11 = gray[row1 + col1] as u32;
            let avg = ((p00 + p01 + p10 + p11 + 2) / 4) as u8;
            out.push(avg);
        }
    }
    Ok((out, new_width, new_height))
}

/// @outputs: 1
pub fn calculate_luminance_histogram(gray: &[u8]) -> [u32; 256] {
    let mut hist = [0u32; 256];
    for &val in gray {
        hist[val as usize] += 1;
    }
    hist
}
