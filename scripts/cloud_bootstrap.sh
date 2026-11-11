#!/usr/bin/env bash
# ==============================================================================
# 雲端一鍵冷啟動引導腳本 (Cloud Cold-Start Bootstrap Script)
# 支援環境：Ubuntu 20.04/22.04/24.04, Debian 11/12, Amazon Linux 2023, Docker 容器
# 職責：
#   1. 自動安裝編譯底層工具鏈 (GCC, Make, Python3, jq, Git, Rust stable)
#   2. 建立標準工作目錄結構並檢驗雙核心倉庫 (rust-bricks + digital-historian)
#   3. 設定金鑰環境變數模板 (.env)
#   4. 執行雙端品質守衛與合約驗收，確保 100% 綠燈冷啟動
# ==============================================================================

set -euo pipefail

# 顏色輸出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}  🚀 正在啟動純 Rust 微積木庫與數位史官雲端冷啟動 (Cold-Start)   ${NC}"
echo -e "${CYAN}================================================================${NC}"

# 1. 檢測作業系統與套件管理工具
echo -e "\n${BLUE}[Step 1/6] 檢測作業系統環境與必要套件...${NC}"
if [ -f /etc/os-release ]; then
    . /etc/os-release
    OS=$ID
    echo -e "檢測到作業系統: ${GREEN}${NAME:-$OS} (${VERSION_ID:-unknown})${NC}"
else
    OS="unknown"
fi

install_deps_debian() {
    export DEBIAN_FRONTEND=noninteractive
    sudo apt-get update -qq
    sudo apt-get install -y -qq build-essential curl git python3 python3-pip jq pkg-config libssl-dev
}

install_deps_rhel() {
    sudo dnf groupinstall -y "Development Tools" || sudo yum groupinstall -y "Development Tools"
    sudo dnf install -y curl git python3 python3-pip jq openssl-devel || sudo yum install -y curl git python3 python3-pip jq openssl-devel
}

if command -v apt-get >/dev/null 2>&1; then
    install_deps_debian
elif command -v dnf >/dev/null 2>&1 || command -v yum >/dev/null 2>&1; then
    install_deps_rhel
else
    echo -e "${YELLOW}⚠️ 未知套件管理器，請確認系統已安裝: gcc, make, curl, git, python3, jq${NC}"
fi

# 2. 檢測或安裝 Rust 工具鏈
echo -e "\n${BLUE}[Step 2/6] 檢查 Rust 編譯工具鏈 (rustc & cargo)...${NC}"
if ! command -v cargo >/dev/null 2>&1; then
    echo -e "未發現 Rust，正在透過 rustup 官方安裝最新穩定版..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # 載入 Cargo 環境變數
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
else
    echo -e "Rust 工具鏈已就緒: ${GREEN}$(rustc --version)${NC}"
fi

# 3. 建立雲端專案主目錄
echo -e "\n${BLUE}[Step 3/6] 佈局專案目錄與倉庫克隆...${NC}"
BASE_DIR="${CLOUD_WORKSPACE_DIR:-$HOME/workspace}"
mkdir -p "$BASE_DIR"
cd "$BASE_DIR"
echo -e "主工作區路徑: ${CYAN}$BASE_DIR${NC}"

# 克隆或更新 rust-bricks
if [ ! -d "$BASE_DIR/rust-bricks" ]; then
    echo -e "正在克隆主微積木庫 rust-bricks..."
    git clone https://github.com/mangala1728/rust-bricks.git "$BASE_DIR/rust-bricks"
else
    echo -e "更新現有 rust-bricks..."
    git -C "$BASE_DIR/rust-bricks" pull origin main
fi

# 檢查或提示 digital-historian
if [ ! -d "$BASE_DIR/digital-historian" ]; then
    echo -e "${YELLOW}ℹ️ digital-historian 尚未在同層目錄。若您有獨立遠端倉庫，可執行：${NC}"
    echo -e "   git clone <digital-historian-url> \"$BASE_DIR/digital-historian\""
fi

# 4. 配置環境變數模板
echo -e "\n${BLUE}[Step 4/6] 配置雲端金鑰與農夫環境變數...${NC}"
ENV_FILE="$BASE_DIR/rust-bricks/.env"
if [ ! -f "$ENV_FILE" ]; then
    cat << 'EOF' > "$ENV_FILE"
# 雲端微積木農夫與品質守衛環境設定
# 請依據您的雲端權限填入金鑰 (多組金鑰以逗號分隔，自動多帳號輪詢)
GITHUB_TOKEN=""
GEMINI_API_KEYS=""
GROQ_API_KEYS=""

# 守衛與防線控制
RUST_BACKTRACE=1
RUST_LOG=info
EOF
    echo -e "已生成環境變數範本: ${CYAN}$ENV_FILE${NC}"
    echo -e "${YELLOW}👉 請依需求編輯此檔填入 API 金鑰以啟用自動背景農耕。${NC}"
else
    echo -e "環境變數檔已存在: ${GREEN}$ENV_FILE${NC}"
fi

# 5. 執行雙端下水道標準同構性檢查
echo -e "\n${BLUE}[Step 5/6] 執行領域下水道標準同步校驗 (Domain Pipe Standards)...${NC}"
cd "$BASE_DIR/rust-bricks"
if [ -d "$BASE_DIR/digital-historian" ]; then
    python3 scripts/sync_domain_standards.py --check || {
        echo -e "${YELLOW}⚠️ 發現標準未完全同步，自動嘗試雙向校準...${NC}"
        python3 scripts/sync_domain_standards.py --sync
    }
else
    echo -e "本機便攜標準自檢..."
    python3 portable/domain-pipe-standards/test_astro_pipeline.py
fi

# 6. 執行全套品質守衛 (brick-guard) 與完整編譯測試
echo -e "\n${BLUE}[Step 6/6] 執行零依賴 09 合約品質守衛與單元測試...${NC}"
cd "$BASE_DIR/rust-bricks"
cargo run -q -- guard
cargo test -q

echo -e "\n${GREEN}================================================================${NC}"
echo -e "${GREEN}  🎉 雲端冷啟動成功！所有 09 合約守衛與 120+ 測試全數綠燈通過！  ${NC}"
echo -e "${GREEN}================================================================${NC}"
echo -e "常用管理指令："
echo -e "  • 檢查遠端貢獻分支:   ${CYAN}cargo run -- patrol${NC}"
echo -e "  • 一鍵零衝突收割:     ${CYAN}cargo run -- harvest${NC}"
echo -e "  • 查看規格佇列狀態:   ${CYAN}cargo run -- farm-status${NC}"
echo -e "  • 啟動背景雲端農夫:   ${CYAN}cargo run -- farm <spec_path> <module_name>${NC}"
echo -e "  • 更新目錄索引:       ${CYAN}cargo run -- catalog${NC}"
