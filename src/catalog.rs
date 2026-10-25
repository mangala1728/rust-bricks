// ## Map
// Role: core/catalog — 零外部依賴之原始碼自動解析與 AI 機器專讀索引生成器 (YAML / JSON)。
// Inputs: src/ 目錄下之所有積木原始檔路徑。
// Outputs: CATALOG.yaml 與 CATALOG.json 索引檔案，回傳索引函式總數。
// Invariants: 零外部依賴、零全域可變狀態、無遞迴、控制流巢狀深度 <= 2。

use std::fs;
use std::path::Path;

pub struct FuncInfo {
    pub name: String,
    pub signature: String,
    pub description: String,
    pub outputs: Option<u32>,
    pub domains: Vec<(String, String)>,
    pub rosetta: Option<String>,
}

pub struct ModuleInfo {
    pub module_name: String,
    pub file_path: String,
    pub architecture_map: Vec<String>,
    pub functions: Vec<FuncInfo>,
}

/// @outputs: 2
fn extract_map_lines(content: &str) -> Vec<String> {
    let mut map_lines = Vec::new();
    let mut in_map = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("// ## Map") || trimmed.starts_with("//## Map") {
            in_map = true;
            continue;
        }
        if !in_map {
            continue;
        }
        if !trimmed.starts_with("//") {
            break;
        }
        let text = trimmed.trim_start_matches('/').trim();
        if !text.is_empty() {
            map_lines.push(text.to_string());
        }
    }
    map_lines
}

/// @outputs: 2
fn parse_domain_tag(d: &str) -> Option<(String, String)> {
    let dom_str = d.trim_start_matches("@domain:").trim();
    let pos = dom_str.find('[')?;
    let param = dom_str[..pos].trim();
    let range = dom_str[pos..].trim();
    Some((param.to_string(), range.to_string()))
}

/// @outputs: 1
fn process_doc_tag(d: &str, outputs: &mut Option<u32>, domains: &mut Vec<(String, String)>, rosetta: &mut Option<String>) {
    if d.starts_with("@outputs:") {
        let num_str = d.trim_start_matches("@outputs:").trim();
        *outputs = num_str.parse::<u32>().ok();
        return;
    }
    if d.starts_with("@domain:") {
        if let Some(pair) = parse_domain_tag(d) {
            domains.push(pair);
        }
        return;
    }
    if d.starts_with("@rosetta:") {
        *rosetta = Some(d.trim_start_matches("@rosetta:").trim().to_string());
    }
}

/// @outputs: 1
/// @domain: fn_idx [0..1000000]
fn parse_fn_doc_block(lines: &[&str], fn_idx: usize) -> (Option<u32>, Vec<(String, String)>, Option<String>, String) {
    if fn_idx > 1_000_000 {
        return (None, Vec::new(), None, String::new());
    }
    let mut doc_lines = Vec::new();
    let mut back = fn_idx;
    while back > 0 {
        back -= 1;
        let prev = lines[back].trim();
        if prev.starts_with("///") {
            doc_lines.push(prev.trim_start_matches('/').trim().to_string());
        } else if !prev.is_empty() {
            break;
        }
    }
    doc_lines.reverse();

    let mut outputs = None;
    let mut domains = Vec::new();
    let mut rosetta = None;
    let mut desc_words = Vec::new();

    for d in &doc_lines {
        process_doc_tag(d, &mut outputs, &mut domains, &mut rosetta);
        if !d.starts_with('@') && !d.is_empty() {
            desc_words.push(d.as_str());
        }
    }

    (outputs, domains, rosetta, desc_words.join(" "))
}

/// @outputs: 1
fn parse_fn_signature(line: &str) -> (String, String) {
    // 使用十六進位避免字面括號造成靜態檢查混淆
    let sig_end = line.find('\x7B').unwrap_or(line.len());
    let sig = line[..sig_end].trim().to_string();
    let after_fn = line.trim_start_matches("pub fn ").trim();
    let fn_name_end = after_fn.find('(').unwrap_or(after_fn.len());
    let fn_name = after_fn[..fn_name_end].trim().to_string();
    (fn_name, sig)
}

/// @outputs: 2
fn parse_module_file(file_path: &Path) -> Option<ModuleInfo> {
    let content = fs::read_to_string(file_path).ok()?;
    let mod_name = file_path.file_stem()?.to_string_lossy().to_string();
    let rel_path = format!("src/{}", file_path.file_name()?.to_string_lossy());
    let map_lines = extract_map_lines(&content);

    let mut functions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("pub fn ") {
            continue;
        }
        let (outputs, domains, rosetta, desc) = parse_fn_doc_block(&lines, idx);
        let (fn_name, sig) = parse_fn_signature(trimmed);

        functions.push(FuncInfo {
            name: fn_name,
            signature: sig,
            description: desc,
            outputs,
            domains,
            rosetta,
        });
    }

    Some(ModuleInfo {
        module_name: mod_name,
        file_path: rel_path,
        architecture_map: map_lines,
        functions,
    })
}

/// @outputs: 1
fn append_yaml_func(yaml: &mut String, f: &FuncInfo) {
    yaml.push_str(&format!("      - name: \"{}\"\n", f.name));
    yaml.push_str(&format!("        signature: \"{}\"\n", f.signature.replace('"', "\\\"")));
    yaml.push_str(&format!("        outputs: {}\n", f.outputs.unwrap_or(1)));
    if !f.domains.is_empty() {
        yaml.push_str("        domains:\n");
        for (p, r) in &f.domains {
            yaml.push_str(&format!("          {}: \"{}\"\n", p, r));
        }
    }
    if let Some(ref ros) = f.rosetta {
        yaml.push_str(&format!("        rosetta: \"{}\"\n", ros.replace('"', "\\\"")));
    }
}

