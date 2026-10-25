#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""generate_catalog.py — 自動剖析 rust-bricks 原始碼，產出專供 AI 代理高效點讀的機器索引清單（CATALOG.yaml & CATALOG.json）。

AI 代理在尋找是否有現成積木時，只需讀取 CATALOG.yaml 或 CATALOG.json，
即可在 O(1) 時間內獲取所有函式名稱、型別簽章、@domain 邊界、@outputs 基數與跨語言 Rosetta 對照，
無需費時逐一掃描各個 .rs 原始檔，達成甩手掌櫃高效率協同。
"""

import json
import re
import sys
from pathlib import Path

# Windows UTF-8 編碼保護
for _s in (sys.stdout, sys.stderr):
    try:
        _s.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "src"

MAP_RE = re.compile(r"//\s*##\s*Map\s*\n((?://[^\n]*\n)+)", re.M)
FN_RE = re.compile(
    r"((?:[ \t]*///[^\n]*\n)*)[ \t]*pub\s+fn\s+([A-Za-z_]\w*)\s*(?:<[^>]*>)?\s*\(([^)]*)\)\s*(?:->\s*([^{]+))?\s*\{",
    re.M
)
OUTPUTS_RE = re.compile(r"@outputs:\s*(\d+)")
DOMAIN_RE = re.compile(r"@domain:\s*([A-Za-z_]\w*)\s*\[([^\]]*)\]")
ROSETTA_RE = re.compile(r"@rosetta:\s*([^\n]+)")


def parse_rs_file(filepath: Path) -> dict:
    text = filepath.read_text(encoding="utf-8")
    mod_name = filepath.stem

    # 1. 抽取 ## Map 區塊
    map_match = MAP_RE.search(text)
    map_lines = []
    if map_match:
        for line in map_match.group(1).splitlines():
            line_str = line.strip().lstrip("/").strip()
            if line_str:
                map_lines.append(line_str)

    functions = []
    for m in FN_RE.finditer(text):
        doc_block = m.group(1)
        fn_name = m.group(2)
        params_raw = m.group(3).strip()
        ret_type = m.group(4).strip() if m.group(4) else "()"

        # 抽取合約標注
        outputs = None
        out_m = OUTPUTS_RE.search(doc_block)
        if out_m:
            outputs = int(out_m.group(1))

        domains = {}
        for d_m in DOMAIN_RE.finditer(doc_block):
            domains[d_m.group(1)] = d_m.group(2)

        rosetta = None
        ros_m = ROSETTA_RE.search(doc_block)
        if ros_m:
            rosetta = ros_m.group(1).strip()

        # 整理純文字說明
        clean_docs = []
        for line in doc_block.splitlines():
            clean_l = line.strip().lstrip("/").strip()
            if clean_l and not clean_l.startswith("@"):
                clean_docs.append(clean_l)

        functions.append({
            "name": fn_name,
            "signature": f"pub fn {fn_name}({params_raw}) -> {ret_type}",
            "description": " ".join(clean_docs),
            "outputs_cardinality": outputs,
            "domain_bounds": domains,
            "rosetta_equivalent": rosetta,
        })

    return {
        "module": mod_name,
        "file": f"src/{filepath.name}",
        "architecture_map": map_lines,
        "functions_count": len(functions),
        "functions": functions,
    }


def main():
    modules = []
    total_fns = 0

    for rs_file in sorted(SRC.glob("*.rs")):
        if rs_file.name in ("lib.rs", "main.rs"):
            continue
        parsed = parse_rs_file(rs_file)
        total_fns += parsed["functions_count"]
        modules.append(parsed)

    catalog = {
        "catalog_version": "1.0.0",
        "repository": "https://github.com/mangala1728/rust-bricks",
        "standard": "09-contract-boundary",
        "zero_dependencies": True,
        "total_functions": total_fns,
        "modules": modules,
    }

    # 1. 產出 JSON 格式
    json_path = ROOT / "CATALOG.json"
    json_path.write_text(json.dumps(catalog, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    # 2. 產出 YAML 格式 (手動產生乾淨的純文字 YAML，零第三方 yaml 庫依賴)
    yaml_lines = [
        "# ===================================================================",
        "# CATALOG.yaml — rust-bricks 函式積木庫 AI 代理專用索引檔（機器專讀）",
        "# 甩手掌櫃免讀！各專案 AI 碼工請讀取本檔以快速獲取現成可複用函式",
        "# ===================================================================",
        f'catalog_version: "{catalog["catalog_version"]}"',
        f'repository: "{catalog["repository"]}"',
        f'standard: "{catalog["standard"]}"',
        f'zero_dependencies: true',
        f'total_functions: {catalog["total_functions"]}',
        "modules:",
    ]

    for mod in modules:
        yaml_lines.append(f'  - module: "{mod["module"]}"')
        yaml_lines.append(f'    file: "{mod["file"]}"')
        yaml_lines.append(f'    functions_count: {mod["functions_count"]}')
        yaml_lines.append("    architecture_map:")
        for line in mod["architecture_map"]:
            clean = line.replace('"', '\\"')
            yaml_lines.append(f'      - "{clean}"')
        yaml_lines.append("    functions:")
        for fn in mod["functions"]:
            yaml_lines.append(f'      - name: "{fn["name"]}"')
            clean_sig = fn["signature"].replace('"', '\\"')
            yaml_lines.append(f'        signature: "{clean_sig}"')
            clean_desc = fn["description"].replace('"', '\\"')
            yaml_lines.append(f'        description: "{clean_desc}"')
            yaml_lines.append(f'        outputs: {fn["outputs_cardinality"]}')
            if fn["domain_bounds"]:
                yaml_lines.append("        domains:")
                for k, v in fn["domain_bounds"].items():
                    yaml_lines.append(f'          {k}: "{v}"')
            if fn["rosetta_equivalent"]:
                yaml_lines.append(f'        rosetta: "{fn["rosetta_equivalent"]}"')

    yaml_path = ROOT / "CATALOG.yaml"
    yaml_path.write_text("\n".join(yaml_lines) + "\n", encoding="utf-8")

    print(f"✨ [generate_catalog] 成功產生 AI 專用索引：")
    print(f"  - YAML: {yaml_path} ({len(modules)} 模組, {total_fns} 函式)")
    print(f"  - JSON: {json_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
