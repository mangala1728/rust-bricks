#!/usr/bin/env bash
# 雲端農夫「壓力紅燈」（對應 mangala-workflow 的 PRESSURE_STREAK）：
# 連續 PRESSURE_LIMIT 輪失敗 → 開 issue 並凍結；issue 未關閉前每輪都凍結。
# 人類關閉 issue 即解凍，連續失敗只從最近一次關閉之後重新計數。
# 輸出：$GITHUB_OUTPUT 的 frozen=true|false
set -euo pipefail

LIMIT="${PRESSURE_LIMIT:-5}"
LABEL="farmer:pressure-red"
OUT="${GITHUB_OUTPUT:-/dev/stdout}"

gh label create "$LABEL" --color B60205 --description "雲端農夫壓力紅燈：連續失敗已凍結" --force >/dev/null 2>&1 || true

open_count=$(gh issue list --label "$LABEL" --state open --json number --jq 'length')
if [ "$open_count" != "0" ]; then
  echo "🧊 壓力紅燈 issue 尚未關閉，本輪凍結（關閉該 issue 即解凍）"
  echo "frozen=true" >> "$OUT"
  exit 0
fi

since=$(gh issue list --label "$LABEL" --state closed --limit 1 --json closedAt --jq '.[0].closedAt // "1970-01-01T00:00:00Z"')
conclusions=$(gh run list --workflow cloud_farmer.yml --limit 30 --json databaseId,status,conclusion,createdAt \
  --jq "[.[] | select(.status == \"completed\" and .databaseId != ${GITHUB_RUN_ID:-0} and .createdAt > \"${since}\" and (.conclusion == \"success\" or .conclusion == \"failure\"))] | .[].conclusion")

streak=0
for c in $conclusions; do
  [ "$c" = "failure" ] || break
  streak=$((streak + 1))
done
echo "連續失敗輪數：${streak}（上限 ${LIMIT}，自 ${since} 起計）"

if [ "$streak" -lt "$LIMIT" ]; then
  echo "frozen=false" >> "$OUT"
  exit 0
fi

body=$(cat <<MD
雲端農夫已連續 **${streak}** 輪失敗，觸發壓力紅燈，之後每輪自動凍結（不呼叫任何模型）。

連續失敗跨越不同規格時，多半是系統性問題，請先檢查：
- 金鑰池額度是否用完、金鑰是否失效（Actions secrets）
- 工具鏈或 \`guard.rs\` 規則是否有變動，導致所有產出都過不了
- 最近的 \`attempt/*-farm-*\` 分支的 \`## Dead-End\` 是否指向同一原因

處理完後**關閉本 issue 即解凍**，連續失敗從關閉時間點重新計數。

（本 issue 由 \`scripts/farmer_pressure.sh\` 自動開啟。）
MD
)
gh issue create --label "$LABEL" --title "雲端農夫壓力紅燈：連續 ${streak} 輪失敗，已凍結" --body "$body"
echo "frozen=true" >> "$OUT"
