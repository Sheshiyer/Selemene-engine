#!/usr/bin/env bash
# Trigger the GitHub Copilot coding agent on agent/issue-* draft PRs by
# posting an @copilot kickoff comment that points at .agent-tasks/issue-<N>.md.
#
# Modes:
#   --pr <num>      Kick a single PR by number
#   --issue <num>   Resolve agent/issue-<num>-* head ref -> PR -> kick
#   --all-ready     Walk every open agent/issue-* draft PR; skip ones already
#                   kicked (any existing comment whose body starts with @copilot)

set -euo pipefail

REPO="${GITHUB_REPOSITORY:-}"
PR_NUM=""
ISSUE_NUM=""
ALL_READY=false
LANE="all"
DRY_RUN=false
SLEEP_SECONDS="0.5"

usage() {
  cat <<'EOF'
Usage:
  scripts/dispatch-copilot.sh [options]

Modes (exactly one of):
  --pr <num>            Kick a single PR by number
  --issue <num>         Resolve agent/issue-<num>-* head ref -> PR -> kick
  --all-ready           Kick every un-kicked open agent/issue-* draft PR

Options:
  --repo <owner/repo>   Repo (default: $GITHUB_REPOSITORY or `gh repo view`)
  --lane <name>         Filter --all-ready by lane: qa-docs|backend|all (default: all)
                        Mapping matches agent-merge-lane.yml:
                          area:qa, area:docs       -> qa-docs
                          area:backend, area:data  -> backend
                          (anything else)          -> all
  --dry-run             Print intended actions, post nothing
  --sleep-seconds <f>   Delay between kicks in --all-ready (default: 0.5)
  --help, -h            Show help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --pr)             PR_NUM="${2:-}";          shift 2 ;;
    --issue)          ISSUE_NUM="${2:-}";       shift 2 ;;
    --all-ready)      ALL_READY=true;           shift ;;
    --lane)           LANE="${2:-all}";         shift 2 ;;
    --repo)           REPO="${2:-}";            shift 2 ;;
    --dry-run)        DRY_RUN=true;             shift ;;
    --sleep-seconds)  SLEEP_SECONDS="${2:-0.5}"; shift 2 ;;
    --help|-h)        usage; exit 0 ;;
    *)                echo "Unknown option: $1" >&2; usage; exit 1 ;;
  esac
done

case "$LANE" in
  qa-docs|backend|all) ;;
  *) echo "Error: --lane must be one of: qa-docs, backend, all (got: $LANE)" >&2; exit 1 ;;
esac

for bin in gh jq; do
  if ! command -v "$bin" >/dev/null 2>&1; then
    echo "Error: required binary '$bin' not found." >&2
    exit 1
  fi
done

if [[ -z "$REPO" ]]; then
  REPO="$(gh repo view --json nameWithOwner --jq '.nameWithOwner' 2>/dev/null || true)"
fi
if [[ -z "$REPO" ]]; then
  echo "Error: repo is required (pass --repo or set GITHUB_REPOSITORY)." >&2
  exit 1
fi

mode_count=0
[[ -n "$PR_NUM" ]] && mode_count=$((mode_count + 1))
[[ -n "$ISSUE_NUM" ]] && mode_count=$((mode_count + 1))
[[ "$ALL_READY" == true ]] && mode_count=$((mode_count + 1))
if [[ "$mode_count" -ne 1 ]]; then
  echo "Error: pass exactly one of --pr, --issue, --all-ready." >&2
  usage
  exit 1
fi

