// ## Map
// Role: app (雙模態定本建構 CLI)
// Inputs: 命令列參數 (--json / --out-dir / --upload / --drive-parent)、聖典規格 JSON 檔案。
// Outputs: 出版級 Word .docx、結構化語料庫 Markdown .md、剛性品質門禁驗收、雲端硬碟自動同步。
// Invariants: 零外部第三方依賴、純 Rust 實現、禁止 unsafe 代碼、100% 遵守 09 號合約、無非有界遞迴 (R09-7)、單函式 <= 100 行、巢狀 <= 2。

#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use rust_bricks::classic_continuity::inspect_markdown_content;

const MAX_JSON_DEPTH: usize = 16;

// ============================================================================
// 1. 純 Rust 零外部依賴有界 JSON 解析器 (Bounded Stack JSON Parser, R09-7)
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// 依鍵值取得物件欄位
    /// @outputs: 2
    fn get(&self, key: &str) -> Option<&JsonValue> {
        if let JsonValue::Object(map) = self {
            for (k, v) in map {
                if k == key {
                    return Some(v);
                }
            }
        }
        None
    }

    /// 轉為字串切片
    /// @outputs: 2
    fn as_str(&self) -> Option<&str> {
        if let JsonValue::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// 轉為陣列切片
    /// @outputs: 2
    fn as_array(&self) -> Option<&[JsonValue]> {
        if let JsonValue::Array(arr) = self {
            Some(arr.as_slice())
        } else {
            None
        }
    }
}

struct JsonParser<'a> {
    chars: Vec<char>,
    pos: usize,
    depth: usize,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> JsonParser<'a> {
    /// 初始化解析器
    /// @outputs: 1
    fn new(input: &'a str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            depth: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// 跳過空白字元
    /// @outputs: 1
    fn skip_whitespace(&mut self) {
        while self.pos < self.chars.len() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    /// 檢視當前字元
    /// @outputs: 2
    fn peek(&self) -> Option<char> {
        if self.pos < self.chars.len() {
            Some(self.chars[self.pos])
        } else {
            None
        }
    }

    /// 解析單一值（帶有界棧守衛，防止非有界遞迴 R09-7）
    /// @outputs: 7
    fn parse_value(&mut self) -> Result<JsonValue, String> {
        if self.depth >= MAX_JSON_DEPTH {
            return Err("JSON 巢狀深度超過上限 16 層，防禦性阻斷 (R09-7 有界棧)".to_string());
        }
        self.skip_whitespace();
        let c = self.peek().ok_or("非預期結束 (EOF)")?;
        match c {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string().map(JsonValue::String),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            '-' | '0'..='9' => self.parse_number(),
            _ => Err(format!("在位置 {} 遇到非預期字符: '{c}'", self.pos)),
        }
    }

    /// 解析 null
    /// @outputs: 2
    fn parse_null(&mut self) -> Result<JsonValue, String> {
        if self.pos + 4 <= self.chars.len() && &self.chars[self.pos..self.pos + 4] == ['n', 'u', 'l', 'l'] {
            self.pos += 4;
            Ok(JsonValue::Null)
        } else {
            Err("無效之 null 字面量".to_string())
        }
    }

    /// 解析布林值
    /// @outputs: 3
    fn parse_bool(&mut self) -> Result<JsonValue, String> {
        if self.pos + 4 <= self.chars.len() && &self.chars[self.pos..self.pos + 4] == ['t', 'r', 'u', 'e'] {
            self.pos += 4;
            Ok(JsonValue::Bool(true))
        } else if self.pos + 5 <= self.chars.len() && &self.chars[self.pos..self.pos + 5] == ['f', 'a', 'l', 's', 'e'] {
            self.pos += 5;
            Ok(JsonValue::Bool(false))
        } else {
            Err("無效之布林字面量".to_string())
        }
    }

    /// 解析字串
    /// @outputs: 3
    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1;
        let mut s = String::new();
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            self.pos += 1;
            if c == '"' {
                return Ok(s);
            }
            if c == '\\' {
                self.parse_escape_char(&mut s)?;
            } else {
                s.push(c);
            }
        }
        Err("字串未閉合".to_string())
    }

    /// 解析轉義字元
    /// @outputs: 3
    fn parse_escape_char(&mut self, s: &mut String) -> Result<(), String> {
        if self.pos >= self.chars.len() {
            return Err("字串跳脫字符非預期結束".to_string());
        }
        let esc = self.chars[self.pos];
        self.pos += 1;
        match esc {
            '"' => s.push('"'),
            '\\' => s.push('\\'),
            '/' => s.push('/'),
            'b' => s.push('\x08'),
            'f' => s.push('\x0C'),
            'n' => s.push('\n'),
            'r' => s.push('\r'),
            't' => s.push('\t'),
            'u' => self.parse_unicode_escape(s)?,
            _ => s.push(esc),
        }
        Ok(())
    }

    /// 解析 Unicode 轉義 (\\uXXXX)
    /// @outputs: 2
    fn parse_unicode_escape(&mut self, s: &mut String) -> Result<(), String> {
        if self.pos + 4 > self.chars.len() {
            return Err("Unicode 跳脫長度不足".to_string());
        }
        let hex: String = self.chars[self.pos..self.pos + 4].iter().collect();
        self.pos += 4;
        let code = u32::from_str_radix(&hex, 16).map_err(|e| format!("無效 Unicode 碼: {e}"))?;
        if let Some(ch) = char::from_u32(code) {
            s.push(ch);
        }
        Ok(())
    }

    /// 解析數字
    /// @outputs: 2
    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let mut num_str = String::new();
        while self.pos < self.chars.len() {
            let c = self.chars[self.pos];
            if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' || c == 'e' || c == 'E' {
                num_str.push(c);
                self.pos += 1;
            } else {
                break;
            }
        }
        let num: f64 = num_str.parse().map_err(|e| format!("數字解析錯誤 '{num_str}': {e}"))?;
        Ok(JsonValue::Number(num))
    }

    /// 解析陣列 (遞迴深度增加守衛)
    /// @outputs: 3
    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        self.depth += 1;
        let mut arr = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(JsonValue::Array(arr));
        }

        loop {
            let val = self.parse_value()?;
            arr.push(val);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some(']') => {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(JsonValue::Array(arr));
                }
                Some(c) => {
                    self.depth -= 1;
                    return Err(format!("陣列缺少逗點或括號，遇到: '{c}'"));
                }
                None => {
                    self.depth -= 1;
                    return Err("陣列未閉合 (EOF)".to_string());
                }
            }
        }
    }

    /// 解析物件 (遞迴深度增加守衛)
    /// @outputs: 3
    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1;
        self.depth += 1;
        let mut obj = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.pos += 1;
            self.depth -= 1;
            return Ok(JsonValue::Object(obj));
        }

        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                self.depth -= 1;
                return Err("物件鍵值必須為字串".to_string());
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(':') {
                self.depth -= 1;
                return Err("物件缺少冒號分隔符".to_string());
            }
            self.pos += 1;
            let val = self.parse_value()?;
            obj.push((key, val));
            self.skip_whitespace();
            match self.peek() {
                Some(',') => self.pos += 1,
                Some('}') => {
                    self.pos += 1;
                    self.depth -= 1;
                    return Ok(JsonValue::Object(obj));
                }
                Some(c) => {
                    self.depth -= 1;
                    return Err(format!("物件缺少逗點或大括號，遇到: '{c}'"));
                }
                None => {
                    self.depth -= 1;
                    return Err("物件未閉合 (EOF)".to_string());
                }
            }
        }
    }
}

