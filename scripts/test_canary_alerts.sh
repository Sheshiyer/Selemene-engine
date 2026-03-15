#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RULES_PATH="$ROOT_DIR/monitoring/prometheus/alerts/noesis-alerts.yml"
TEST_PATH="$ROOT_DIR/monitoring/prometheus/tests/noesis-canary-alerts.test.yml"
PROMTOOL_VERSION="${PROMTOOL_VERSION:-2.54.1}"
PROMTOOL_BIN="${PROMTOOL_BIN:-}"
TMP_DIR=""

cleanup() {
  if [[ -n "$TMP_DIR" && -d "$TMP_DIR" ]]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT

normalize_os() {
  case "$(uname -s)" in
    Darwin) echo "darwin" ;;
    Linux) echo "linux" ;;
    *)
      echo "Unsupported OS for promtool auto-download: $(uname -s)" >&2
      exit 1
      ;;
  esac
}

normalize_arch() {
  case "$(uname -m)" in
    arm64|aarch64) echo "arm64" ;;
    x86_64|amd64) echo "amd64" ;;
    *)
      echo "Unsupported architecture for promtool auto-download: $(uname -m)" >&2
      exit 1
      ;;
  esac
}

download_promtool() {
  local os arch archive url extract_dir cache_root cached_bin
  os="$(normalize_os)"
  arch="$(normalize_arch)"
  cache_root="${XDG_CACHE_HOME:-$HOME/.cache}/selemene/promtool/${PROMTOOL_VERSION}/${os}-${arch}"
  cached_bin="$cache_root/promtool"

  if [[ -x "$cached_bin" ]]; then
    printf '%s\n' "$cached_bin"
    return 0
  fi

  TMP_DIR="$(mktemp -d)"
  archive="$TMP_DIR/prometheus.tar.gz"
  url="https://github.com/prometheus/prometheus/releases/download/v${PROMTOOL_VERSION}/prometheus-${PROMTOOL_VERSION}.${os}-${arch}.tar.gz"

  curl -L -o "$archive" "$url"
  tar -xzf "$archive" -C "$TMP_DIR"
  extract_dir="$TMP_DIR/prometheus-${PROMTOOL_VERSION}.${os}-${arch}"
  if [[ ! -x "$extract_dir/promtool" ]]; then
    echo "Downloaded archive did not contain promtool" >&2
    exit 1
  fi
  mkdir -p "$cache_root"
  cp "$extract_dir/promtool" "$cached_bin"
  chmod +x "$cached_bin"
  printf '%s\n' "$cached_bin"
}

resolve_promtool() {
  if [[ -n "$PROMTOOL_BIN" ]]; then
    printf '%s\n' "$PROMTOOL_BIN"
    return 0
  fi

  if command -v promtool >/dev/null 2>&1; then
    command -v promtool
    return 0
  fi

  download_promtool
}

main() {
  local promtool
  promtool="$(resolve_promtool)"

  "$promtool" check rules "$RULES_PATH"
  "$promtool" test rules "$TEST_PATH"

  echo "canary alert tests passed"
}

main "$@"
