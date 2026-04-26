#!/usr/bin/env bash
# Backward-compatible wrapper for legacy command name.

set -euo pipefail

echo "scripts/import-tasks-to-github.sh is deprecated."
echo "Routing to scripts/sync-plans-to-github-issues.sh"

exec "$(dirname "$0")/sync-plans-to-github-issues.sh" "$@"
