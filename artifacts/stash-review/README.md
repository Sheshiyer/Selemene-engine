# Stash Review Patches

Generated on 2026-04-30 from local stashes.

## Files
- stash-1.patch (from stash@{1})
- stash-3.patch (from stash@{3})
- stash-4.patch (from stash@{4})

## Inspect
- git apply --stat artifacts/stash-review/stash-1.patch
- git apply --stat artifacts/stash-review/stash-3.patch
- git apply --stat artifacts/stash-review/stash-4.patch

## Dry-run apply check
- git apply --check artifacts/stash-review/stash-1.patch
- git apply --check artifacts/stash-review/stash-3.patch
- git apply --check artifacts/stash-review/stash-4.patch

## Apply one patch
- git apply artifacts/stash-review/stash-1.patch

## Apply and stage one patch
- git apply --index artifacts/stash-review/stash-1.patch

## Recover original stash by immutable tag
- git stash apply stash-backup-2026-04-30-1
- git stash apply stash-backup-2026-04-30-3
- git stash apply stash-backup-2026-04-30-4
