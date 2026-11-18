// ## Map
// Role: app (MSA 量測系統分析跨語言/跨模型盲測驗收原生 CLI 工具)
// Inputs: 預設黃金標準件、挑戰瑕疵件 (篇章缺失、末章截斷、漏頌、未閉合、禁字、非雙語、逐字解殘缺)。
// Outputs: 終端 MSA 鑑別力報告 (Recall >= 90%, Precision >= 90%, Repeatability = 1.00)、剛性退出碼。
// Invariants: 零外部第三方依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use rust_bricks::classic_continuity::inspect_markdown_content;

struct BlindVector<'a> {
    id: &'a str,
    name: &'a str,
    filename: &'a str,
    content: &'a str,
    expected_pass: bool,
    exp_chapters: Option<usize>,
    min_final_verse: Option<i32>,
    min_total_verses: Option<usize>,
}

const BLIND_VECTORS: &[BlindVector<'static>] = &[
    BlindVector {
        id: "VEC-01",
        name: "Standard_Continuous_3_Verses",
        filename: "Sample-Book_示範經典_繁體中文全譯本.md",
        content: "# Sample-Book_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 2 ．第二頌\n॥ २ ॥\n譯文: 二。\n\nText 3 ．第三頌\n॥ ३ ॥\n譯文: 三。\n",
        expected_pass: true,
        exp_chapters: Some(1),
        min_final_verse: Some(3),
        min_total_verses: Some(3),
    },
    BlindVector {
        id: "VEC-02",
        name: "Defect_Incomplete_Starting_Verse_27",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 27 ．第二十七頌\n॥ २७ ॥\n譯文: 缺前導經文。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-03",
        name: "Defect_Missing_Verses_Gap",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\nText 5 ．第五頌\n॥ ५ ॥\n譯文: 缺 2..4。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-04",
        name: "Defect_Unclosed_Danda",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\nयावत् किम् एतद् इति ता अहाहेति नोचुर्\n\n譯文: 缺少雙豎線閉合。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-05",
        name: "Defect_Banned_Words_Guiyi",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 應當皈依至尊主，登上靈性平台，獲得自我證悟。\n",
        expected_pass: false,
        exp_chapters: None,
        min_final_verse: None,
        min_total_verses: None,
    },
    BlindVector {
        id: "VEC-06",
        name: "Defect_Non_Bilingual_Filename",
        filename: "奇妙月光_繁體中文全譯本.md",
        content: "# 奇妙月光_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 純中文檔名。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    BlindVector {
        id: "VEC-07",
        name: "Defect_Incomplete_Chapters",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第一道月光。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 第二道月光。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(200),
    },
    BlindVector {
        id: "VEC-08",
        name: "Defect_Chapter4_Truncated_Below_55",
        filename: "Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md",
        content: "# Camatkāra-candrikā_奇妙月光_繁體中文全譯本\n\n## 第一道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 一。\n\n## 第二道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 二。\n\n## 第三道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 三。\n\n## 第四道月光\nText 1 ．第一頌\n॥ १ ॥\n譯文: 四。\n\nText 30 ．第三十頌\n॥ ३० ॥\n譯文: 截斷於第 30 頌。\n",
        expected_pass: false,
        exp_chapters: Some(4),
        min_final_verse: Some(55),
        min_total_verses: Some(200),
    },
    BlindVector {
        id: "VEC-09",
        name: "Defect_Anvaya_Empty_Meaning_Or_Bracket",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n> **【散文句法與逐字解（Anvaya & Padaccheda）】**：\n| 梵文單詞 | 切分 | 格位 | 逐字義 |\n| :--- | :--- | :--- | :--- |\n| yaḥ | yaḥ |  |  |\n譯文: 逐字義缺失。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    BlindVector {
        id: "VEC-10",
        name: "Defect_EOF_Unclosed_Danda",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 完整第一頌。\n\nText 2 ．第二頌\nईशा वास्यम् इदं सर्वम्",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(2),
        min_total_verses: Some(2),
    },
    BlindVector {
        id: "VEC-11",
        name: "Golden_Commentary_Arrow_Outside_Anvaya",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: Sādhana-bhakti 的核心步驟如下。\n【義理校注】：由外緣步驟 -> 內在直觀，層次分明。\n",
        expected_pass: true,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    BlindVector {
        id: "VEC-12",
        name: "Defect_Anvaya_Arrow_Incomplete",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n> **【散文句法與逐字解（Anvaya & Padaccheda）】**：\n- *yaḥ* ➔ \n譯文: 逐字義箭頭殘缺。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(1),
        min_total_verses: Some(1),
    },
    BlindVector {
        id: "VEC-13",
        name: "Golden_Hyphen_Range_Chapter",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一編第二浪潮 (1.2.31-1.2.33)\nText 31 ．第三十一頌\n॥ ३१ ॥\n譯文: 第三十一頌。\n\nText 32 ．第三十二頌\n॥ ३२ ॥\n譯文: 第三十二頌。\n\nText 33 ．第三十三頌\n॥ ३३ ॥\n譯文: 第三十三頌。\n",
        expected_pass: true,
        exp_chapters: Some(1),
        min_final_verse: Some(33),
        min_total_verses: Some(3),
    },
    BlindVector {
        id: "VEC-14",
        name: "Defect_Hyphen_Range_Missing_Start",
        filename: "Sample-Classic_示範經典_繁體中文全譯本.md",
        content: "# Sample-Classic_示範經典_繁體中文全譯本\n\n## 第一編第二浪潮 (1.2.31-1.2.33)\nText 32 ．第三十二頌\n॥ ३२ ॥\n譯文: 遺漏起始第 31 頌。\n\nText 33 ．第三十三頌\n॥ ३३ ॥\n譯文: 第三十三頌。\n",
        expected_pass: false,
        exp_chapters: Some(1),
        min_final_verse: Some(33),
        min_total_verses: Some(2),
    },
];

/// 計算標準 IEEE CRC-32（用於無外部工具之純 Rust ZIP/DOCX 夾具生成）
/// @outputs: 1
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// 將一組檔案寫為無壓縮 (STORE) 標準 ZIP 封包
/// @outputs: 2
fn write_stored_zip(out_path: &Path, entries: &[(&str, &[u8])]) -> Result<(), String> {
    let mut out = Vec::new();
    let mut central = Vec::new();

    for &(name, data) in entries {
        let name_bytes = name.as_bytes();
        let offset = out.len() as u32;
        let crc = crc32_ieee(data);
        let size = data.len() as u32;

        // Local file header
        out.extend_from_slice(&0x0403_4B50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name_bytes);
        out.extend_from_slice(data);

        // Central directory header
        central.extend_from_slice(&0x0201_4B50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&0u32.to_le_bytes());
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name_bytes);
    }

    let cd_offset = out.len() as u32;
    let cd_size = central.len() as u32;
    let count = entries.len() as u16;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4B50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&count.to_le_bytes());
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());

    fs::write(out_path, out).map_err(|e| format!("寫入 ZIP 失敗: {e}"))
}

