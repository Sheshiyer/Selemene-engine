#!/usr/bin/env bash
set -euo pipefail

SRC="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos/harshita/new-l0-flow"
DST="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-harshita/harshita"

mkdir -p "$DST/source-pack" "$DST/local"
cp "$SRC/source-pack/manifest.json" "$DST/source-pack/manifest.json"
cp "$SRC/source-pack/reading.md" "$DST/source-pack/reading.md"
cp "$SRC/source-pack/reflection-questions.md" "$DST/source-pack/reflection-questions.md"
cp "$SRC/engines.json" "$DST/source-pack/engines.json"
cp "$SRC/report.html" "$DST/local/reading.html"
cp "$SRC/report.pdf" "$DST/local/reading.pdf"

echo "Migrated $SRC -> $DST"
