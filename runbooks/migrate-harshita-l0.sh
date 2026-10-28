#!/usr/bin/env bash
# Copy a solo's new-l0-flow/ output into the canonical premium-assets factory layout.
#
# Usage: runbooks/migrate-harshita-l0.sh <src-new-l0-flow-dir> <dst-person-dir>
#   src: .../723/Solos/harshita/new-l0-flow
#   dst: .../723/witness-agents-archive/.premium-assets-witness-harshita/harshita
set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "Usage: $0 <src-new-l0-flow-dir> <dst-person-dir>" >&2
  exit 2
fi

SRC="$1"
DST="$2"

mkdir -p "$DST/source-pack" "$DST/local"
cp "$SRC/source-pack/manifest.json" "$DST/source-pack/manifest.json"
cp "$SRC/source-pack/reading.md" "$DST/source-pack/reading.md"
cp "$SRC/source-pack/reflection-questions.md" "$DST/source-pack/reflection-questions.md"
cp "$SRC/engines.json" "$DST/source-pack/engines.json"
cp "$SRC/report.html" "$DST/local/reading.html"
cp "$SRC/report.pdf" "$DST/local/reading.pdf"

echo "Migrated $SRC -> $DST"
