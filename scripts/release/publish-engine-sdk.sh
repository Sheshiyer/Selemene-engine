#!/usr/bin/env bash
# Publish @selemene/engine-sdk to the public npm registry from a clean checkout.
# No GitHub CI by policy: this is the release gate. Run it yourself (npm may ask for a 2FA code).
#
#   scripts/release/publish-engine-sdk.sh [git-ref]      # default: origin/main
#   DRY_RUN=1 scripts/release/publish-engine-sdk.sh      # everything except the publish
set -euo pipefail

REF="${1:-origin/main}"
PKG="@selemene/engine-sdk"
REPO_ROOT="$(git rev-parse --show-toplevel)"
WT="$(mktemp -d "${TMPDIR:-/tmp}/engine-sdk-release.XXXXXX")"
cleanup() { git -C "$REPO_ROOT" worktree remove --force "$WT" >/dev/null 2>&1 || true; }
trap cleanup EXIT

echo "== 1. npm identity"
npm whoami >/dev/null 2>&1 || { echo "Not logged in to npm. Run: npm login"; exit 1; }
echo "   logged in as $(npm whoami)"

echo "== 2. clean checkout of $REF"
git -C "$REPO_ROOT" fetch -q origin
git -C "$REPO_ROOT" worktree add -q --detach "$WT" "$REF"
SHA="$(git -C "$WT" rev-parse --short HEAD)"
cd "$WT/packages/noesis-engine-sdk"
VERSION="$(node -p "require('./package.json').version")"
echo "   $PKG@$VERSION from $SHA"

echo "== 3. version not already published"
if npm view "$PKG@$VERSION" version >/dev/null 2>&1; then
  echo "   $PKG@$VERSION already exists on npm. Bump the version in a PR first."; exit 1
fi

echo "== 4. install, typecheck, test, build"
bun install --no-save >/dev/null
npm run -s typecheck
bun test
rm -rf dist && npm run -s build

echo "== 5. pack contents"
npm pack --dry-run 2>&1 | sed -n '/Tarball Contents/,/Tarball Details/p'
if npm pack --dry-run --json 2>/dev/null | grep -qE '"path": *"(src|tests)/'; then
  echo "   source or tests leaked into the tarball"; exit 1
fi

if [[ "${DRY_RUN:-0}" == "1" ]]; then echo "== DRY_RUN=1, stopping before publish"; exit 0; fi

echo "== 6. publish (public)"
npm publish --access public

echo "== 7. registry readback"
for i in 1 2 3 4 5 6; do npm view "$PKG@$VERSION" version >/dev/null 2>&1 && break; sleep 5; done
npm view "$PKG@$VERSION" name version dist.tarball dist.shasum
echo "   published $PKG@$VERSION from $SHA"
echo "   optional tag: git tag engine-sdk-v$VERSION $SHA && git push origin engine-sdk-v$VERSION"
