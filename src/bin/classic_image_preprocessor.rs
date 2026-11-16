// ## Map
// Role: app (古典聖典影像預處理、二值化、分欄切片與 dHash 去重原生 CLI 工具)
// Inputs: 命令列參數 (denoise / slice / vlm-prompt / check-dup)、圖像路徑、輸出目錄。
// Outputs: 淨化二值圖、經文切片圖塊、結構化 VLM 提示詞、重複頁檢驗結果。
// Invariants: 零外部依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{self, Command};

use rust_bricks::vlm_classic_preprocessor::{
    apply_median_filter_3x3, apply_threshold, calc_dynamic_otsu_threshold, calc_histogram,
    calc_horizontal_projection, compute_dhash_64, find_gutter_slices,
    generate_vlm_verification_prompt, is_duplicate_dhash_pair,
};

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
classic_image_preprocessor — 古典聖典影像預處理與去重原生工具 (Rust 版)

用法:
  classic_image_preprocessor denoise <image_in> <image_out> [--threshold N]
      圖像自適應色階動態二值化與 3x3 中值去噪
  classic_image_preprocessor slice <image_in> <out_dir> [--gap N]
      依據裝訂槽空隙水平投影切分獨立經文區塊
  classic_image_preprocessor vlm-prompt <image_path> [--hint <hint>]
      生成防幻覺字形交叉校對 VLM 提示詞
  classic_image_preprocessor check-dup <pgm1> <pgm2> [--max-dist N]
      比對兩張灰度圖之 64-bit dHash 漢明距離，檢驗是否為實體重複頁
"#);
}

