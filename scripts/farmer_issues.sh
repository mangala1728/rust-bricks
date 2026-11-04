#!/usr/bin/env bash
# 依 git 推導的佇列狀態開關 issue（日誌保鮮：狀態一變就自動關閉過期的 issue，不留舊結論）
#   farmer:blocked       同一版規格累積放棄達上限
#   farmer:spec-suspect  不同模型卡在同一個黃金向量
#   farmer:queue-empty   沒有可做的規格
set -euo pipefail

TMP="${RUNNER_TEMP:-/tmp}"
kv=$(cargo run -q -- farm-queue)
get() { printf '%s\n' "$kv" | sed -n "s/^$1=//p"; }
blocked=$(get blocked)
suspect=$(get suspect)
pending=$(get pending_count)

ensure_label() { gh label create "$1" --color "$2" --description "$3" --force >/dev/null 2>&1 || true; }
ensure_label "farmer:blocked" D93F0B "雲端農夫卡關，需要人類裁判"
ensure_label "farmer:spec-suspect" FBCA04 "不同模型卡在同一個黃金向量，懷疑規格"
ensure_label "farmer:queue-empty" 0E8A16 "雲端農夫沒有可做的規格"

# $1=label $2=逗號分隔的模組清單 $3=標題前綴
sync_module_issues() {
  local label="$1" list="$2" prefix="$3" m title count
  for m in ${list//,/ }; do
    title="${prefix}：${m}"
    count=$(gh issue list --label "$label" --state open --limit 200 --json title \
      --jq "[.[] | select(.title == \"${title}\")] | length")
    if [ "$count" = "0" ]; then
      cargo run -q -- farm-report "$m" > "$TMP/farm-report.md"
      gh issue create --label "$label" --title "$title" --body-file "$TMP/farm-report.md"
    fi
  done
  gh issue list --label "$label" --state open --limit 200 --json number,title --jq '.[] | "\(.number)\t\(.title)"' |
    while IFS=$'\t' read -r num t; do
      m="${t#"${prefix}："}"
      case ",${list}," in
        *",${m},"*) ;;
        *) gh issue close "$num" --comment "依 git 狀態自動關閉：\`${m}\` 已不在此狀態（已入庫、規格改版或死路分支已刪除）。" ;;
      esac
    done
}

sync_module_issues "farmer:blocked" "$blocked" "雲端農夫卡關"
sync_module_issues "farmer:spec-suspect" "$suspect" "雲端農夫懷疑規格"

open_empty=$(gh issue list --label "farmer:queue-empty" --state open --json number --jq '.[0].number // empty')
if [ "$pending" = "0" ] && [ -z "$open_empty" ]; then
  cargo run -q -- farm-status > "$TMP/farm-status.md"
  printf '\n雲端農夫目前沒有可做的規格，排程會空轉。請在 `specs/` 補充新規格（含 ```golden 區塊，見 `specs/template.md`）；推送到 main 會立即觸發農作。\n\n（佇列一有待做規格，本 issue 會自動關閉。）\n' >> "$TMP/farm-status.md"
  gh issue create --label "farmer:queue-empty" --title "雲端農夫佇列已空：請補充 specs/" --body-file "$TMP/farm-status.md"
fi
if [ "$pending" != "0" ] && [ -n "$open_empty" ]; then
  gh issue close "$open_empty" --comment "佇列已有 ${pending} 份待做規格，自動關閉。"
fi