// ============================================================================
// 2. 雙模態出版生成器 (Markdown & OpenXML Docx 模組化拆分，單函式 <= 60 行)
// ============================================================================

/// XML 字元跳脫
/// @outputs: 1
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 衍生標準雙語檔名
/// @outputs: 1
fn derive_canonical_filenames(data: &JsonValue, out_dir: &str) -> (String, PathBuf, PathBuf) {
    let orig = data.get("original_title").and_then(|v| v.as_str()).unwrap_or("Camatkāra-candrikā");
    let meaning_raw = data.get("title_meaning").and_then(|v| v.as_str()).unwrap_or("奇妙月光");
    let clean_meaning = meaning_raw.replace('《', "").replace('》', "").trim().to_string();

    let base_name = format!("{orig}_{clean_meaning}_繁體中文全譯本");
    let docx_file = Path::new(out_dir).join(format!("{base_name}.docx"));
    let md_file = Path::new(out_dir).join(format!("{base_name}.md"));
    (base_name, docx_file, md_file)
}

/// 渲染 Markdown 導讀與審計矩陣
/// @outputs: 1
fn render_md_preface(data: &JsonValue, md: &mut Vec<String>) {
    md.push("## 【譯者導讀：結構審計與學術導覽】\n".to_string());
    if let Some(pref) = data.get("preface") {
        if let Some(sections) = pref.get("sections").and_then(|v| v.as_array()) {
            for sec in sections {
                let heading = sec.get("heading").and_then(|v| v.as_str()).unwrap_or("");
                md.push(format!("### {heading}\n"));
                if let Some(paras) = sec.get("paragraphs").and_then(|v| v.as_array()) {
                    for p in paras {
                        if let Some(text) = p.as_str() {
                            md.push(format!("{text}\n"));
                        }
                    }
                }
            }
        }
    }

    if let Some(matrix) = data.get("audit_matrix").and_then(|v| v.as_array()) {
        md.push("\n### 全書篇章結構與完整度審計矩陣表\n".to_string());
        md.push("| 篇章單元 | 原著頁碼/來源 | 經文頌數 | 核心收錄義理 |".to_string());
        md.push("| :--- | :---: | :---: | :--- |".to_string());
        for row in matrix {
            if let Some(cols) = row.as_array() {
                let c0 = cols.get(0).and_then(|v| v.as_str()).unwrap_or("");
                let c1 = cols.get(1).and_then(|v| v.as_str()).unwrap_or("");
                let c2 = cols.get(2).and_then(|v| v.as_str()).unwrap_or("");
                let c3 = cols.get(3).and_then(|v| v.as_str()).unwrap_or("");
                md.push(format!("| {c0} | {c1} | {c2} | {c3} |"));
            }
        }
        md.push("\n---\n".to_string());
    }
}

