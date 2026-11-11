#!/usr/bin/env bash
# Copy a solo's new-l0-flow/ output into the canonical premium-assets factory layout.
#
# Usage: runbooks/migrate-harshita-l0.sh <src-new-l0-flow-dir> <dst-person-dir>
#   src: .../723/Solos/harshita/new-l0-flow
#   dst: .../723/witness-agents-archive/.premium-assets-witness-harshita/harshita
#
# The migrated manifest gets `report_level` (REPORT_LEVEL, default L0) when the
# source manifest predates that field. A missing report.pdf is not fatal: the
# script copies everything else and prints the l0-render command that builds it.
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <src-new-l0-flow-dir> <dst-person-dir>" >&2
  exit 2
fi

SRC="$1"
DST="$2"
REPORT_LEVEL="${REPORT_LEVEL:-L0}"

mkdir -p "$DST/source-pack" "$DST/local"
cp "$SRC/source-pack/manifest.json" "$DST/source-pack/manifest.json"
cp "$SRC/source-pack/reading.md" "$DST/source-pack/reading.md"
cp "$SRC/source-pack/reflection-questions.md" "$DST/source-pack/reflection-questions.md"
cp "$SRC/engines.json" "$DST/source-pack/engines.json"
cp "$SRC/report.html" "$DST/local/reading.html"

# Older new-l0-flow manifests predate report_level; keep an existing value.
node -e '
  const fs = require("node:fs");
  const [path, level] = process.argv.slice(1);
  const manifest = JSON.parse(fs.readFileSync(path, "utf8"));
  if (manifest.report_level === undefined) {
    manifest.report_level = level;
    fs.writeFileSync(path, JSON.stringify(manifest, null, 2) + "\n");
  }
' "$DST/source-pack/manifest.json" "$REPORT_LEVEL"

echo "Migrated $SRC -> $DST"

if [ -f "$SRC/report.pdf" ]; then
  cp "$SRC/report.pdf" "$DST/local/reading.pdf"
else
  PERSON_ID="$(basename "$DST")"
  ARCHIVE_ROOT="$(dirname "$(dirname "$DST")")"
  echo "No report.pdf in $SRC; skipped local/reading.pdf. Render it from the migrated source pack:" >&2
  echo "  PREMIUM_ASSETS_ROOT=\"$ARCHIVE_ROOT\" BRAND_CONFIG_PATH=<brand-config.yaml> \\" >&2
  echo "    pnpm --filter @noesis/witness-pipeline exec tsx scripts/l0-render.ts $PERSON_ID" >&2
fi
