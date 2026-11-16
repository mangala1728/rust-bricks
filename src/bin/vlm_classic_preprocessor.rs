// ## Map
// Role: bin/vlm_classic_preprocessor — 古典善本圖像預處理、自適應二值化、空隙切片與 VLM 提示詞原生 CLI 工具。
// Inputs: 命令列參數 (extract / denoise / slice / vlm-prompt)、PDF/圖像路徑、輸出目錄。
// Outputs: 300 DPI 抽圖、去噪淨化圖、經文切片圖塊、結構化 VLM 提示詞。
// Invariants: 零外部第三方庫依賴、禁止 unsafe 代碼、100% 遵守 09 號合約。

#![forbid(unsafe_code)]

use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use rust_bricks::vlm_classic_preprocessor::{
    apply_median_filter_3x3, apply_threshold, calc_dynamic_otsu_threshold, calc_histogram,
    calc_horizontal_projection, find_gutter_slices, generate_vlm_verification_prompt,
};

fn print_usage() {
    eprintln!(r#"
vlm_classic_preprocessor — 古典啟示經典圖像預處理與 VLM 比對原生工具 (Rust 版)

用法:
  vlm_classic_preprocessor extract <pdf> <out_dir> [--dpi N] [--start N] [--end N]
      自 PDF 提取 300 DPI 高解析度 PNG 頁面
  vlm_classic_preprocessor denoise <image_in> <image_out> [--threshold N]
      圖像自適應色階動態二值化與 3x3 中值去噪
  vlm_classic_preprocessor slice <image_in> <out_dir> [--gap N]
      依據裝訂槽空隙水平投影切分獨立經文區塊
  vlm_classic_preprocessor vlm-prompt <image_path> [--hint <hint>]
      生成防幻覺字形交叉校對 VLM 提示詞
"#);
}

/// 讀取 P5 格式之 PGM 二進位灰度檔
fn read_pgm_p5(path: &Path) -> Result<(usize, usize, Vec<u8>), String> {
    let mut file = File::open(path).map_err(|e| format!("無法開啟 PGM 檔: {e}"))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| format!("無法讀取 PGM 檔: {e}"))?;

    let mut idx = 0;
    // 檢查魔術字元 "P5"
    if data.len() < 3 || data[0] != b'P' || data[1] != b'5' {
        return Err("無效之 PGM 格式 (必須為 P5 二進位)".to_string());
    }
    idx += 2;

    let mut tokens = Vec::new();
    while tokens.len() < 3 && idx < data.len() {
        // 跳過空白與換行
        while idx < data.len() && (data[idx] == b' ' || data[idx] == b'\t' || data[idx] == b'\r' || data[idx] == b'\n') {
            idx += 1;
        }
        if idx >= data.len() {
            break;
        }
        // 跳過註解
        if data[idx] == b'#' {
            while idx < data.len() && data[idx] != b'\n' {
                idx += 1;
            }
            continue;
        }
        // 讀取數字 token
        let start = idx;
        while idx < data.len() && data[idx] >= b'0' && data[idx] <= b'9' {
            idx += 1;
        }
        let token_str = std::str::from_utf8(&data[start..idx]).map_err(|_| "PGM 標頭 UTF-8 錯誤")?;
        let num: usize = token_str.parse().map_err(|_| "PGM 標頭數值解析失敗")?;
        tokens.push(num);
    }

    if tokens.len() < 3 {
        return Err("PGM 標頭不完整".to_string());
    }
    let width = tokens[0];
    let height = tokens[1];
    let maxval = tokens[2];
    if maxval > 255 {
        return Err("僅支援 8-bit PGM (maxval <= 255)".to_string());
    }

    // 單一換行符號後即為純像素資料
    if idx < data.len() && (data[idx] == b' ' || data[idx] == b'\n' || data[idx] == b'\r') {
        idx += 1;
    }

    let pixels = data[idx..].to_vec();
    let expected = width.saturating_mul(height);
    if pixels.len() < expected {
        return Err(format!("PGM 像素資料不足 (預期 {expected}, 實得 {})", pixels.len()));
    }

    Ok((width, height, pixels[..expected].to_vec()))
}

/// 寫入 P5 格式之 PGM 二進位灰度檔
fn write_pgm_p5(path: &Path, width: usize, height: usize, pixels: &[u8]) -> Result<(), String> {
    let mut file = File::create(path).map_err(|e| format!("無法建立 PGM 檔: {e}"))?;
    let header = format!("P5\n{width} {height}\n255\n");
    file.write_all(header.as_bytes()).map_err(|e| format!("寫入 PGM 標頭失敗: {e}"))?;
    file.write_all(pixels).map_err(|e| format!("寫入 PGM 像素失敗: {e}"))?;
    Ok(())
}

