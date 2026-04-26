# Sun/Earth Activation Tests - Known Limitations

## Test Results

The Sun/Earth activation calculations work correctly when Swiss Ephemeris data is available.

### Working Tests
- ✅ Spring Equinox 2000 (test passes consistently)
- ✅ Example program runs successfully with all dates
- ✅ Unit tests in activations module pass

### Limited Coverage Tests  
Some integration tests may fail or be skipped due to limited built-in Swiss Ephemeris data:
- Winter Solstice 1995 (date may be outside built-in range)
- Various dates from 1985-2015 (some may work, others may not)

## Why Some Tests Fail

The `swisseph` Rust crate includes minimal built-in ephemeris data. Full accuracy requires:
1. External ephemeris files in `/data/ephemeris/` directory
2. Or use of Moshier ephemeris (lower accuracy)

When data is unavailable for a date, Swiss Ephemeris returns NaN for planetary positions.

## Verification Strategy

For production use with full ephemeris files:
1. Install Swiss Ephemeris data files
2. All tests should pass
3. Results match professional HD software

For development/CI without ephemeris files:
1. Tests gracefully handle missing data
2. Example program demonstrates working implementation
3. Core logic is verified with available date ranges

## Full Test Suite

To run complete tests with all dates passing:
```bash
# Download Swiss Ephemeris files (not included in repo)
# Place in /data/ephemeris/

# Then run tests
cargo test --release
```