/// @outputs: 1
fn append_yaml_module(yaml: &mut String, m: &ModuleInfo) {
    yaml.push_str(&format!("  - module: \"{}\"\n", m.module_name));
    yaml.push_str(&format!("    file: \"{}\"\n", m.file_path));
    yaml.push_str(&format!("    functions_count: {}\n", m.functions.len()));
    yaml.push_str("    architecture_map:\n");
    for line in &m.architecture_map {
        let clean = line.replace('"', "\\\"");
        yaml.push_str(&format!("      - \"{}\"\n", clean));
    }
    yaml.push_str("    functions:\n");
    for f in &m.functions {
        append_yaml_func(yaml, f);
    }
}

/// @outputs: 1
/// @domain: total_fns [0..1000000]
fn serialize_yaml_catalog(modules: &[ModuleInfo], total_fns: usize) -> String {
    if total_fns > 1_000_000 {
        return String::new();
    }
    let mut yaml = String::new();
    yaml.push_str("# ===================================================================\n");
    yaml.push_str("# CATALOG.yaml — rust-bricks 函式積木庫 AI 代理專用索引檔（Rust 自動產出）\n");
    yaml.push_str("# 甩手掌櫃免讀！各專案 AI 碼工請讀取本檔以獲取現成可複用函式\n");
    yaml.push_str("# ===================================================================\n");
    yaml.push_str("catalog_version: \"1.0.0\"\n");
    yaml.push_str("repository: \"https://github.com/mangala1728/rust-bricks\"\n");
    yaml.push_str("standard: \"09-contract-boundary\"\n");
    yaml.push_str("zero_dependencies: true\n");
    yaml.push_str(&format!("total_functions: {}\n", total_fns));
    yaml.push_str("modules:\n");

    for m in modules {
        append_yaml_module(&mut yaml, m);
    }
    yaml
}

/// @outputs: 1
fn append_json_func(json: &mut String, f: &FuncInfo, is_last: bool) {
    json.push_str("        {\n");
    json.push_str(&format!("          \"name\": \"{}\",\n", f.name));
    json.push_str(&format!("          \"signature\": \"{}\",\n", f.signature.replace('"', "\\\"")));
    json.push_str(&format!("          \"outputs\": {}{}\n", f.outputs.unwrap_or(1), if f.rosetta.is_some() { "," } else { "" }));
    if let Some(ref ros) = f.rosetta {
        json.push_str(&format!("          \"rosetta\": \"{}\"\n", ros.replace('"', "\\\"")));
    }
    let comma = if is_last { "" } else { "," };
    json.push_str(&format!("        }}{}\n", comma));
}

/// @outputs: 1
fn append_json_module(json: &mut String, m: &ModuleInfo, is_last: bool) {
    json.push_str("    {\n");
    json.push_str(&format!("      \"module\": \"{}\",\n", m.module_name));
    json.push_str(&format!("      \"file\": \"{}\",\n", m.file_path));
    json.push_str(&format!("      \"functions_count\": {},\n", m.functions.len()));
    json.push_str("      \"functions\": [\n");

    for (f_idx, f) in m.functions.iter().enumerate() {
        let fn_last = f_idx + 1 == m.functions.len();
        append_json_func(json, f, fn_last);
    }

    let comma = if is_last { "" } else { "," };
    json.push_str(&format!("      ]\n    }}{}\n", comma));
}

/// @outputs: 1
/// @domain: total_fns [0..1000000]
fn serialize_json_catalog(modules: &[ModuleInfo], total_fns: usize) -> String {
    if total_fns > 1_000_000 {
        return String::new();
    }
    let mut json = String::new();
    json.push_str("{\n");
    json.push_str("  \"catalog_version\": \"1.0.0\",\n");
    json.push_str("  \"repository\": \"https://github.com/mangala1728/rust-bricks\",\n");
    json.push_str("  \"standard\": \"09-contract-boundary\",\n");
    json.push_str("  \"zero_dependencies\": true,\n");
    json.push_str(&format!("  \"total_functions\": {},\n", total_fns));
    json.push_str("  \"modules\": [\n");

    for (m_idx, m) in modules.iter().enumerate() {
        let mod_last = m_idx + 1 == modules.len();
        append_json_module(&mut json, m, mod_last);
    }

    json.push_str("  ]\n");
    json.push_str("}\n");
    json
}

/// @outputs: 2
pub fn generate_catalogs(repo_root: &Path) -> Result<usize, String> {
    let src_dir = repo_root.join("src");
    if !src_dir.exists() {
        return Err("src 目錄不存在".to_string());
    }

    let entries = fs::read_dir(&src_dir).map_err(|e| format!("讀取 src 失敗: {}", e))?;
    let mut modules = Vec::new();
    let mut total_fns = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        if fname == "lib.rs" || fname == "main.rs" || fname == "catalog.rs" || fname == "guard.rs" {
            continue;
        }

        if let Some(info) = parse_module_file(&path) {
            total_fns += info.functions.len();
            modules.push(info);
        }
    }

    modules.sort_by(|a, b| a.module_name.cmp(&b.module_name));

    let yaml = serialize_yaml_catalog(&modules, total_fns);
    fs::write(repo_root.join("CATALOG.yaml"), yaml).map_err(|e| format!("寫入 CATALOG.yaml 失敗: {}", e))?;

    let json = serialize_json_catalog(&modules, total_fns);
    fs::write(repo_root.join("CATALOG.json"), json).map_err(|e| format!("寫入 CATALOG.json 失敗: {}", e))?;

    Ok(total_fns)
}