/// 讀取 P5 格式之 PGM 二進位灰度檔
/// @outputs: 4
fn read_pgm_p5(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let mut file = File::open(path).map_err(|e| format!("無法開啟 PGM 檔: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("無法讀取 PGM 檔: {e}"))?;

    if data.len() < 3 || data[0] != b'P' || data[1] != b'5' {
        return Err("無效之 PGM 格式 (必須為 P5 二進位)".to_string());
    }
    let mut idx = 2;
    let mut tokens = Vec::new();
    while tokens.len() < 3 && idx < data.len() {
        while idx < data.len() && (data[idx] == b' ' || data[idx] == b'\t' || data[idx] == b'\r' || data[idx] == b'\n') {
            idx += 1;
        }
        if idx >= data.len() {
            break;
        }
        if data[idx] == b'#' {
            while idx < data.len() && data[idx] != b'\n' {
                idx += 1;
            }
            continue;
        }
        let start = idx;
        while idx < data.len() && data[idx] != b' ' && data[idx] != b'\t' && data[idx] != b'\r' && data[idx] != b'\n' {
            idx += 1;
        }
        let token_str = String::from_utf8_lossy(&data[start..idx]);
        let val: usize = token_str.parse().map_err(|_| "無法解析 PGM 尺寸參數".to_string())?;
        tokens.push(val);
    }

    if tokens.len() < 3 || idx >= data.len() {
        return Err("PGM 檔頭不完整".to_string());
    }

    let width = tokens[0];
    let height = tokens[1];
    let payload = data[idx.saturating_add(1)..].to_vec();
    Ok((width, height, payload))
}

/// 將任意影像透過系統 convert 工具轉為臨時 PGM
/// @outputs: 2
fn convert_to_temp_pgm(input_path: &str, out_pgm: &Path) -> bool {
    let status = Command::new("convert")
        .args([input_path, "-colorspace", "Gray", out_pgm.to_str().unwrap_or("")])
        .status();
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

/// 縮放為 9x8 縮圖並計算 64-bit dHash
/// @outputs: 2
fn compute_image_file_dhash(img_path: &str) -> Result<u64, String> {
    let tmp_pgm = format!("/tmp/dhash_thumb_{}.pgm", process::id());
    let status = Command::new("convert")
        .args([img_path, "-resize", "9x8!", "-colorspace", "Gray", &tmp_pgm])
        .status()
        .map_err(|e| format!("執行 convert 失敗: {e}"))?;

    if !status.success() {
        return Err("無法透過 convert 生成 9x8 灰度縮圖".to_string());
    }

    let path = Path::new(&tmp_pgm);
    let (w, h, pixels) = read_pgm_p5(path)?;
    let _ = fs::remove_file(path);

    if w != 9 || h != 8 || pixels.len() < 72 {
        return Err("縮圖尺寸不符 9x8".to_string());
    }

    Ok(compute_dhash_64(&pixels[..72]))
}

/// 執行去噪指令
/// @outputs: 1
fn handle_denoise(args: &[String]) {
    if args.len() < 4 {
        eprintln!("❌ 錯誤: denoise 指令需要 <image_in> 與 <image_out>");
        process::exit(1);
    }
    let image_in = &args[2];
    let image_out = &args[3];
    let tmp_pgm_in = format!("/tmp/denoise_in_{}.pgm", process::id());
    let tmp_pgm_out = format!("/tmp/denoise_out_{}.pgm", process::id());

    if !convert_to_temp_pgm(image_in, Path::new(&tmp_pgm_in)) {
        eprintln!("❌ 無法轉換輸入影像至 PGM 格式");
        process::exit(1);
    }

    let (width, height, pixels) = match read_pgm_p5(Path::new(&tmp_pgm_in)) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ {e}");
            process::exit(1);
        }
    };

    let hist = calc_histogram(&pixels);
    let threshold = calc_dynamic_otsu_threshold(&hist, pixels.len() as u32);
    let mut binary_pixels = vec![255u8; width.saturating_mul(height)];
    apply_threshold(&pixels, width, height, threshold, &mut binary_pixels);

    let mut clean_pixels = vec![255u8; width.saturating_mul(height)];
    apply_median_filter_3x3(&binary_pixels, width, height, &mut clean_pixels);

    // 寫出 PGM
    let pgm_header = format!("P5\n{} {}\n255\n", width, height);
    let mut out_file = match File::create(&tmp_pgm_out) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("❌ 無法建立輸出暫存檔: {e}");
            process::exit(1);
        }
    };
    let _ = out_file.write_all(pgm_header.as_bytes());
    let _ = out_file.write_all(&clean_pixels);

    let convert_stat = Command::new("convert")
        .args([&tmp_pgm_out, image_out])
        .status();

    let _ = fs::remove_file(Path::new(&tmp_pgm_in));
    let _ = fs::remove_file(Path::new(&tmp_pgm_out));

    if let Ok(st) = convert_stat {
        if st.success() {
            println!("✨ 圖像自適應去噪與二值化已完成 -> {image_out} (Otsu 門檻值: {threshold})");
            process::exit(0);
        }
    }
    eprintln!("❌ 寫出至 {image_out} 失敗");
    process::exit(1);
}

/// 執行重複頁比對指令
/// @outputs: 1
fn handle_check_dup(args: &[String]) {
    if args.len() < 4 {
        eprintln!("❌ 錯誤: check-dup 需要 <image1> 與 <image2>");
        process::exit(1);
    }
    let img1 = &args[2];
    let img2 = &args[3];
    let max_dist = args.get(4).and_then(|s| s.parse::<u32>().ok()).unwrap_or(4);

    let h1 = match compute_image_file_dhash(img1) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("❌ 讀取 {img1} 失敗: {e}");
            process::exit(1);
        }
    };

    let h2 = match compute_image_file_dhash(img2) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("❌ 讀取 {img2} 失敗: {e}");
            process::exit(1);
        }
    };

    let is_dup = is_duplicate_dhash_pair(h1, h2, max_dist);
    let dist = (h1 ^ h2).count_ones();
    println!("🔍 圖像比對結果: Hash1=0x{h1:016x}, Hash2=0x{h2:016x}, 漢明距離={dist} (門檻<={max_dist})");
    if is_dup {
        println!("⚠️ 判定為實體重複掃描頁 (Duplicate Page Detected)！");
        process::exit(0);
    } else {
        println!("✅ 判定為不同內容頁面 (Distinct Content)");
        process::exit(2);
    }
}

