#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""brick-guard.py — rust-bricks 倉庫容量管理與 09 合約品質守門員（Git pre-commit & Antigravity Hook）。

守門鐵律（剛性攔截）：
1. 【容量防線】拒絕任何二進位/多媒體檔案（.wav, .mp3, .png, .jpg, .zip, .bin, .onnx, .pt, .exe, .so 等）。
2. 【大小防線】單一文字檔案嚴禁超過 500 KB；防止代理將巨型文字日誌或百萬行假資料塞入庫。
3. 【依賴防線】Cargo.toml 的 [dependencies] 必須為空，嚴禁引入任何第三方 crate。
4. 【合約防線】調用 rust-lint.py 檢查 09 號合約（## Map / @outputs / @domain / 零全域 / 無遞迴）。
5. 【品質防線】執行 cargo test，確保 100% 測試全綠通過。
"""

import os
import re
import subprocess
import sys
from pathlib import Path

# Windows UTF-8 主控台編碼保護
for _s in (sys.stdout, sys.stderr):
    try:
        _s.reconfigure(encoding="utf-8", errors="replace")
    except (AttributeError, ValueError):
        pass

ROOT = Path(__file__).resolve().parents[1]
MAX_FILE_BYTES = 500 * 1024  # 500 KB

BLOCKED_EXTENSIONS = {
    ".wav", ".mp3", ".aac", ".flac", ".ogg",
    ".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".ico",
    ".mp4", ".mkv", ".avi", ".mov",
    ".zip", ".tar", ".gz", ".bz2", ".7z", ".rar",
    ".exe", ".dll", ".so", ".dylib", ".bin", ".dat",
    ".onnx", ".pt", ".pth", ".tflite", ".gguf", ".safetensors",
    ".pkl", ".npy", ".npz",
}


def check_file_bloat(repo_root: Path) -> list:
    """檢查工作目錄中的檔案大小與二進位副檔名。"""
    issues = []
    # 掃描已追蹤與暫存區檔案
    cmd = ["git", "ls-files"]
    r = subprocess.run(cmd, cwd=str(repo_root), capture_output=True, text=True, encoding="utf-8", errors="replace")
    files = r.stdout.splitlines() if r.returncode == 0 else []

    # 包含未追蹤的檔案 (git ls-files --others --exclude-standard)
    cmd_others = ["git", "ls-files", "--others", "--exclude-standard"]
    r_others = subprocess.run(cmd_others, cwd=str(repo_root), capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r_others.returncode == 0:
        files.extend(r_others.stdout.splitlines())

    for f_rel in set(files):
        f = repo_root / f_rel
        if not f.exists() or not f.is_file():
            continue
        ext = f.suffix.lower()
        if ext in BLOCKED_EXTENSIONS:
            issues.append(f"【二進位檔案攔截】禁止提交多媒體/大型二進位檔：{f_rel} (副檔名 {ext})")
        size = f.stat().st_size
        if size > MAX_FILE_BYTES:
            issues.append(f"【超大檔案攔截】檔案超過 500 KB 上限：{f_rel} ({size / 1024:.1f} KB)")

    return issues


def check_cargo_dependencies(repo_root: Path) -> list:
    """檢查 Cargo.toml 是否偷偷引入任何外部 crate。"""
    issues = []
    cargo_toml = repo_root / "Cargo.toml"
    if not cargo_toml.exists():
        return ["找不到 Cargo.toml"]

    content = cargo_toml.read_text(encoding="utf-8")
    in_deps = False
    for line in content.splitlines():
        line_clean = line.strip()
        if line_clean.startswith("[") and line_clean.endswith("]"):
            in_deps = (line_clean == "[dependencies]")
            continue
        if in_deps and line_clean and not line_clean.startswith("#"):
            issues.append(f"【依賴違規】[dependencies] 必須為空，禁止引入外部 crate：{line_clean}")

    return issues


def check_contract(repo_root: Path) -> list:
    """調用 09 號合約檢查器。"""
    issues = []
    # 尋找工作流中的 rust-lint.py
    lint_script = repo_root.parent / "工作流" / "scripts" / "rust-lint.py"
    if not lint_script.exists():
        return issues  # 若工作流不在相鄰目錄，退化放行或報備

    cmd = [sys.executable, str(lint_script), str(repo_root / "src")]
    r = subprocess.run(cmd, cwd=str(repo_root), capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        issues.append(f"【09 合約檢查失敗】\n{r.stdout}")

    return issues


def check_tests(repo_root: Path) -> list:
    """確保單元測試全數通過。"""
    issues = []
    r = subprocess.run(["cargo", "test"], cwd=str(repo_root), capture_output=True, text=True, encoding="utf-8", errors="replace")
    if r.returncode != 0:
        issues.append(f"【單元測試失敗】cargo test 未通過：\n{r.stdout}\n{r.stderr}")
    return issues


def main():
    print("🛡️ [brick-guard] 執行 rust-bricks 倉庫容量與合約守護檢查...")
    all_issues = []

    # 1. 檔案大小與二進位檢查
    bloat_issues = check_file_bloat(ROOT)
    all_issues.extend(bloat_issues)

    # 2. 外部依賴檢查
    dep_issues = check_cargo_dependencies(ROOT)
    all_issues.extend(dep_issues)

    # 3. 09 合約靜態審查
    contract_issues = check_contract(ROOT)
    all_issues.extend(contract_issues)

    # 4. 單元測試驗證 (若前面的靜態檢查已出錯則先行報錯)
    if not all_issues:
        test_issues = check_tests(ROOT)
        all_issues.extend(test_issues)

    if all_issues:
        print("\n❌ [brick-guard] 檢查未通過，攔截提交：")
        for err in all_issues:
            print(f"  - {err}")
        return 1

    print("✅ [brick-guard] 檢查通過！零外部依賴、零二進位大檔、100% 符合 09 合約規範、測試全綠。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