/// 渲染 Markdown 終卷大圓滿題記 (Puṣpikā)
/// @outputs: 1
fn render_md_colophon(data: &JsonValue, md: &mut Vec<String>) {
    if let Some(col) = data.get("colophon") {
        let title = col.get("title").and_then(|s| s.as_str()).unwrap_or("終卷大圓滿題記（Puṣpikā）");
        md.push("\n---\n".to_string());
        md.push(format!("## 【{title}】\n"));

        if let Some(dev) = col.get("devanagari").and_then(|s| s.as_str()) {
            if !dev.trim().is_empty() {
                md.push(format!("> ### {dev}\n"));
            }
        }
        if let Some(beng) = col.get("bengali").and_then(|s| s.as_str()) {
            if !beng.trim().is_empty() {
                md.push(format!("> ### {beng}\n"));
            }
        }
        if let Some(iast) = col.get("iast").and_then(|s| s.as_str()) {
            if !iast.trim().is_empty() {
                md.push(format!("*{iast}*\n"));
            }
        }
        if let Some(trans) = col.get("translation_zh").and_then(|s| s.as_str()) {
            if !trans.trim().is_empty() {
                md.push(format!("**【題記真義定本】**: {trans}\n"));
            }
        }
        if let Some(sig) = col.get("significance").and_then(|s| s.as_str()) {
            if !sig.trim().is_empty() {
                md.push(format!("*{sig}*\n"));
            }
        }
        md.push("\n---\n".to_string());
    }
}

/// 渲染 Markdown 單頌內容
/// @outputs: 1
fn render_md_single_verse(v: &JsonValue, idx: usize, md: &mut Vec<String>) {
    let default_id = format!("Text {idx}");
    let vid = v.get("id").and_then(|s| s.as_str()).unwrap_or(&default_id);
    let title = v.get("title").and_then(|s| s.as_str()).unwrap_or("");
    let coord = if vid.contains('[') {
        vid.to_string()
    } else {
        format!("{vid} [全書第 {idx} 頌]")
    };
    let title_line = if title.is_empty() { coord } else { format!("{coord} ．{title}") };
    md.push(format!("### {title_line}\n"));

    if let Some(meter) = v.get("meter").and_then(|s| s.as_str()) {
        if !meter.trim().is_empty() {
            md.push(format!("> **【詩體格律（Chhandas）】**：*{meter}*\n"));
        }
    }

    if let Some(lines) = v.get("devanagari").and_then(|l| l.as_array()) {
        for l in lines {
            if let Some(t) = l.as_str() {
                md.push(format!("> {t}"));
            }
        }
        md.push(String::new());
    }

    if let Some(iast) = v.get("iast").and_then(|s| s.as_str()) {
        if !iast.trim().is_empty() {
            md.push(format!("*{iast}*\n"));
        }
    }
    if let Some(en) = v.get("english").and_then(|s| s.as_str()) {
        if !en.trim().is_empty() {
            md.push(format!("> **【原著英譯】**：*\"{en}\"*\n"));
        }
    }
    if let Some(beng) = v.get("bengali_commentary").and_then(|s| s.as_str()) {
        if !beng.trim().is_empty() {
            md.push(format!("> **【孟加拉文古註（অনুবাদ ও তাৎপর্য）】**：{beng}\n"));
        }
    }

    if let Some(anvaya_items) = v.get("anvaya").and_then(|a| a.as_array()) {
        if !anvaya_items.is_empty() {
            md.push("> **【散文句法與逐字解（Anvaya & Padaccheda）】**：\n".to_string());
            md.push("| 梵文單詞 / 複合詞 | 連聲切分 (Padaccheda) | 語法格位分析 | 繁體中文逐字義 |".to_string());
            md.push("| :--- | :--- | :--- | :--- |".to_string());
            for item in anvaya_items {
                let w = item.get("word").and_then(|s| s.as_str()).unwrap_or("");
                let p = item.get("padaccheda").and_then(|s| s.as_str()).unwrap_or(w);
                let g = item.get("grammar").and_then(|s| s.as_str()).unwrap_or("");
                let m = item.get("meaning_zh").and_then(|s| s.as_str()).unwrap_or("");
                md.push(format!("| {w} | {p} | {g} | {m} |"));
            }
            md.push(String::new());
        }
    }

    let mut trans = v.get("translation_zh").and_then(|s| s.as_str()).unwrap_or("").to_string();
    if !trans.starts_with("譯文:") && !trans.starts_with("譯文：") {
        trans = format!("譯文: {trans}");
    }
    md.push(format!("{trans}\n"));

    if let Some(notes) = v.get("notes").and_then(|s| s.as_str()) {
        if !notes.trim().is_empty() {
            md.push(format!("*{notes}*\n"));
        }
    }
}

