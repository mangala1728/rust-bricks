#!/usr/bin/env bash
# ==============================================================================
# 雲端冷啟動全鏈條驗收測試腳本 (Cold-Start All-Pass Verification Suite)
# ==============================================================================

set -euo pipefail

PASS=0
FAIL=0

check_step() {
    local name="$1"
    shift
    echo -n "正在執行驗證: $name ... "
    if "$@" > /tmp/cold_start_step.log 2>&1; then
        echo "✅ PASS"
        PASS=$((PASS + 1))
    else
        echo "❌ FAIL"
        echo "----------------- 錯誤詳情 -----------------"
        cat /tmp/cold_start_step.log
        echo "-------------------------------------------"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== 雲端冷啟動五階剛性驗收開始 ==="

# Stage 1: Rust 工具鏈與純度檢查 (Zero External Dependencies)
check_step "[Stage 1] Cargo.toml 零外部依賴檢查" bash -c '
    deps=$(grep -A 100 "\[dependencies\]" Cargo.toml | grep -v "\[" | grep -v "^#" | grep -v "^$" || true)
    if [ -n "$deps" ]; then
        echo "違反鐵律：Cargo.toml 發現外部依賴：$deps"
        exit 1
    fi
'

# Stage 2: 09 號合約與 NASA JPL 60 行規範守衛 (brick-guard)
check_step "[Stage 2] brick-guard 合約與容量守衛" cargo run -q -- guard

# Stage 3: 全單元測試與黑箱巡檢
check_step "[Stage 3] 全庫單元測試驗收 (120+ tests)" cargo test -q

# Stage 4: 雙端下水道標準便攜包物理尺寸與純整數算術驗收
check_step "[Stage 4] 便攜下水道標準套件驗收" python3 portable/domain-pipe-standards/test_astro_pipeline.py

# Stage 5: 雲端農夫佇列與死路樹推導完整性
check_step "[Stage 5] 雲端農夫佇列無卡死驗收" cargo run -q -- farm-status

echo "=========================================="
echo "驗收結果: 通過 $PASS 項, 失敗 $FAIL 項"
if [ "$FAIL" -eq 0 ]; then
    echo "🎉 雲端節點已完全具備生產環境冷啟動標準！"
    exit 0
else
    echo "⚠️ 存在失敗項目，請修正後重新驗證。"
    exit 1
fi
