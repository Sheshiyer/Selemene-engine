#!/usr/bin/env bash
set -euo pipefail

DEFAULT_SOURCE="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos/harshita/new-l0-flow"
DEFAULT_DESTINATION="/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-harshita/harshita"

SOURCE="${SELEMENE_MIGRATION_SOURCE:-$DEFAULT_SOURCE}"
DESTINATION="${SELEMENE_MIGRATION_DESTINATION:-$DEFAULT_DESTINATION}"

REQUIRED_SOURCE_FILES=(
  "$SOURCE/source-pack/manifest.json"
  "$SOURCE/source-pack/reading.md"
  "$SOURCE/source-pack/reflection-questions.md"
  "$SOURCE/engines.json"
  "$SOURCE/report.html"
  "$SOURCE/report.pdf"
)

for source_file in "${REQUIRED_SOURCE_FILES[@]}"; do
  if [[ ! -f "$source_file" ]]; then
    echo "Missing required source file: ${source_file}" >&2
    exit 1
  fi
done

mkdir -p "$DESTINATION/source-pack" "$DESTINATION/local"

cp "$SOURCE/source-pack/manifest.json" "$DESTINATION/source-pack/manifest.json"
cp "$SOURCE/source-pack/reading.md" "$DESTINATION/source-pack/reading.md"
cp "$SOURCE/source-pack/reflection-questions.md" "$DESTINATION/source-pack/reflection-questions.md"
cp "$SOURCE/engines.json" "$DESTINATION/source-pack/engines.json"
cp "$SOURCE/report.html" "$DESTINATION/local/reading.html"
cp "$SOURCE/report.pdf" "$DESTINATION/local/reading.pdf"

echo "Migrated $SOURCE -> $DESTINATION"