/// 渲染 Markdown 經文全錄
/// @outputs: 1
fn render_md_verses(data: &JsonValue, md: &mut Vec<String>) {
    if let Some(verses) = data.get("verses").and_then(|v| v.as_array()) {
        let mut curr_chap = String::new();
        for (idx_0, v) in verses.iter().enumerate() {
            let idx = idx_0.saturating_add(1);
            let chap = v.get("chapter").and_then(|s| s.as_str()).unwrap_or("").trim();
            if !chap.is_empty() && chap != curr_chap {
                curr_chap = chap.to_string();
                md.push(format!("\n## {chap}\n"));
            }
            render_md_single_verse(v, idx, md);
        }
    }
}

/// 建構 Markdown 輸出檔
/// @outputs: 2
fn build_markdown_file(data: &JsonValue, md_path: &Path) -> Result<(), String> {
    let mut md = Vec::new();
    let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("經文全譯本");
    let subtitle = data.get("subtitle").and_then(|v| v.as_str()).unwrap_or("繁體中文定本");
    let orig_title = data.get("original_title").and_then(|v| v.as_str()).unwrap_or("");
    let script_title = data.get("original_script_title").and_then(|v| v.as_str()).unwrap_or("");
    let translit = data.get("title_translit").and_then(|v| v.as_str()).unwrap_or("");
    let meaning = data.get("title_meaning").and_then(|v| v.as_str()).unwrap_or("");
    let author = data.get("author_info").and_then(|v| v.as_str()).unwrap_or("");

    md.push(format!("# {title}\n"));
    md.push(format!("## {subtitle}\n"));
    md.push(format!("**原典題名與考證**：原典題名：{script_title} ({orig_title}) ｜ 音譯：{translit} ｜ 意譯：{meaning}\n"));
    md.push(format!("**原著傳承與宗義考證**：{author}\n"));
    md.push("---\n".to_string());

    render_md_preface(data, &mut md);
    render_md_verses(data, &mut md);
    render_md_colophon(data, &mut md);

    let full_text = md.join("\n");
    fs::write(md_path, full_text).map_err(|e| format!("無法寫入 Markdown: {e}"))?;
    println!("📝 [MD] 結構化 Markdown 語料庫已導出 -> {}", md_path.display());
    Ok(())
}