# Build kickoff body (issue number injected per-PR).
kickoff_body() {
  local n="$1"
  cat <<EOF
@copilot Implement everything in \`.agent-tasks/issue-${n}.md\`.

Acceptance:
- All items in the issue's Acceptance Criteria pass
- \`cargo build && cargo test\` is green
- Delete \`.agent-tasks/issue-${n}.md\` in your final commit
- Convert this PR from draft to ready-for-review
- Post \`agent: done -- tests passing, ready for review\`
EOF
}

# Extract issue number from agent/issue-<N>-... head ref. Echoes "" if no match.
issue_num_from_head() {
  local head="$1"
  echo "$head" | sed -nE 's#^agent/issue-([0-9]+)-.*$#\1#p'
}

# Mirror of agent-merge-lane.yml::laneForArea — keep in sync.
lane_for_area() {
  case "$1" in
    "area:qa"|"area:docs")     echo "qa-docs" ;;
    "area:backend"|"area:data") echo "backend" ;;
    *)                          echo "all" ;;
  esac
}

# Fetch the area:* label for an issue number (echoes "" if none / lookup fails).
area_for_issue() {
  local n="$1"
  [[ -z "$n" ]] && { echo ""; return; }
  gh issue view "$n" --repo "$REPO" --json labels \
    --jq '[.labels[].name | select(startswith("area:"))][0] // ""' 2>/dev/null || echo ""
}

# Has this PR already been kicked? (any comment body starting with "@copilot")
already_kicked() {
  local pr="$1"
  local count
  count="$(
    gh pr view "$pr" --repo "$REPO" --json comments \
      --jq '[.comments[].body | select(startswith("@copilot"))] | length' 2>/dev/null || echo 0
  )"
  [[ "$count" -gt 0 ]]
}

# Resolve --issue N to PR number; echoes "" if no matching open PR.
pr_for_issue() {
  local issue="$1"
  gh pr list --repo "$REPO" --state open --limit 200 \
    --json number,headRefName \
    --jq ".[] | select(.headRefName | startswith(\"agent/issue-${issue}-\")) | .number" \
    | head -n 1
}

# Kick a single PR (skips if already kicked).
kick_pr() {
  local pr="$1"
  local head="$2"

  local n
  n="$(issue_num_from_head "$head")"
  if [[ -z "$n" ]]; then
    echo "  skip PR #${pr}: head ref '${head}' is not agent/issue-<N>-*"
    return 0
  fi

  if already_kicked "$pr"; then
    echo "  skip PR #${pr} (issue #${n}): already has an @copilot comment"
    return 0
  fi

  if [[ "$DRY_RUN" == true ]]; then
    echo "  dry-run kick PR #${pr} (issue #${n})"
    return 0
  fi

  local body
  body="$(kickoff_body "$n")"
  gh pr comment "$pr" --repo "$REPO" --body "$body" >/dev/null
  echo "  kicked PR #${pr} (issue #${n})"
}

echo "Copilot dispatch start"
echo "  repo: $REPO"
echo "  mode: $([[ "$DRY_RUN" == true ]] && echo dry-run || echo apply)"
echo "  lane: $LANE"

if [[ -n "$PR_NUM" ]]; then
  head="$(gh pr view "$PR_NUM" --repo "$REPO" --json headRefName --jq '.headRefName')"
  kick_pr "$PR_NUM" "$head"
elif [[ -n "$ISSUE_NUM" ]]; then
  pr="$(pr_for_issue "$ISSUE_NUM")"
  if [[ -z "$pr" ]]; then
    echo "Error: no open PR with head ref agent/issue-${ISSUE_NUM}-* in $REPO" >&2
    exit 1
  fi
  kick_pr "$pr" "agent/issue-${ISSUE_NUM}-"
else
  # --all-ready: enumerate open agent/issue-* PRs (drafts AND non-drafts; the
  # already-kicked check is what gates re-posting, not draft state). Avoid
  # `mapfile` for macOS bash 3.2 compatibility.
  candidates="$(
    gh pr list --repo "$REPO" --state open --limit 200 \
      --json number,headRefName,isDraft \
      --jq '.[] | select(.headRefName | startswith("agent/issue-")) | "\(.number)\t\(.headRefName)\t\(.isDraft)"'
  )"

  if [[ -z "$candidates" ]]; then
    count=0
  else
    count="$(printf '%s\n' "$candidates" | wc -l | tr -d ' ')"
  fi
  echo "  candidates: $count"

  if [[ "$count" -gt 0 ]]; then
    while IFS=$'\t' read -r pr head _is_draft; do
      [[ -z "$pr" ]] && continue

      if [[ "$LANE" != "all" ]]; then
        n="$(issue_num_from_head "$head")"
        area="$(area_for_issue "$n")"
        pr_lane="$(lane_for_area "$area")"
        if [[ "$pr_lane" != "$LANE" ]]; then
          echo "  skip PR #${pr} (issue #${n:-?}): lane=${pr_lane:-unknown} != ${LANE} (area=${area:-none})"
          continue
        fi
      fi

      kick_pr "$pr" "$head"
      sleep "$SLEEP_SECONDS"
    done <<< "$candidates"
  fi
fi

echo "Copilot dispatch complete."