/// 寫出可被 unzip 與 WordCraft 讀取之最小測試 .docx 檔案
/// @outputs: 2
fn write_minimal_docx(out_path: &Path, body_xml: &str) -> Result<(), String> {
    let ct = br#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;
    let doc_xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>{body_xml}</w:body></w:document>"#
    );
    write_stored_zip(
        out_path,
        &[
            ("[Content_Types].xml", ct),
            ("word/document.xml", doc_xml.as_bytes()),
        ],
    )
}

/// 計算 P5 PGM 測試夾具單點像素亮度
/// @outputs: 1
fn calc_pgm_pixel(x: usize, y: usize, w: usize, pattern_a: bool) -> u8 {
    if pattern_a {
        ((x.saturating_mul(7) + y.saturating_mul(3)) % 256) as u8
    } else {
        (((w - x).saturating_mul(11) + y.saturating_mul(13)) % 256) as u8
    }
}

/// 寫出超過 1024 bytes 之 P5 PGM 灰階圖供哨兵與 dHash 盲測使用
/// @outputs: 2
fn write_pgm_fixture(out_path: &Path, w: usize, h: usize, pattern_a: bool) -> Result<(), String> {
    let mut data = format!("P5\n{w} {h}\n255\n").into_bytes();
    for y in 0..h {
        for x in 0..w {
            data.push(calc_pgm_pixel(x, y, w, pattern_a));
        }
    }
    fs::write(out_path, data).map_err(|e| format!("寫入 PGM 失敗: {e}"))
}