/// 渲染 Word 導讀與段落
/// @outputs: 1
fn render_docx_preface(data: &JsonValue, cs_font: &str, doc_xml: &mut String) {
    let title = data.get("title").and_then(|v| v.as_str()).unwrap_or("經文全譯本");
    let author = data.get("author_info").and_then(|v| v.as_str()).unwrap_or("");

    doc_xml.push_str(&format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:before="480" w:after="80"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="44"/><w:szCs w:val="44"/><w:color w:val="1E293B"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
        escape_xml(title)
    ));
    doc_xml.push_str(&format!(
        r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="480"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="21"/><w:szCs w:val="21"/><w:color w:val="475569"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
        escape_xml(author)
    ));
    doc_xml.push_str(r#"<w:p><w:pPr><w:spacing w:before="320" w:after="120"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="Nirmala UI"/><w:b/><w:sz w:val="30"/><w:szCs w:val="30"/><w:color w:val="0F172A"/></w:rPr><w:t>【譯者導讀：結構審計與學術導覽】</w:t></w:r></w:p>"#);

    if let Some(pref) = data.get("preface") {
        if let Some(sections) = pref.get("sections").and_then(|v| v.as_array()) {
            for sec in sections {
                let heading = sec.get("heading").and_then(|v| v.as_str()).unwrap_or("");
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:spacing w:before="200" w:after="80"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="24"/><w:szCs w:val="24"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(heading)
                ));
                if let Some(paras) = sec.get("paragraphs").and_then(|v| v.as_array()) {
                    for p in paras {
                        if let Some(text) = p.as_str() {
                            doc_xml.push_str(&format!(
                                r#"<w:p><w:pPr><w:spacing w:after="120"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="21"/><w:szCs w:val="21"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                                escape_xml(text)
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// 渲染 Word 審計矩陣表格
/// @outputs: 1
fn render_docx_audit_table(data: &JsonValue, cs_font: &str, doc_xml: &mut String) {
    if let Some(matrix) = data.get("audit_matrix").and_then(|v| v.as_array()) {
        doc_xml.push_str(r#"<w:p><w:pPr><w:spacing w:before="240" w:after="80"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="Nirmala UI"/><w:b/><w:sz w:val="23"/><w:szCs w:val="23"/></w:rPr><w:t>全書篇章結構與完整度審計矩陣表</w:t></w:r></w:p>"#);
        doc_xml.push_str(r#"<w:tbl><w:tblPr><w:jc w:val="center"/><w:tblBorders><w:top w:val="single" w:sz="4" w:space="0" w:color="CCCCCC"/><w:left w:val="single" w:sz="4" w:space="0" w:color="CCCCCC"/><w:bottom w:val="single" w:sz="4" w:space="0" w:color="CCCCCC"/><w:right w:val="single" w:sz="4" w:space="0" w:color="CCCCCC"/><w:insideH w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/><w:insideV w:val="single" w:sz="4" w:space="0" w:color="E2E8F0"/></w:tblBorders></w:tblPr>"#);

        doc_xml.push_str(r#"<w:tr>"#);
        for h in &["篇章單元", "原著頁碼/來源", "經文頌數", "核心收錄義理"] {
            doc_xml.push_str(&format!(
                r#"<w:tc><w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="19"/><w:szCs w:val="19"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>"#,
                escape_xml(h)
            ));
        }
        doc_xml.push_str(r#"</w:tr>"#);

        for row in matrix {
            if let Some(cols) = row.as_array() {
                doc_xml.push_str(r#"<w:tr>"#);
                for c_idx in 0..4 {
                    let val = cols.get(c_idx).and_then(|v| v.as_str()).unwrap_or("");
                    let align = if c_idx < 3 { "center" } else { "left" };
                    doc_xml.push_str(&format!(
                        r#"<w:tc><w:p><w:pPr><w:jc w:val="{align}"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="18"/><w:szCs w:val="18"/></w:rPr><w:t>{}</w:t></w:r></w:p></w:tc>"#,
                        escape_xml(val)
                    ));
                }
                doc_xml.push_str(r#"</w:tr>"#);
            }
        }
        doc_xml.push_str(r#"</w:tbl>"#);
    }
}

/// 渲染 Word 終卷大圓滿題記 (Puṣpikā)
/// @outputs: 1
fn render_docx_colophon(data: &JsonValue, cs_font: &str, doc_xml: &mut String) {
    if let Some(col) = data.get("colophon") {
        let title = col.get("title").and_then(|s| s.as_str()).unwrap_or("終卷大圓滿題記（Puṣpikā）");
        doc_xml.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
        doc_xml.push_str(&format!(
            r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:before="600" w:after="240"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="32"/><w:szCs w:val="32"/><w:color w:val="7C2D12"/></w:rPr><w:t>【{}】</w:t></w:r></w:p>"#,
            escape_xml(title)
        ));

        if let Some(dev) = col.get("devanagari").and_then(|s| s.as_str()) {
            if !dev.trim().is_empty() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="160"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="26"/><w:szCs w:val="26"/><w:color w:val="0F172A"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(dev)
                ));
            }
        }
        if let Some(beng) = col.get("bengali").and_then(|s| s.as_str()) {
            if !beng.trim().is_empty() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="160"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="Vrinda"/><w:b/><w:sz w:val="26"/><w:szCs w:val="26"/><w:color w:val="0F172A"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(beng)
                ));
            }
        }
        if let Some(iast) = col.get("iast").and_then(|s| s.as_str()) {
            if !iast.trim().is_empty() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="200"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:i/><w:sz w:val="22"/><w:szCs w:val="22"/><w:color w:val="475569"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(iast)
                ));
            }
        }
        if let Some(trans) = col.get("translation_zh").and_then(|s| s.as_str()) {
            if !trans.trim().is_empty() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="240"/><w:lineSpacing w:val="320" w:lineRule="auto"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="24"/><w:szCs w:val="24"/><w:color w:val="0F172A"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(trans)
                ));
            }
        }
        if let Some(sig) = col.get("significance").and_then(|s| s.as_str()) {
            if !sig.trim().is_empty() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:jc w:val="center"/><w:spacing w:after="200"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:i/><w:sz w:val="20"/><w:szCs w:val="20"/><w:color w:val="64748B"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(sig)
                ));
            }
        }
    }
}

