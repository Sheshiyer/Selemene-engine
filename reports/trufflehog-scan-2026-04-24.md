# TruffleHog Secret Scan Report

**Date:** 2026-04-24  
**Tool:** TruffleHog v3.95.2  
**Mode:** `--only-verified` (verified secrets only)

## Summary

| Metric | Value |
|--------|-------|
| Verified Secrets | **0** |
| Unverified Secrets | 0 |
| Chunks Scanned | 18,424 |
| Bytes Scanned | 114,479,664 |
| Scan Duration | ~26 seconds |

## Result

✅ **CLEAN** — Zero verified secrets found in repository history.

## Command

```bash
trufflehog git file://. --only-verified --json
```

## Raw Output

```json
{"level":"info-0","ts":"2026-04-24T19:20:00Z","logger":"trufflehog","msg":"finished scanning","chunks":18424,"bytes":114479664,"verified_secrets":0,"unverified_secrets":0,"scan_duration":"25.681634098s","trufflehog_version":"3.95.2","verification_caching":{"Hits":8,"Misses":23,"HitsWasted":0,"AttemptsSaved":8,"VerificationTimeSpentMS":30556}}
```

## CI Integration

The secret scan runs automatically in CI via the `secrets` job in `.github/workflows/test.yml`:

```yaml
secrets:
  name: Secret Scanning
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    - name: TruffleHog secret scan
      uses: trufflesecurity/trufflehog@main
      with:
        extra_args: --only-verified
```
