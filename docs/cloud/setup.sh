#!/bin/bash
# Setup script for the shared "thoughtseed" Claude Code cloud environment.
# Paste into: cloud environment menu -> Edit -> Setup script.
#
# Runs once per environment cache (about every 7 days, or when this script or
# the network allowlist changes). The resulting filesystem is snapshotted, so
# everything installed here is on disk when a session starts. It cannot read
# the environment's variables or secrets, and must finish in about 5 minutes.
#
# Toolchains are pre-installed (Node 22, pnpm, Bun, Rust, Python 3, Postgres 16,
# Redis 7, Docker, Chromium for Playwright), so this script only:
#   1. builds the Swagger UI zip that Selemene's Rust build needs offline, and
#   2. runs each repository's SessionStart hook, so single-repo and multi-repo
#      sessions both start with dependencies installed. (Multi-repo sessions do
#      not run repository hooks themselves.)
set -uo pipefail

# 1. utoipa-swagger-ui downloads a GitHub archive at build time, which the
#    cloud proxy refuses. Build the identical zip from the npm registry.
#    Keep the version in step with .claude/hooks/session-start.sh in Selemene.
swagger_version=5.17.14
swagger_zip="$HOME/.cache/selemene/swagger-ui-$swagger_version.zip"
if [ ! -s "$swagger_zip" ]; then
  work="$(mktemp -d)"
  (cd "$work" && npm pack "swagger-ui-dist@$swagger_version" --silent >/dev/null \
    && tar xzf "swagger-ui-dist-$swagger_version.tgz" \
    && mkdir "swagger-ui-$swagger_version" \
    && mv package "swagger-ui-$swagger_version/dist" \
    && python3 -I -c "import shutil,sys; shutil.make_archive(sys.argv[1], 'zip', '.', sys.argv[2])" \
      "swagger-ui-$swagger_version" "swagger-ui-$swagger_version") \
    && mkdir -p "$(dirname "$swagger_zip")" \
    && mv "$work/swagger-ui-$swagger_version.zip" "$swagger_zip"
  rm -rf "$work"
fi

# 2. Pre-install each cloned repository's dependencies. A failure here is
#    logged but does not fail setup: the repo hook retries at session start.
for hook in /home/user/*/.claude/hooks/session-start.sh; do
  [ -x "$hook" ] || continue
  repo="$(cd "$(dirname "$hook")/../.." && pwd)"
  echo "setup: running $hook"
  CLAUDE_CODE_REMOTE=true CLAUDE_PROJECT_DIR="$repo" "$hook" \
    || echo "setup: $hook failed; it will retry at session start"
done

exit 0