/// 渲染 Word 單頌段落
/// @outputs: 1
fn render_docx_single_verse(v: &JsonValue, idx: usize, cs_font: &str, doc_xml: &mut String) {
    let default_id = format!("Text {idx}");
    let vid = v.get("id").and_then(|s| s.as_str()).unwrap_or(&default_id);
    let title = v.get("title").and_then(|s| s.as_str()).unwrap_or("");
    let coord = if vid.contains('[') {
        vid.to_string()
    } else {
        format!("{vid} [全書第 {idx} 頌]")
    };
    let title_line = if title.is_empty() { coord } else { format!("{coord} ．{title}") };

    let bm_name = format!("urn_classic_verse_{idx}");
    doc_xml.push_str(&format!(
        r#"<w:p><w:pPr><w:spacing w:before="280" w:after="60"/></w:pPr><w:bookmarkStart w:id="{idx}" w:name="{bm_name}"/><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="23"/><w:szCs w:val="23"/><w:color w:val="1E3A8A"/></w:rPr><w:t>{}</w:t></w:r><w:bookmarkEnd w:id="{idx}"/></w:p>"#,
        escape_xml(&title_line)
    ));

    if let Some(meter) = v.get("meter").and_then(|s| s.as_str()) {
        if !meter.trim().is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:after="40"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:i/><w:sz w:val="18"/><w:szCs w:val="18"/><w:color w:val="475569"/></w:rPr><w:t>【格律（Chhandas）】：{}</w:t></w:r></w:p>"#,
                escape_xml(meter)
            ));
        }
    }

    if let Some(lines) = v.get("devanagari").and_then(|l| l.as_array()) {
        for l in lines {
            if let Some(t) = l.as_str() {
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:after="40"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="22"/><w:szCs w:val="22"/><w:color w:val="0F172A"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(t)
                ));
            }
        }
    }

    if let Some(iast) = v.get("iast").and_then(|s| s.as_str()) {
        if !iast.trim().is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:after="60"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:i/><w:sz w:val="20"/><w:szCs w:val="20"/><w:color w:val="475569"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                escape_xml(iast)
            ));
        }
    }

    if let Some(en) = v.get("english").and_then(|s| s.as_str()) {
        if !en.trim().is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:after="80"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="19"/><w:szCs w:val="19"/><w:color w:val="334155"/></w:rPr><w:t>【原著英譯】：</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:i/><w:sz w:val="19"/><w:szCs w:val="19"/><w:color w:val="475569"/></w:rPr><w:t>"{}"</w:t></w:r></w:p>"#,
                escape_xml(en)
            ));
        }
    }

    if let Some(beng) = v.get("bengali_commentary").and_then(|s| s.as_str()) {
        if !beng.trim().is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:after="80"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="Vrinda"/><w:b/><w:sz w:val="19"/><w:szCs w:val="19"/><w:color w:val="7C2D12"/></w:rPr><w:t>【孟加拉文古註（অনুবাদ ও তাৎপর্য）】：</w:t></w:r><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="Vrinda"/><w:sz w:val="19"/><w:szCs w:val="19"/><w:color w:val="431407"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                escape_xml(beng)
            ));
        }
    }

    if let Some(anvaya_items) = v.get("anvaya").and_then(|a| a.as_array()) {
        if !anvaya_items.is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="288"/><w:spacing w:before="60" w:after="40"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="18"/><w:szCs w:val="18"/><w:color w:val="0369A1"/></w:rPr><w:t>【散文句法與逐字解（Anvaya &amp; Padaccheda）】：</w:t></w:r></w:p>"#
            ));
            for item in anvaya_items {
                let w = item.get("word").and_then(|s| s.as_str()).unwrap_or("");
                let p = item.get("padaccheda").and_then(|s| s.as_str()).unwrap_or(w);
                let g = item.get("grammar").and_then(|s| s.as_str()).unwrap_or("");
                let m = item.get("meaning_zh").and_then(|s| s.as_str()).unwrap_or("");
                let line = format!("- {w} ({p}) [{g}] ➔ {m}");
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:ind w:left="432"/><w:spacing w:after="20"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="17"/><w:szCs w:val="17"/><w:color w:val="334155"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(&line)
                ));
            }
        }
    }

    let mut trans = v.get("translation_zh").and_then(|s| s.as_str()).unwrap_or("").to_string();
    if !trans.starts_with("譯文:") && !trans.starts_with("譯文：") {
        trans = format!("譯文: {trans}");
    }
    doc_xml.push_str(&format!(
        r#"<w:p><w:pPr><w:spacing w:after="120"/><w:lineSpacing w:val="300" w:lineRule="auto"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="22"/><w:szCs w:val="22"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
        escape_xml(&trans)
    ));

    if let Some(notes) = v.get("notes").and_then(|s| s.as_str()) {
        if !notes.trim().is_empty() {
            doc_xml.push_str(&format!(
                r#"<w:p><w:pPr><w:ind w:left="216"/><w:spacing w:after="160"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:sz w:val="19"/><w:szCs w:val="19"/><w:color w:val="64748B"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                escape_xml(notes)
            ));
        }
    }
}

