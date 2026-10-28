#!/bin/bash
# SessionStart hook for Claude Code cloud sessions. No-op on local machines.
# Installs the pnpm workspace and ts-engines, and prepares the offline Rust
# build inputs. Each install is skipped while its manifests are unchanged.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

cd "${CLAUDE_PROJECT_DIR:-.}"
hook=.claude/hooks/session-start.sh

# pnpm workspace (apps/*, packages/*)
hashes=node_modules/.session-start.sha256
if ! sha256sum --check --status "$hashes" 2>/dev/null; then
  rm -f "$hashes"
  pnpm install --frozen-lockfile
  sha256sum "$hook" package.json pnpm-lock.yaml pnpm-workspace.yaml > "$hashes"
fi

# ts-engines is a separate Bun project
hashes=ts-engines/node_modules/.session-start.sha256
if ! sha256sum --check --status "$hashes" 2>/dev/null; then
  rm -f "$hashes"
  (cd ts-engines && bun install --frozen-lockfile)
  sha256sum "$hook" ts-engines/package.json ts-engines/bun.lock > "$hashes"
fi

# utoipa-swagger-ui's build script downloads a GitHub archive, which the cloud
# proxy refuses. Build the same zip from the npm registry instead. 5.17.14 is
# the default for utoipa-swagger-ui 8.1.0; bump both together.
swagger_version=5.17.14
swagger_zip="$HOME/.cache/selemene/swagger-ui-$swagger_version.zip"
if [ ! -s "$swagger_zip" ]; then
  work="$(mktemp -d)"
  (cd "$work" && npm pack "swagger-ui-dist@$swagger_version" --silent >/dev/null \
    && tar xzf "swagger-ui-dist-$swagger_version.tgz" \
    && mkdir "swagger-ui-$swagger_version" \
    && mv package "swagger-ui-$swagger_version/dist" \
    && python3 -I -c "import shutil,sys; shutil.make_archive(sys.argv[1], 'zip', '.', sys.argv[2])" \
      "swagger-ui-$swagger_version" "swagger-ui-$swagger_version")
  mkdir -p "$(dirname "$swagger_zip")"
  mv "$work/swagger-ui-$swagger_version.zip" "$swagger_zip"
  rm -rf "$work"
fi

# Rust crates, so builds do not stall on the registry mid-task
hashes=target/.session-start-cargo.sha256
if ! sha256sum --check --status "$hashes" 2>/dev/null; then
  rm -f "$hashes"
  cargo fetch --locked
  mkdir -p target
  sha256sum "$hook" Cargo.lock > "$hashes"
fi

if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
  {
    echo "export SWAGGER_UI_DOWNLOAD_URL=file://$swagger_zip"
    echo "export SWISS_EPHEMERIS_PATH=$PWD/data/ephemeris"
    echo "export DATABASE_URL=\${DATABASE_URL:-postgres://postgres@localhost/noesis_test}"
  } >> "$CLAUDE_ENV_FILE"
fi
