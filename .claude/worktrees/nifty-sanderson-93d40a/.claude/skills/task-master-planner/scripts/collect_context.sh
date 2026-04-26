#!/usr/bin/env bash
set -euo pipefail

repo_root="${1:-.}"
output_file="${2:-context.bundle.md}"

if ! command -v rg >/dev/null 2>&1; then
  echo "rg not found; please install ripgrep." >&2
  exit 1
fi

{
  echo "# Context Bundle"
  echo
  echo "## Repo"
  echo "${repo_root}"
  echo
  echo "## Key Docs"
  for f in "DesignSpec.md" "ProjectArchitecture.md"; do
    if [[ -f "${repo_root}/${f}" ]]; then
      echo "- ${f}"
    fi
  done
  echo
  echo "## .context files"
  rg --files "${repo_root}/.context" | sed "s#^${repo_root}/##"
} > "${repo_root}/${output_file}"

echo "Wrote ${repo_root}/${output_file}"