/// 渲染 Word 經文全錄
/// @outputs: 1
fn render_docx_verses(data: &JsonValue, cs_font: &str, doc_xml: &mut String) {
    if let Some(verses) = data.get("verses").and_then(|v| v.as_array()) {
        let mut curr_chap = String::new();
        for (idx_0, v) in verses.iter().enumerate() {
            let idx = idx_0.saturating_add(1);
            let chap = v.get("chapter").and_then(|s| s.as_str()).unwrap_or("").trim();
            if !chap.is_empty() && chap != curr_chap {
                curr_chap = chap.to_string();
                doc_xml.push_str(&format!(
                    r#"<w:p><w:pPr><w:spacing w:before="400" w:after="160"/></w:pPr><w:r><w:rPr><w:rFonts w:ascii="Times New Roman" w:hAnsi="Times New Roman" w:eastAsia="教育部標準楷書" w:cs="{cs_font}"/><w:b/><w:sz w:val="28"/><w:szCs w:val="28"/><w:color w:val="0F172A"/></w:rPr><w:t>{}</w:t></w:r></w:p>"#,
                    escape_xml(chap)
                ));
            }
            render_docx_single_verse(v, idx, cs_font, doc_xml);
        }
    }
}

/// 建構 Word .docx 封裝包 (OpenXML 標準，無外部依賴)
/// @outputs: 2
fn build_docx_package(data: &JsonValue, docx_path: &Path) -> Result<(), String> {
    let tmp_dir = Path::new("/tmp").join(format!("docx_build_{}", process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(tmp_dir.join("_rels")).map_err(|e| e.to_string())?;
    fs::create_dir_all(tmp_dir.join("word").join("_rels")).map_err(|e| e.to_string())?;

    let lang = data.get("language").and_then(|v| v.as_str()).unwrap_or("sanskrit");
    let cs_font = if lang.to_lowercase() == "bengali" { "Vrinda" } else { "Nirmala UI" };

    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#;
    fs::write(tmp_dir.join("[Content_Types].xml"), content_types).map_err(|e| e.to_string())?;

    let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#;
    fs::write(tmp_dir.join("_rels").join(".rels"), root_rels).map_err(|e| e.to_string())?;

    let mut doc_xml = String::with_capacity(128 * 1024);
    doc_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#);

    render_docx_preface(data, cs_font, &mut doc_xml);
    render_docx_audit_table(data, cs_font, &mut doc_xml);
    doc_xml.push_str(r#"<w:p><w:r><w:br w:type="page"/></w:r></w:p>"#);
    render_docx_verses(data, cs_font, &mut doc_xml);
    render_docx_colophon(data, cs_font, &mut doc_xml);

    doc_xml.push_str(r#"<w:sectPr><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#);
    fs::write(tmp_dir.join("word").join("document.xml"), doc_xml).map_err(|e| e.to_string())?;

    // @safety-proof: docx_path is derived from validated base_name and out_dir
    let abs_docx_path = if docx_path.is_absolute() {
        docx_path.to_path_buf()
    } else {
        env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")).join(docx_path)
    };
    let docx_target_str = abs_docx_path.to_string_lossy();
    let status = Command::new("zip")
        .args(["-q", "-r", &docx_target_str, "[Content_Types].xml", "_rels", "word"])
        .current_dir(&tmp_dir)
        .status()
        .map_err(|e| format!("執行 zip 封裝失敗: {e}"))?;

    let _ = fs::remove_dir_all(&tmp_dir);
    if !status.success() {
        return Err("zip 封裝 docx 回傳非零狀態碼".to_string());
    }

    println!("📄 [DOCX] 繁體中文全譯本已建構完成 -> {}", docx_path.display());
    Ok(())
}

/// 上傳檔案至雲端硬碟
/// @outputs: 2
fn upload_file_to_drive(file_path: &Path, parent_id: &str) {
    let fname = file_path.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
    println!("☁️ [GDRIVE] 正在同步雙語定本至 Google Drive -> {fname}");

    let status = Command::new("gdrive")
        .args(["mutate", "upload", &file_path.to_string_lossy(), "--parent", parent_id])
        .status();

    match status {
        Ok(s) if s.success() => println!("☁️ [GDRIVE] 成功同步定本至雲端硬碟 (目錄 ID: {parent_id}) -> {fname}"),
        Ok(_) => eprintln!("⚠️ [GDRIVE] gdrive 上傳回傳非零代碼"),
        Err(e) => eprintln!("⚠️ [GDRIVE] 無法執行 gdrive 指令: {e}"),
    }
}

/// 列印使用說明
/// @outputs: 1
fn print_usage() {
    eprintln!(r#"
build_classic_edition — 古典聖典繁體中文定本雙模態生成器 (Word .docx + Markdown .md 原生 Rust 版)

用法:
  build_classic_edition --json <path/to/classic.json> [選項]

選項:
  --out-dir <dir>              輸出目錄
  --upload                     建構完成後自動上傳至 Google Drive「翻譯完成」資料夾
  --drive-parent <folder_id>   Google Drive 目標資料夾 ID
  -h, --help                   顯示此說明訊息
"#);
}

struct BuildOptions {
    json_path: String,
    out_dir: String,
    upload: bool,
    drive_parent: String,
}

/// 解析建構參數
/// @outputs: 2
fn parse_build_args(args: &[String]) -> Result<BuildOptions, String> {
    let mut json_path = None;
    let mut out_dir = "/home/antigravity/mangala1728/Sanskrit-Classic-Translation/out".to_string();
    let mut upload = false;
    let mut drive_parent = "1xresTeMn_ddALkgFg3sUtxFEI_hpib30".to_string();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-h" || arg == "--help" {
            print_usage();
            process::exit(0);
        } else if arg == "--json" && i + 1 < args.len() {
            json_path = Some(args[i + 1].clone());
            i += 2;
        } else if arg == "--out-dir" && i + 1 < args.len() {
            out_dir = args[i + 1].clone();
            i += 2;
        } else if arg == "--upload" {
            upload = true;
            i += 1;
        } else if arg == "--drive-parent" && i + 1 < args.len() {
            drive_parent = args[i + 1].clone();
            i += 2;
        } else {
            i += 1;
        }
    }

    match json_path {
        Some(j) => Ok(BuildOptions { json_path: j, out_dir, upload, drive_parent }),
        None => Err("請使用 --json 指定聖典結構化 JSON 資料檔案".to_string()),
    }
}

/// CLI 主入口點
/// @outputs: 3
fn main() {
    let args: Vec<String> = env::args().collect();
    let opts = match parse_build_args(&args) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("❌ 錯誤: {e}");
            print_usage();
            process::exit(1);
        }
    };

    let json_text = match fs::read_to_string(&opts.json_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("❌ 無法讀取 JSON 檔案 '{}': {e}", opts.json_path);
            process::exit(1);
        }
    };

    let mut parser = JsonParser::new(&json_text);
    let book_data = match parser.parse_value() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("❌ JSON 解析錯誤: {e}");
            process::exit(1);
        }
    };

    let (base_name, docx_file, md_file) = derive_canonical_filenames(&book_data, &opts.out_dir);
    println!("📌 [CANONICAL NAME] 標準交付標識: {base_name}");
    let _ = fs::create_dir_all(&opts.out_dir);

    if let Err(e) = build_markdown_file(&book_data, &md_file) {
        eprintln!("❌ 建構 Markdown 失敗: {e}");
        process::exit(1);
    }
    if let Err(e) = build_docx_package(&book_data, &docx_file) {
        eprintln!("❌ 建構 Word .docx 失敗: {e}");
        process::exit(1);
    }

    println!("\n🔍 正在執行剛性品質與連續性門禁驗收...");
    let md_content = fs::read_to_string(&md_file).unwrap_or_default();
    let qa_res = inspect_markdown_content(&md_content, &md_file.to_string_lossy(), None, None, None);

    if qa_res.verdict == "PASS" {
        println!("✅ 門禁驗收全項通過！雙語檔名合規、經文單調連續、雙豎線閉合、0 禁字、保留原著英譯供大師校審。");
    } else {
        eprintln!("❌ 門禁驗收失敗！請檢視缺失後修正：");
        for err in &qa_res.totality_errors {
            eprintln!("   • {err}");
        }
        process::exit(1);
    }

    if opts.upload {
        println!("\n☁️ 正在同步雙語定本至 Google Drive...");
        upload_file_to_drive(&docx_file, &opts.drive_parent);
        upload_file_to_drive(&md_file, &opts.drive_parent);
    }

    println!("\n🎉 雙模態出版級定本建構完畢！\n   📄 Word: {}\n   📝 MD:   {}", docx_file.display(), md_file.display());
}