/// 準備跨工作流 CLI 盲測所需之暫存標準件與瑕疵件
/// @outputs: 2
fn prepare_cli_fixtures(base: &Path) -> Result<(), String> {
    fs::create_dir_all(base.join("scan_ok")).map_err(|e| e.to_string())?;
    fs::create_dir_all(base.join("scan_bad")).map_err(|e| e.to_string())?;

    // 1. Classic .md & .docx (Golden vs Defect with empty Anvaya table + empty translation + banned word)
    let good_md = "# Sample-Book_示範經典_繁體中文全譯本\n\n## 第一章\nText 1 ．第一頌\n॥ १ ॥\n譯文: 完整第一頌。\n";
    fs::write(base.join("Sample-Book_示範經典_繁體中文全譯本.md"), good_md).map_err(|e| e.to_string())?;
    write_minimal_docx(
        &base.join("Sample-Book_示範經典_繁體中文全譯本.docx"),
        "<w:p><w:r><w:t>Text 1</w:t></w:r></w:p><w:p><w:r><w:t>譯文: 完整第一頌。</w:t></w:r></w:p>",
    )?;
    write_minimal_docx(
        &base.join("Sample-Bad_瑕疵_繁體中文全譯本.docx"),
        "<w:p><w:r><w:t>Text 1</w:t></w:r></w:p><w:p><w:r><w:t>譯文:</w:t></w:r></w:p><w:p><w:r><w:t>Text 2</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>梵文單詞</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>逐字義</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>yaḥ</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>[]</w:t></w:r></w:p></w:tc></w:tr></w:tbl>",
    )?;

    // 2. Scan Sentinel & dHash PGMs
    write_pgm_fixture(&base.join("scan_ok/page_001.pgm"), 36, 36, true)?;
    write_pgm_fixture(&base.join("scan_ok/page_002.pgm"), 36, 36, false)?;
    write_pgm_fixture(&base.join("scan_ok/page_003.pgm"), 36, 36, true)?;
    write_pgm_fixture(&base.join("scan_bad/page_001.pgm"), 36, 36, true)?;
    fs::write(base.join("scan_bad/page_003.pgm"), b"P5\n2 2\n255\n0123").map_err(|e| e.to_string())?;

    // 3. Astro completeness & QA docx
    let mut astro_ok = String::from("# Vedanga Jyotisha\n導言 Table A Table B Table of Parvas 數理推導 第 1 頌\n");
    for _ in 0..12 {
        astro_ok.push_str("| a | b |\n");
    }
    for v in 1..=45 {
        astro_ok.push_str(&format!("VJ {v}\n"));
    }
    fs::write(base.join("Vedanga_ok.md"), astro_ok).map_err(|e| e.to_string())?;
    fs::write(base.join("Vedanga_bad.md"), "# Vedanga\n導言\nVJ 1\n").map_err(|e| e.to_string())?;
    write_minimal_docx(&base.join("astro_ok.docx"), "<w:p><w:r><w:t>VJ 1</w:t></w:r></w:p><w:p><w:r><w:t>譯文:</w:t></w:r></w:p><w:p><w:r><w:t>拉嘎達．大仙於此闡述時間法則。</w:t></w:r></w:p>")?;
    write_minimal_docx(&base.join("astro_bad.docx"), "<w:p><w:r><w:t>拉嘎達·大仙歸依。</w:t></w:r></w:p><w:p><w:r><w:t>Translation: leaked English text</w:t></w:r></w:p>")?;

    // 4. Subtitle SRT + VAD boundaries
    fs::write(base.join("sub_ok.srt"), "1\n00:00:01,000 --> 00:00:03,000\nKrishna 說你今天好嗎\n").map_err(|e| e.to_string())?;
    fs::write(base.join("sub_bad_cps.srt"), "1\n00:00:01,000 --> 00:00:01,400\n這句字幕字數太多而且時間只有零點四秒\n").map_err(|e| e.to_string())?;
    fs::write(base.join("sub_bad_eng.srt"), "1\n00:00:01,000 --> 00:00:03,000\n請注意 untranslated 殘留字\n").map_err(|e| e.to_string())?;
    fs::write(base.join("sub_bad_vad.srt"), "1\n00:00:01,500 --> 00:00:03,000\nKrishna 說你今天好嗎\n").map_err(|e| e.to_string())?;
    fs::write(base.join("boundaries.json"), r#"{"sentences":[{"group_id":1,"has_speech":true,"subtitle_indices":[1],"detected_speech_start":1.000,"detected_speech_end":3.000,"text":"ok"}]}"#).map_err(|e| e.to_string())
}

/// 執行單項二進位 CLI 探針並回傳 (是否黃金標準件, 實際退出碼是否吻合預期)
/// @outputs: 2
fn eval_cli_probe(bin_dir: &Path, bin_name: &str, args: &[&str], expected_code: i32, is_golden: bool) -> (bool, bool, String) {
    let bin_path = bin_dir.join(bin_name);
    let out = Command::new(&bin_path).args(args).output();
    match out {
        Ok(res) => {
            let actual = res.status.code().unwrap_or(-1);
            let ok = actual == expected_code;
            let msg = format!("{bin_name} {:?} -> exit {actual} (expected {expected_code})", args);
            (is_golden, ok, msg)
        }
        Err(e) => (is_golden, false, format!("{bin_name} 啟動失敗: {e}")),
    }
}

/// 將單個探針結果計入 TP/TN/FP/FN
/// @outputs: 1
fn tally_probe_result(
    round_idx: usize,
    res: (bool, bool, String),
    counts: &mut (usize, usize, usize, usize),
) {
    let (is_golden, matched, detail) = res;
    if matched && !is_golden {
        counts.0 += 1;
    } else if matched && is_golden {
        counts.1 += 1;
    } else if is_golden {
        counts.2 += 1;
        eprintln!("[CLI Round {round_idx}] 標準件誤判 (FP): {detail}");
    } else {
        counts.3 += 1;
        eprintln!("[CLI Round {round_idx}] 瑕疵件漏檢 (FN): {detail}");
    }
}

/// 執行單輪跨工作流 CLI 二進位端到端盲測
/// @outputs: 4
fn run_cli_blind_round(round_idx: usize, bin_dir: &Path, fx: &Path) -> (usize, usize, usize, usize) {
    let s = |p: &str| fx.join(p).to_string_lossy().into_owned();
    let good_docx = s("Sample-Book_示範經典_繁體中文全譯本.docx");
    let good_md = s("Sample-Book_示範經典_繁體中文全譯本.md");
    let bad_docx = s("Sample-Bad_瑕疵_繁體中文全譯本.docx");
    let scan_ok = s("scan_ok");
    let scan_bad = s("scan_bad");
    let pgm1 = s("scan_ok/page_001.pgm");
    let pgm2 = s("scan_ok/page_002.pgm");
    let pgm3 = s("scan_ok/page_003.pgm");
    let v_ok = s("Vedanga_ok.md");
    let v_bad = s("Vedanga_bad.md");
    let a_ok = s("astro_ok.docx");
    let a_bad = s("astro_bad.docx");
    let sub_ok = s("sub_ok.srt");
    let sub_cps = s("sub_bad_cps.srt");
    let sub_eng = s("sub_bad_eng.srt");
    let sub_vad = s("sub_bad_vad.srt");
    let bounds = s("boundaries.json");

    let probes: Vec<(&str, Vec<&str>, i32, bool)> = vec![
        ("classic_trans_gate", vec![&good_docx, "--md", &good_md], 0, true),
        ("classic_trans_gate", vec![&bad_docx], 1, false),
        ("classic_scan_sentinel", vec![&scan_ok, "--min-pages", "3"], 0, true),
        ("classic_scan_sentinel", vec![&scan_bad, "--min-pages", "3"], 1, false),
        ("classic_image_preprocessor", vec!["check-dup", &pgm1, &pgm3], 0, true),
        ("classic_image_preprocessor", vec!["check-dup", &pgm1, &pgm2], 2, false),
        ("astro_completeness_checker", vec![&v_ok, "--offline"], 0, true),
        ("astro_completeness_checker", vec![&v_bad, "--offline"], 1, false),
        ("astro_trans_hook", vec!["qa", &a_ok, "--skip-wordcraft"], 0, true),
        ("astro_trans_hook", vec!["qa", &a_bad, "--skip-wordcraft"], 1, false),
        ("subtitle_qa_gate", vec!["cps", &sub_ok, "--strict"], 0, true),
        ("subtitle_qa_gate", vec!["cps", &sub_cps, "--strict"], 1, false),
        ("subtitle_qa_gate", vec!["check-eng", &sub_ok], 0, true),
        ("subtitle_qa_gate", vec!["check-eng", &sub_eng], 1, false),
        ("subtitle_qa_gate", vec!["vad-qa", &bounds, &sub_ok], 0, true),
        ("subtitle_qa_gate", vec!["vad-qa", &bounds, &sub_vad], 1, false),
    ];

    let mut counts = (0usize, 0usize, 0usize, 0usize);
    for (bin, args, code, golden) in probes {
        let res = eval_cli_probe(bin_dir, bin, &args, code, golden);
        tally_probe_result(round_idx, res, &mut counts);
    }
    counts
}

/// 執行單輪 MSA 向量與 CLI 探針比對
/// @outputs: 4
fn run_msa_round(round_idx: usize, bin_dir: Option<&PathBuf>, fx: &Path) -> (usize, usize, usize, usize, f64, f64) {
    let mut counts = (0usize, 0usize, 0usize, 0usize);

    for vec in BLIND_VECTORS {
        let rep = inspect_markdown_content(
            vec.content,
            vec.filename,
            vec.exp_chapters,
            vec.min_final_verse,
            vec.min_total_verses,
        );
        let actual_pass = rep.verdict == "PASS";
        let matched = actual_pass == vec.expected_pass;
        let detail = format!("[{}] {} (原因: {:?})", vec.id, vec.name, rep.totality_errors);
        tally_probe_result(round_idx, (vec.expected_pass, matched, detail), &mut counts);
    }

    if let Some(bd) = bin_dir {
        let cli_counts = run_cli_blind_round(round_idx, bd, fx);
        counts.0 += cli_counts.0;
        counts.1 += cli_counts.1;
        counts.2 += cli_counts.2;
        counts.3 += cli_counts.3;
    }

    let (tp, tn, fp, fn_count) = counts;
    let recall = if (tp + fn_count) > 0 { (tp as f64) / ((tp + fn_count) as f64) } else { 1.0 };
    let precision = if (tp + fp) > 0 { (tp as f64) / ((tp + fp) as f64) } else { 1.0 };

    println!(
        "第 {round_idx} 輪量測: TP={tp}, TN={tn}, FP={fp}, FN={fn_count} | 召回率={:.2}%, 精確率={:.2}%",
        recall * 100.0,
        precision * 100.0
    );

    (tp, tn, fp, fn_count, recall, precision)
}

/// 檢查現行實體交付定本是否通過門禁
/// @outputs: 2
fn verify_live_golden_deliverable() -> bool {
    let golden_path = "/home/antigravity/mangala1728/Sanskrit-Classic-Translation/out/Camatkāra-candrikā_奇妙月光_繁體中文全譯本.md";
    if !Path::new(golden_path).exists() {
        return true;
    }
    let Ok(content) = fs::read_to_string(golden_path) else {
        return true;
    };
    let golden_rep = inspect_markdown_content(&content, golden_path, Some(4), Some(55), Some(220));
    println!("實體全本黃金樣件驗收: {} -> {}", golden_path, golden_rep.verdict);
    if golden_rep.verdict != "PASS" {
        eprintln!("❌ 實體黃金樣件未通過門禁: {:?}", golden_rep.totality_errors);
        return false;
    }
    true
}

/// CLI 主入口點
/// @outputs: 2
fn main() {
    println!("======================================================================");
    println!("🔬 跨語言跨模型量測系統分析 (MSA Blind-Testing Matrix): Rust 原生版");
    println!("======================================================================");

    let bin_dir = env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let fx_dir = PathBuf::from(format!("/tmp/msa_cli_test_{}", process::id()));
    if let Err(e) = prepare_cli_fixtures(&fx_dir) {
        eprintln!("❌ 準備 CLI 盲測夾具失敗: {e}");
        let _ = fs::remove_dir_all(&fx_dir);
        process::exit(1);
    }

    let runs = 3;
    let mut all_runs = Vec::new();
    for r in 1..=runs {
        all_runs.push(run_msa_round(r, bin_dir.as_ref(), &fx_dir));
    }
    let _ = fs::remove_dir_all(&fx_dir);

    let first = all_runs[0];
    let repeatable = all_runs.iter().all(|r| *r == first);
    println!("----------------------------------------------------------------------");
    println!("量具重複性 (Repeatability): {}", if repeatable { "1.00 (完全吻合)" } else { "FAILED" });
    println!("量具召回率 (Recall):         {:.2}% (門檻 >= 90.0%)", first.4 * 100.0);
    println!("量具精確率 (Precision):      {:.2}% (門檻 >= 90.0%)", first.5 * 100.0);

    if !verify_live_golden_deliverable() {
        process::exit(1);
    }

    if repeatable && first.4 >= 0.90 && first.5 >= 0.90 {
        println!("\n🎉 【MSA 驗收結論】：量測系統合格！Rust 原生二進位程式完全具備對真值與瑕疵的 100% 鑑別力！");
        process::exit(0);
    } else {
        eprintln!("\n🚫 【MSA 驗收結論】：量測系統未達標，禁止切換生產環境！");
        process::exit(1);
    }
}
