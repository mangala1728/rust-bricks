// ## Map
// Role: media/subtitle_qa_audit — 影視字幕 SRT 解析、CPS 語速分級、殘留英文字詞掃描與 VAD 語音邊界驗收 Hook 純函式積木。
// Inputs: SRT 純文字內容、sentence_speech_boundaries.json 純文字內容、閾值與容差參數。
// Outputs: 結構化 CPS 報告、殘留英文字報告、VAD 邊界驗收與貼合修正結果。
// Invariants: 零外部依賴、零全域可變狀態、禁止 unsafe、無遞迴、函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use crate::subtitle_cps_vad::{subtitle_cps_tenths, subtitle_vad_drift_ok};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrtEntry {
    pub idx: usize,
    pub start_ms: u32,
    pub end_ms: u32,
    pub duration_ms: u32,
    pub text: String,
    pub readable_chars: u32,
    pub cps_tenths: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpsAuditReport {
    pub total: usize,
    pub red: usize,
    pub orange: usize,
    pub green: usize,
    pub inverted: usize,
    pub dummy_half_sec: usize,
    pub red_entries: Vec<SrtEntry>,
    pub inverted_entries: Vec<SrtEntry>,
    pub dummy_entries: Vec<SrtEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidualEnglishHit {
    pub idx: usize,
    pub start_ms: u32,
    pub end_ms: u32,
    pub words: Vec<String>,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpeechSentenceBoundary {
    pub group_id: usize,
    pub has_speech: bool,
    pub subtitle_indices: Vec<usize>,
    pub speech_start_ms: u32,
    pub speech_end_ms: u32,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VadErrorKind {
    StartError,
    EndError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VadDiscrepancy {
    pub group_id: usize,
    pub sub_idx: usize,
    pub kind: VadErrorKind,
    pub sub_ms: u32,
    pub speech_ms: u32,
    pub error_ms: u32,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VadAuditReport {
    pub total_turns: usize,
    pub total_subs: usize,
    pub turn_start_pass: usize,
    pub turn_end_pass: usize,
    pub discrepancies: Vec<VadDiscrepancy>,
}

/// 解析 SRT 時間戳 (HH:MM:SS,mmm 或 HH:MM:SS.mmm) 為毫秒
/// @outputs: 2
pub fn parse_srt_timestamp_ms(ts: &str) -> Option<u32> {
    let cleaned = ts.trim().replace('.', ",");
    let mut parts = cleaned.split(':');
    let h: u32 = parts.next()?.trim().parse().ok()?;
    let m: u32 = parts.next()?.trim().parse().ok()?;
    let sec_ms = parts.next()?;
    let mut sm = sec_ms.split(',');
    let s: u32 = sm.next()?.trim().parse().ok()?;
    let ms_str = sm.next()?.trim();
    if ms_str.len() != 3 {
        return None;
    }
    let ms: u32 = ms_str.parse().ok()?;
    if m >= 60 || s >= 60 {
        return None;
    }
    h.checked_mul(3_600_000)?
        .checked_add(m.checked_mul(60_000)?)?
        .checked_add(s.checked_mul(1000)?)?
        .checked_add(ms)
}

/// 將毫秒格式化為標準 SRT 時間戳 HH:MM:SS,mmm
/// @outputs: 1
/// @domain: total_ms [0..359999999]
pub fn format_srt_timestamp_ms(total_ms: u32) -> String {
    if total_ms > 359_999_999 {
        return "99:59:59,999".to_string();
    }
    let h = total_ms / 3_600_000;
    let rem_h = total_ms % 3_600_000;
    let m = rem_h / 60_000;
    let rem_m = rem_h % 60_000;
    let s = rem_m / 1000;
    let ms = rem_m % 1000;
    format!("{h:02}:{m:02}:{s:02},{ms:03}")
}

/// 判斷字元是否屬於應剔除的標點/空白符號
/// @outputs: 1
fn is_ignored_subtitle_char(ch: char) -> bool {
    ch.is_whitespace()
        || matches!(
            ch,
            '.' | ','
                | '!'
                | '?'
                | '，'
                | '。'
                | '！'
                | '？'
                | '、'
                | '：'
                | '；'
                | '「'
                | '」'
                | '『'
                | '』'
                | '('
                | ')'
                | '（'
                | '）'
                | '-'
                | '—'
                | '"'
                | '\''
                | '“'
                | '”'
        )
}

/// 統計可閱讀字元數（剔除標點、空白與 \N 換行符；若剔除後為 0 則回傳原字數）
/// @outputs: 1
pub fn count_readable_subtitle_chars(text: &str) -> u32 {
    let without_nl = text.replace("\\N", "");
    let cleaned_len = without_nl
        .chars()
        .filter(|&c| !is_ignored_subtitle_char(c))
        .count();
    if cleaned_len > 0 {
        cleaned_len as u32
    } else {
        text.chars().count() as u32
    }
}

/// 解析單一 SRT 字幕區塊
/// @outputs: 2
fn parse_single_srt_block(block: &str) -> Option<SrtEntry> {
    let lines: Vec<&str> = block
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 3 {
        return None;
    }
    let idx: usize = lines[0].parse().ok()?;
    let arrow_pos = lines[1].find("-->")?;
    let start_ms = parse_srt_timestamp_ms(&lines[1][..arrow_pos])?;
    let end_ms = parse_srt_timestamp_ms(&lines[1][arrow_pos + 3..])?;
    let duration_ms = end_ms.saturating_sub(start_ms);
    let text = lines[2..].join("\n");
    let readable_chars = count_readable_subtitle_chars(&text);
    let cps_tenths = if duration_ms == 0 {
        9999
    } else {
        subtitle_cps_tenths(readable_chars.min(5000), duration_ms.min(600_000)).unwrap_or(9999)
    };

    Some(SrtEntry {
        idx,
        start_ms,
        end_ms,
        duration_ms,
        text,
        readable_chars,
        cps_tenths,
    })
}

/// 解析完整 SRT 字幕檔案內容
/// @outputs: 1
pub fn parse_srt_content(content: &str) -> Vec<SrtEntry> {
    let stripped_bom = content.trim_start_matches('\u{FEFF}');
    let normalized = stripped_bom.replace("\r\n", "\n");
    let mut entries = Vec::new();
    for block in normalized.split("\n\n") {
        let trimmed = block.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(entry) = parse_single_srt_block(trimmed) {
            entries.push(entry);
        }
    }
    entries
}

/// 執行 CPS 語速三段分級與異常短幀/倒錯檢測
/// @outputs: 2
/// @domain: red_tenths [10..500]
/// @domain: orange_tenths [10..500]
pub fn audit_srt_cps(
    subs: &[SrtEntry],
    red_tenths: u32,
    orange_tenths: u32,
) -> Option<CpsAuditReport> {
    if !(10..=500).contains(&red_tenths) || !(10..=500).contains(&orange_tenths) {
        return None;
    }

    let mut red_entries = Vec::new();
    let mut orange_count = 0usize;
    let mut inverted_entries = Vec::new();
    let mut dummy_entries = Vec::new();

    for entry in subs {
        if entry.end_ms <= entry.start_ms {
            inverted_entries.push(entry.clone());
        } else if entry.cps_tenths > red_tenths {
            red_entries.push(entry.clone());
        } else if entry.cps_tenths > orange_tenths {
            orange_count += 1;
        }

        if entry.duration_ms <= 550 && entry.readable_chars >= 8 {
            dummy_entries.push(entry.clone());
        }
    }

    let red = red_entries.len();
    let inverted = inverted_entries.len();
    let dummy_half_sec = dummy_entries.len();
    let green = subs.len().saturating_sub(red + orange_count + inverted);

    Some(CpsAuditReport {
        total: subs.len(),
        red,
        orange: orange_count,
        green,
        inverted,
        dummy_half_sec,
        red_entries,
        inverted_entries,
        dummy_entries,
    })
}

/// 掃描單句文本內的 ASCII 英文字詞（不含白名單專有名詞）
/// @outputs: 1
fn collect_residual_ascii_words(text: &str, allowlist: &[&str]) -> Vec<String> {
    let mut hits = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            current.push(ch);
        } else if !current.is_empty() {
            if !allowlist.contains(&current.as_str()) {
                hits.push(current.clone());
            }
            current.clear();
        }
    }
    if !current.is_empty() && !allowlist.contains(&current.as_str()) {
        hits.push(current);
    }
    hits
}

/// 掃描 SRT 字幕內尚未翻譯的英文單詞（預設放行 Krishna / Balarama / KRISHNA）
/// @outputs: 1
pub fn find_residual_english_lines(
    subs: &[SrtEntry],
    extra_allowlist: &[&str],
) -> Vec<ResidualEnglishHit> {
    let mut allow: Vec<&str> = vec!["Krishna", "Balarama", "KRISHNA"];
    allow.extend_from_slice(extra_allowlist);

    let mut results = Vec::new();
    for s in subs {
        let words = collect_residual_ascii_words(&s.text, &allow);
        if !words.is_empty() {
            results.push(ResidualEnglishHit {
                idx: s.idx,
                start_ms: s.start_ms,
                end_ms: s.end_ms,
                words,
                text: s.text.clone(),
            });
        }
    }
    results
}

/// 從 JSON 片段讀取布林值
/// @outputs: 2
fn extract_json_bool(obj: &str, key: &str) -> Option<bool> {
    let pat = format!("\"{key}\"");
    let pos = obj.find(&pat)?;
    let colon = obj[pos + pat.len()..].find(':')?;
    let rest = obj[pos + pat.len() + colon + 1..].trim_start();
    if rest.starts_with("true") {
        Some(true)
    } else if rest.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

/// 從 JSON 片段讀取無號整數
/// @outputs: 2
fn extract_json_usize(obj: &str, key: &str) -> Option<usize> {
    let pat = format!("\"{key}\"");
    let pos = obj.find(&pat)?;
    let colon = obj[pos + pat.len()..].find(':')?;
    let digits: String = obj[pos + pat.len() + colon + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// 從 JSON 片段讀取秒數浮點並安全換算為整數毫秒（round half-to-even / round）
/// @outputs: 2
fn extract_json_seconds_as_ms(obj: &str, key: &str) -> Option<u32> {
    let pat = format!("\"{key}\"");
    let pos = obj.find(&pat)?;
    let colon = obj[pos + pat.len()..].find(':')?;
    let num_str: String = obj[pos + pat.len() + colon + 1..]
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let secs: f64 = num_str.parse().ok()?;
    if secs < 0.0 || secs > 86_400.0 {
        return None;
    }
    Some((secs * 1000.0).round() as u32)
}

/// 從 JSON 片段讀取整數陣列 (subtitle_indices)
/// @outputs: 1
fn extract_json_usize_array(obj: &str, key: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let pat = format!("\"{key}\"");
    let Some(pos) = obj.find(&pat) else {
        return out;
    };
    let Some(lbr) = obj[pos + pat.len()..].find('[') else {
        return out;
    };
    let after_lbr = &obj[pos + pat.len() + lbr + 1..];
    let Some(rbr) = after_lbr.find(']') else {
        return out;
    };
    for part in after_lbr[..rbr].split(',') {
        if let Ok(v) = part.trim().parse::<usize>() {
            out.push(v);
        }
    }
    out
}

/// 從 JSON 片段讀取字串欄位
/// @outputs: 2
fn extract_json_string(obj: &str, key: &str) -> Option<String> {
    let pat = format!("\"{key}\"");
    let pos = obj.find(&pat)?;
    let colon = obj[pos + pat.len()..].find(':')?;
    let rest = obj[pos + pat.len() + colon + 1..].trim_start();
    if !rest.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut esc = false;
    for ch in rest[1..].chars() {
        if esc {
            out.push(ch);
            esc = false;
        } else if ch == '\\' {
            esc = true;
        } else if ch == '"' {
            return Some(out);
        } else {
            out.push(ch);
        }
    }
    None
}

/// 從 sentence_speech_boundaries.json 解析所有句段邊界物件
/// @outputs: 1
pub fn parse_speech_boundaries_json(json_str: &str) -> Vec<SpeechSentenceBoundary> {
    let mut items = Vec::new();
    let Some(key_pos) = json_str.find("\"sentences\"") else {
        return items;
    };
    let Some(arr_rel) = json_str[key_pos..].find('[') else {
        return items;
    };
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    let mut obj_start = 0usize;

    for (rel_idx, ch) in json_str[key_pos + arr_rel..].char_indices() {
        let abs = key_pos + arr_rel + rel_idx;
        if esc {
            esc = false;
            continue;
        }
        if ch == '\\' && in_str {
            esc = true;
            continue;
        }
        if ch == '"' {
            in_str = !in_str;
            continue;
        }
        if in_str {
            continue;
        }
        if ch == '{' && depth == 0 {
            obj_start = abs;
            depth = 1;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' && depth == 1 {
            depth = 0;
            let obj = &json_str[obj_start..=abs];
            if let Some(b) = parse_single_boundary_object(obj) {
                items.push(b);
            }
        } else if ch == '}' {
            depth = depth.saturating_sub(1);
        } else if ch == ']' && depth == 0 {
            break;
        }
    }
    items
}

/// 將單一 boundary JSON 物件片段轉換為結構體
/// @outputs: 2
fn parse_single_boundary_object(obj: &str) -> Option<SpeechSentenceBoundary> {
    let group_id = extract_json_usize(obj, "group_id")?;
    let has_speech = extract_json_bool(obj, "has_speech").unwrap_or(false);
    let subtitle_indices = extract_json_usize_array(obj, "subtitle_indices");
    let speech_start_ms = extract_json_seconds_as_ms(obj, "detected_speech_start").unwrap_or(0);
    let speech_end_ms = extract_json_seconds_as_ms(obj, "detected_speech_end").unwrap_or(0);
    let text = extract_json_string(obj, "text").unwrap_or_default();

    Some(SpeechSentenceBoundary {
        group_id,
        has_speech,
        subtitle_indices,
        speech_start_ms,
        speech_end_ms,
        text,
    })
}

/// 檢驗單一句段之起始與結束 VAD 時間誤差
/// @outputs: 1
fn evaluate_single_vad_turn(
    sent: &SpeechSentenceBoundary,
    by_idx: &BTreeMap<usize, &SrtEntry>,
    tol_ms: u32,
    turn_start_pass: &mut usize,
    turn_end_pass: &mut usize,
    discrepancies: &mut Vec<VadDiscrepancy>,
) {
    if !sent.has_speech || sent.subtitle_indices.is_empty() {
        return;
    }
    let first_idx = sent.subtitle_indices[0];
    let last_idx = sent.subtitle_indices[sent.subtitle_indices.len() - 1];
    let (Some(&first_sub), Some(&last_sub)) = (by_idx.get(&first_idx), by_idx.get(&last_idx)) else {
        return;
    };
    let _ = subtitle_vad_drift_ok(
        first_sub.start_ms.min(86_400_000),
        last_sub.end_ms.clamp(1, 86_400_000),
        sent.speech_start_ms.min(86_400_000),
        sent.speech_end_ms.clamp(1, 86_400_000),
        tol_ms,
    );
    let d_start = first_sub.start_ms.abs_diff(sent.speech_start_ms);
    if d_start <= tol_ms {
        *turn_start_pass += 1;
    } else {
        discrepancies.push(VadDiscrepancy {
            group_id: sent.group_id,
            sub_idx: first_sub.idx,
            kind: VadErrorKind::StartError,
            sub_ms: first_sub.start_ms,
            speech_ms: sent.speech_start_ms,
            error_ms: d_start,
            text: first_sub.text.clone(),
        });
    }
    let d_end = last_sub.end_ms.abs_diff(sent.speech_end_ms);
    if d_end <= tol_ms {
        *turn_end_pass += 1;
    } else {
        discrepancies.push(VadDiscrepancy {
            group_id: sent.group_id,
            sub_idx: last_sub.idx,
            kind: VadErrorKind::EndError,
            sub_ms: last_sub.end_ms,
            speech_ms: sent.speech_end_ms,
            error_ms: d_end,
            text: last_sub.text.clone(),
        });
    }
}

/// 執行 VAD 語音活動邊界與 SRT 字幕時間對齊驗收 Hook
/// @outputs: 2
/// @domain: tol_ms [1..5000]
pub fn evaluate_vad_boundaries(
    boundaries: &[SpeechSentenceBoundary],
    subs: &[SrtEntry],
    tol_ms: u32,
) -> Option<VadAuditReport> {
    if !(1..=5000).contains(&tol_ms) {
        return None;
    }

    let mut by_idx: BTreeMap<usize, &SrtEntry> = BTreeMap::new();
    for s in subs {
        by_idx.insert(s.idx, s);
    }

    let mut turn_start_pass = 0usize;
    let mut turn_end_pass = 0usize;
    let mut discrepancies = Vec::new();
    for sent in boundaries {
        evaluate_single_vad_turn(
            sent,
            &by_idx,
            tol_ms,
            &mut turn_start_pass,
            &mut turn_end_pass,
            &mut discrepancies,
        );
    }

    Some(VadAuditReport {
        total_turns: boundaries.len(),
        total_subs: subs.len(),
        turn_start_pass,
        turn_end_pass,
        discrepancies,
    })
}

/// 依超標清單執行語音邊界貼合修正並解除相鄰字幕時間碰撞
/// @outputs: 2
/// @domain: tol_ms [1..5000]
pub fn snap_subtitles_to_vad_boundaries(
    boundaries: &[SpeechSentenceBoundary],
    subs: &[SrtEntry],
    tol_ms: u32,
) -> Option<Vec<SrtEntry>> {
    let report = evaluate_vad_boundaries(boundaries, subs, tol_ms)?;
    let mut fixed = subs.to_vec();
    let mut idx_pos = BTreeMap::new();
    for (pos, entry) in fixed.iter().enumerate() {
        idx_pos.insert(entry.idx, pos);
    }

    for disc in &report.discrepancies {
        let Some(&pos) = idx_pos.get(&disc.sub_idx) else {
            continue;
        };
        match disc.kind {
            VadErrorKind::StartError => fixed[pos].start_ms = disc.speech_ms,
            VadErrorKind::EndError => fixed[pos].end_ms = disc.speech_ms,
        }
        fixed[pos].duration_ms = fixed[pos].end_ms.saturating_sub(fixed[pos].start_ms);
    }

    for i in 0..fixed.len().saturating_sub(1) {
        if fixed[i].end_ms >= fixed[i + 1].start_ms.saturating_sub(30) {
            fixed[i].end_ms = fixed[i + 1].start_ms.saturating_sub(40);
            fixed[i].duration_ms = fixed[i].end_ms.saturating_sub(fixed[i].start_ms);
        }
    }

    Some(fixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// @outputs: 1
    #[test]
    fn test_parse_srt_and_cps_audit_golden_and_defects() {
        let srt = "\u{FEFF}1\n00:00:01,000 --> 00:00:03,000\n你今天好嗎？\n\n2\n00:00:04,000 --> 00:00:04,500\n這句字幕字數超過十個字元很急促\n\n3\n00:00:06,000 --> 00:00:05,000\n時間倒錯字幕\n";
        let entries = parse_srt_content(srt);
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].readable_chars, 5);
        assert_eq!(entries[0].cps_tenths, 25);
        assert_eq!(entries[1].readable_chars, 15);
        assert_eq!(entries[1].duration_ms, 500);

        let report = audit_srt_cps(&entries, 120, 90);
        assert!(report.is_some());
        if let Some(r) = report {
            assert_eq!(r.total, 3);
            assert_eq!(r.green, 1);
            assert_eq!(r.red, 1);
            assert_eq!(r.inverted, 1);
            assert_eq!(r.dummy_half_sec, 1);
        }
    }

    /// @outputs: 1
    #[test]
    fn test_residual_english_lines_filter() {
        let srt = "1\n00:00:01,000 --> 00:00:03,000\nKrishna 與 Balarama 來到森林\n\n2\n00:00:04,000 --> 00:00:06,000\n請注意 hello world 尚未翻譯\n";
        let entries = parse_srt_content(srt);
        let hits = find_residual_english_lines(&entries, &[]);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].idx, 2);
        assert_eq!(hits[0].words, vec!["hello".to_string(), "world".to_string()]);
    }

    /// @outputs: 1
    #[test]
    fn test_vad_boundaries_eval_and_snap_resolution() {
        let json = r#"{
  "sentences": [
    {"group_id": 1, "has_speech": true, "subtitle_indices": [1], "detected_speech_start": 1.000, "detected_speech_end": 2.500, "text": "第一句"},
    {"group_id": 2, "has_speech": true, "subtitle_indices": [2], "detected_speech_start": 3.000, "detected_speech_end": 4.200, "text": "第二句"}
  ]
}"#;
        let boundaries = parse_speech_boundaries_json(json);
        assert_eq!(boundaries.len(), 2);

        let srt = "1\n00:00:01,050 --> 00:00:02,800\n第一句\n\n2\n00:00:03,300 --> 00:00:04,200\n第二句\n";
        let subs = parse_srt_content(srt);
        let eval = evaluate_vad_boundaries(&boundaries, &subs, 100);
        assert!(eval.is_some());
        if let Some(ev) = eval {
            assert_eq!(ev.turn_start_pass, 1);
            assert_eq!(ev.turn_end_pass, 1);
            assert_eq!(ev.discrepancies.len(), 2);
        }

        let snapped = snap_subtitles_to_vad_boundaries(&boundaries, &subs, 100);
        assert!(snapped.is_some());
        if let Some(fixed) = snapped {
            assert_eq!(fixed[0].end_ms, 2500);
            assert_eq!(fixed[1].start_ms, 3000);
            let recheck = evaluate_vad_boundaries(&boundaries, &fixed, 100);
            assert!(recheck.is_some_and(|r| r.discrepancies.is_empty()));
        }
    }
}