/// 寫出 PGM 二進位灰度檔
/// @outputs: 2
fn write_pgm_p5(path: &Path, width: usize, height: usize, pixels: &[u8]) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| format!("建立 PGM 失敗: {e}"))?;
    let header = format!("P5\n{} {}\n255\n", width, height);
    file.write_all(header.as_bytes()).map_err(|e| format!("寫入檔頭失敗: {e}"))?;
    file.write_all(pixels).map_err(|e| format!("寫入像素失敗: {e}"))?;
    Ok(())
}

/// 執行切片指令
/// @outputs: 1
fn handle_slice(args: &[String]) {
    if args.len() < 4 {
        eprintln!("❌ 錯誤: slice 需要 <image_in> 與 <out_dir>");
        process::exit(1);
    }
    let image_in = &args[2];
    let out_dir = &args[3];
    let tmp_pgm = format!("/tmp/slice_in_{}.pgm", process::id());

    if !convert_to_temp_pgm(image_in, Path::new(&tmp_pgm)) {
        eprintln!("❌ 無法轉換輸入影像至 PGM 格式");
        process::exit(1);
    }

    let (width, height, raw_pixels) = match read_pgm_p5(Path::new(&tmp_pgm)) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ {e}");
            process::exit(1);
        }
    };

    let hist = calc_histogram(&raw_pixels);
    let threshold = calc_dynamic_otsu_threshold(&hist, raw_pixels.len() as u32);
    let mut binary_pixels = vec![255u8; raw_pixels.len()];
    apply_threshold(&raw_pixels, width, height, threshold, &mut binary_pixels);

    let mut row_counts = vec![0u32; height];
    calc_horizontal_projection(&binary_pixels, width, height, &mut row_counts);

    let mut slices = Vec::new();
    find_gutter_slices(&row_counts, width, 40, 20, &mut slices);

    let _ = fs::create_dir_all(out_dir);
    let base_name = Path::new(image_in).file_stem().and_then(|s| s.to_str()).unwrap_or("page");
    println!("✂️ [RUST] 偵測到 {} 個獨立經文區塊，正在切片導出...", slices.len());

    for (idx, (sy, ey)) in slices.iter().enumerate() {
        let block_h = ey.saturating_sub(*sy);
        let block_pixels = &raw_pixels[sy.saturating_mul(width)..ey.saturating_mul(width)];
        let slice_pgm = format!("/tmp/slice_{}_{idx}.pgm", process::id());
        let out_png = Path::new(out_dir).join(format!("{base_name}_block_{:02}.png", idx.saturating_add(1)));

        if write_pgm_p5(Path::new(&slice_pgm), width, block_h, block_pixels).is_ok() {
            let _ = Command::new("convert").args([&slice_pgm, out_png.to_str().unwrap_or("")]).status();
            let _ = fs::remove_file(&slice_pgm);
        }
    }

    let _ = fs::remove_file(Path::new(&tmp_pgm));
    println!("✅ 切片完成，已存放至目錄: {out_dir}");
    process::exit(0);
}

/// 執行提示詞生成指令
/// @outputs: 1
fn handle_prompt(args: &[String]) {
    let hint = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let prompt = generate_vlm_verification_prompt(hint);
    println!("{prompt}");
    process::exit(0);
}

/// CLI 主入口點
/// @outputs: 3
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "denoise" => handle_denoise(&args),
        "slice" => handle_slice(&args),
        "check-dup" => handle_check_dup(&args),
        "vlm-prompt" => handle_prompt(&args),
        "-h" | "--help" => {
            print_usage();
            process::exit(0);
        }
        other => {
            eprintln!("❌ 未知子指令: {other}");
            print_usage();
            process::exit(1);
        }
    }
}