fn handle_extract(args: &[String]) {
    if args.len() < 4 {
        eprintln!("錯誤: extract 指令需要 <pdf> 與 <out_dir>");
        process::exit(1);
    }
    let pdf_path = &args[2];
    let out_dir = &args[3];
    let mut dpi = 300;
    let mut start_page = 1;
    let mut end_page: Option<usize> = None;

    let mut i = 4;
    while i < args.len() {
        match args[i].as_str() {
            "--dpi" if i + 1 < args.len() => {
                dpi = args[i + 1].parse().unwrap_or(300);
                i += 2;
            }
            "--start" if i + 1 < args.len() => {
                start_page = args[i + 1].parse().unwrap_or(1);
                i += 2;
            }
            "--end" if i + 1 < args.len() => {
                end_page = args[i + 1].parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }

    let _ = fs::create_dir_all(out_dir);
    let mut cmd = Command::new("pdftoppm");
    cmd.arg("-png").arg("-r").arg(dpi.to_string()).arg("-f").arg(start_page.to_string());
    if let Some(end) = end_page {
        cmd.arg("-l").arg(end.to_string());
    }
    let prefix = Path::new(out_dir).join("page");
    cmd.arg(pdf_path).arg(prefix);

    println!("📄 [RUST] 正在自 PDF 提取高解析度圖片 (DPI={dpi}, 頁碼 {start_page} 至 {})...", end_page.map(|p| p.to_string()).unwrap_or_else(|| "末頁".to_string()));
    let status = cmd.status().unwrap_or_else(|e| {
        eprintln!("執行 pdftoppm 失敗: {e}");
        process::exit(1);
    });

    if status.success() {
        println!("✅ 高清圖片提取完成，輸出目錄: {out_dir}");
    } else {
        eprintln!("❌ pdftoppm 執行異常");
        process::exit(1);
    }
}

fn handle_denoise(args: &[String]) {
    if args.len() < 4 {
        eprintln!("錯誤: denoise 指令需要 <image_in> 與 <image_out>");
        process::exit(1);
    }
    let image_in = &args[2];
    let image_out = &args[3];
    let mut forced_threshold: Option<u8> = None;

    let mut i = 4;
    while i < args.len() {
        if args[i] == "--threshold" && i + 1 < args.len() {
            forced_threshold = args[i + 1].parse().ok();
            i += 2;
        } else {
            i += 1;
        }
    }

    // 1. 將輸入圖轉為灰階 PGM 暫存檔
    let tmp_pgm_in = PathBuf::from("/tmp/vlm_denoise_in.pgm");
    let tmp_pgm_out = PathBuf::from("/tmp/vlm_denoise_out.pgm");

    let status = Command::new("convert")
        .arg(image_in)
        .arg("-colorspace")
        .arg("Gray")
        .arg(&tmp_pgm_in)
        .status();

    if status.is_err() || !status.unwrap().success() {
        eprintln!("❌ 無法調用 convert 將圖像轉為灰階 PGM");
        process::exit(1);
    }

    // 2. 讀取 PGM 像素
    let (width, height, raw_pixels) = match read_pgm_p5(&tmp_pgm_in) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ 讀取灰階資料失敗: {e}");
            process::exit(1);
        }
    };

    let total = (width.saturating_mul(height)) as u32;
    let hist = calc_histogram(&raw_pixels);
    let threshold = forced_threshold.unwrap_or_else(|| calc_dynamic_otsu_threshold(&hist, total));

    // 3. 二值化
    let mut binary_pixels = vec![255u8; raw_pixels.len()];
    apply_threshold(&raw_pixels, width, height, threshold, &mut binary_pixels);

    // 4. 3x3 中值去噪
    let mut clean_pixels = vec![255u8; binary_pixels.len()];
    apply_median_filter_3x3(&binary_pixels, width, height, &mut clean_pixels);

    // 5. 寫出 PGM 並轉為目標格式
    if let Err(e) = write_pgm_p5(&tmp_pgm_out, width, height, &clean_pixels) {
        eprintln!("❌ 寫出淨化 PGM 失敗: {e}");
        process::exit(1);
    }

    let status2 = Command::new("convert")
        .arg(&tmp_pgm_out)
        .arg("-density")
        .arg("300")
        .arg(image_out)
        .status();

    let _ = fs::remove_file(&tmp_pgm_in);
    let _ = fs::remove_file(&tmp_pgm_out);

    if status2.is_ok() && status2.unwrap().success() {
        println!("✨ [RUST] 圖像自適應去噪與二值化已完成 -> {image_out} (門檻值: {threshold})");
    } else {
        eprintln!("❌ 轉檔至 {image_out} 失敗");
        process::exit(1);
    }
}

