#!/usr/bin/env python3
# SPDX-License-Identifier: LicenseRef-Project-License
"""## Map
Role: Cross-Repository Domain Pipe Standards Synchronizer
Inputs:
  - Upstream Master: c:/For Gemini/digital-historian/portable/domain-pipe-standards
  - Local Mirror: portable/domain-pipe-standards
Outputs:
  - SHA-256 integrity report & file mirror synchronization
Invariants:
  1. Pure Python standard library only (hashlib, shutil, sys, pathlib).
  2. Fail-closed: Exit code 1 if upstream is missing or hashes mismatch in --check mode.
  3. Automatically triggers test_astro_pipeline.py upon sync.
"""

import argparse
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import sys

# Configure UTF-8 stdout
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

REPO_ROOT = Path(__file__).resolve().parent.parent
LOCAL_DIR = REPO_ROOT / "portable" / "domain-pipe-standards"
UPSTREAM_DIR = Path("C:/For Gemini/digital-historian/portable/domain-pipe-standards")


def calc_sha256(path: Path) -> str:
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while chunk := f.read(65536):
            h.update(chunk)
    return h.hexdigest()


def main():
    parser = argparse.ArgumentParser(description="Domain Pipe Standards Synchronizer")
    parser.add_argument("--sync", action="store_true", help="Sync files from upstream to local mirror")
    parser.add_argument("--check", action="store_true", help="Check sync status without copying")
    args = parser.parse_args()

    if not UPSTREAM_DIR.exists():
        print(f"[ERROR] Upstream directory not found: {UPSTREAM_DIR}")
        sys.exit(1)

    upstream_files = sorted([p for p in UPSTREAM_DIR.glob("*") if p.is_file()])
    print(f"=== Domain Pipe Standards Synchronization Check ===")
    print(f"Upstream : {UPSTREAM_DIR}")
    print(f"Local    : {LOCAL_DIR}\n")

    mismatch = False
    for up_file in upstream_files:
        rel_name = up_file.name
        loc_file = LOCAL_DIR / rel_name
        up_hash = calc_sha256(up_file)

        if not loc_file.exists():
            print(f"[MISSING] {rel_name} not present in local mirror")
            mismatch = True
            if args.sync:
                LOCAL_DIR.mkdir(parents=True, exist_ok=True)
                shutil.copy2(up_file, loc_file)
                print(f"  -> Synced {rel_name}")
        else:
            loc_hash = calc_sha256(loc_file)
            if up_hash != loc_hash:
                print(f"[DIFF] {rel_name} SHA-256 differs!")
                print(f"  Upstream: {up_hash[:16]}... | Local: {loc_hash[:16]}...")
                mismatch = True
                if args.sync:
                    shutil.copy2(up_file, loc_file)
                    print(f"  -> Synced {rel_name}")
            else:
                print(f"[OK] {rel_name:35} (SHA256: {up_hash[:12]}...)")

    if args.sync or not mismatch:
        test_script = LOCAL_DIR / "test_astro_pipeline.py"
        if test_script.exists():
            print("\n--- Running Local Verification Suite ---")
            res = subprocess.run([sys.executable, str(test_script)], check=False)
            if res.returncode != 0:
                print("[ERROR] Verification suite failed!")
                sys.exit(res.returncode)

    if mismatch and args.check:
        print("\n[FAIL] Local mirror is out of sync with upstream master. Run with --sync.")
        sys.exit(1)

    print("\n[SUCCESS] Domain pipe standards are 100% in sync.")
    sys.exit(0)


if __name__ == "__main__":
    main()