fn handle_slice(args: &[String]) {
    if args.len() < 4 {
        eprintln!("錯誤: slice 指令需要 <image_in> 與 <out_dir>");
        process::exit(1);
    }
    let image_in = &args[2];
    let out_dir = &args[3];
    let mut min_gap_px = 40;

    let mut i = 4;
    while i < args.len() {
        if args[i] == "--gap" && i + 1 < args.len() {
            min_gap_px = args[i + 1].parse().unwrap_or(40);
            i += 2;
        } else {
            i += 1;
        }
    }

    let _ = fs::create_dir_all(out_dir);

    // 1. 轉為灰階 PGM
    let tmp_pgm = PathBuf::from("/tmp/vlm_slice_tmp.pgm");
    let status = Command::new("convert")
        .arg(image_in)
        .arg("-colorspace")
        .arg("Gray")
        .arg(&tmp_pgm)
        .status();

    if status.is_err() || !status.unwrap().success() {
        eprintln!("❌ 調用 convert 轉檔失敗");
        process::exit(1);
    }

    let (width, height, raw_pixels) = match read_pgm_p5(&tmp_pgm) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("❌ 讀取切片 PGM 失敗: {e}");
            process::exit(1);
        }
    };

    // 2. 二值化以精確計算黑墨像素
    let total = (width.saturating_mul(height)) as u32;
    let hist = calc_histogram(&raw_pixels);
    let threshold = calc_dynamic_otsu_threshold(&hist, total);
    let mut binary_pixels = vec![255u8; raw_pixels.len()];
    apply_threshold(&raw_pixels, width, height, threshold, &mut binary_pixels);

    // 3. 水平黑像素投影與裝訂槽切分
    let mut row_counts = vec![0u32; height];
    calc_horizontal_projection(&binary_pixels, width, height, &mut row_counts);

    let mut slices = Vec::new();
    find_gutter_slices(&row_counts, width, min_gap_px, 20, &mut slices);

    let base_name = Path::new(image_in)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("page");

    println!("✂️ [RUST] 成功偵測到 {} 個獨立經文/註疏幾何區塊，正在切片導出...", slices.len());

    for (idx, (sy, ey)) in slices.iter().enumerate() {
        let block_h = ey.saturating_sub(*sy);
        let block_pixels = &raw_pixels[sy.saturating_mul(width)..ey.saturating_mul(width)];
        let slice_pgm = format!("/tmp/slice_{idx}.pgm");
        let out_png = Path::new(out_dir).join(format!("{base_name}_block_{:02}.png", idx + 1));

        if let Ok(()) = write_pgm_p5(Path::new(&slice_pgm), width, block_h, block_pixels) {
            let _ = Command::new("convert")
                .arg(&slice_pgm)
                .arg("-density")
                .arg("300")
                .arg(&out_png)
                .status();
            let _ = fs::remove_file(&slice_pgm);
        }
    }

    let _ = fs::remove_file(&tmp_pgm);
    println!("✅ 切片完成，已存放至目錄: {out_dir}");
}

fn handle_vlm_prompt(args: &[String]) {
    if args.len() < 3 {
        eprintln!("錯誤: vlm-prompt 指令需要 <image_path>");
        process::exit(1);
    }
    let mut hint = "古典啟示聖典經文 (Revealed Classic)";
    let mut i = 3;
    while i < args.len() {
        if args[i] == "--hint" && i + 1 < args.len() {
            hint = &args[i + 1];
            i += 2;
        } else {
            i += 1;
        }
    }

    let prompt = generate_vlm_verification_prompt(hint);
    println!("\n{}", "=".repeat(60));
    println!("VLM 視覺交叉校對提示詞 (Prompt) [RUST NATIVE]:");
    println!("{}", "=".repeat(60));
    println!("{prompt}");
    println!("{}\n", "=".repeat(60));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "extract" => handle_extract(&args),
        "denoise" => handle_denoise(&args),
        "slice" => handle_slice(&args),
        "vlm-prompt" => handle_vlm_prompt(&args),
        "-h" | "--help" | "help" => print_usage(),
        other => {
            eprintln!("未知的指令: {other}");
            print_usage();
            process::exit(1);
        }
    }
}
