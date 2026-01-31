# Wave 1 Performance Benchmark Results

**Date:** 2026-01-31
**Platform:** macOS Darwin 25.2.0 (Apple Silicon)
**Build:** Release profile (optimized)
**Tool:** Criterion 0.5

---

## Human Design Engine

| Benchmark | Time | Target | Status |
|-----------|------|--------|--------|
| Full HD chart (engine.calculate) | 463 us (0.46ms) | <100ms | PASS |
| Chart generation (generate_hd_chart) | 379 us (0.38ms) | <100ms | PASS |
| 26 activations (13 personality + 13 design) | 434 us (0.43ms) | <5ms | PASS |
| 13 personality activations | 80 us (0.08ms) | <5ms | PASS |
| Type determination | 26 ns | <1ms | PASS |
| Authority determination | 38 ns | <1ms | PASS |
| Channel detection (13 activations) | 237 ns | <1ms | PASS |
| Center definition (8 activations) | 1.47 us | <1ms | PASS |
| Ephemeris Moon lookup | 6.71 us | <10ms | PASS |
| Ephemeris Sun lookup | 6.60 us | <10ms | PASS |
| Longitude to gate (360 conversions) | 3.32 us | <1ms | PASS |
| Profile calculation | 1.42 ns | <1ms | PASS |

**Summary:** All HD benchmarks well within targets. Full chart at 0.46ms is 217x faster than the 100ms target. Swiss Ephemeris lookups are sub-7us. Type/Authority determination is sub-40ns (pure logic).

---

## Gene Keys Engine

| Benchmark | Time | Target | Status |
|-----------|------|--------|--------|
| Full GK chart (engine.calculate, Mode 2) | 11.7 us (0.012ms) | <50ms | PASS |
| Activation sequences | 3.78 ns | <5ms | PASS |
| Frequency assessment (4 keys, level 3) | 2.44 us | <5ms | PASS |
| Transformation pathways | 1.68 us | <5ms | PASS |
| Complete pathways (Shadow+Gift) | 3.43 us | <5ms | PASS |
| Witness prompt generation | 140 ns | <5ms | PASS |
| Gene Key lookup (all 64) | 298 ns | <1ms | PASS |
| Individual Gene Key lookup | 6.5-7.4 ns | <1ms | PASS |
| Frequency by level (Shadow/Gift/Siddhi) | 2.4-2.5 us | <5ms | PASS |

**Summary:** All GK benchmarks dramatically under targets. Full chart at 0.012ms is 4,167x faster than the 50ms target. Pure data lookups are sub-10ns. Frequency assessments are sub-3us.

---

## Vimshottari Engine

| Benchmark | Time | Target | Status |
|-----------|------|--------|--------|
| Full 120-year timeline (engine.calculate) | 35.1 us (0.035ms) | <200ms | PASS |
| Nakshatra from longitude (pure) | 2.01 ns | <10ms | PASS |
| Nakshatra from ephemeris (Swiss Ephe) | 21.0 us | <10ms | PASS |
| Dasha balance | 1.77 ns | <1ms | PASS |
| 9 Mahadashas | 254 ns | <5ms | PASS |
| 81 Antardashas | 2.27 us | <5ms | PASS |
| 729 Pratyantardashas (complete timeline) | 16.1 us | <100ms | PASS |
| Current period search (binary) | 1.18 us | <5ms | PASS |
| Period search at 1y/10y/30y/60y/100y | 1.12-1.19 us | <5ms | PASS |
| Upcoming transitions (5) | 2.44 us | <5ms | PASS |
| Period enrichment | 864 ns | <1ms | PASS |
| Nakshatra lookup (all 27) | 21.3 ns | <1ms | PASS |
| Timeline by nakshatra (Ashwini/Magha/Mula) | 16.1-17.4 us | <100ms | PASS |

**Summary:** All Vim benchmarks far exceed targets. Full 120-year timeline at 0.035ms is 5,714x faster than the 200ms target. Binary search through 729 periods takes 1.18us. Complete 729-period timeline builds in 16us.

---

## Engine Comparison (Side-by-Side)

| Engine | Calculate Time | Validate Time | Ratio |
|--------|---------------|---------------|-------|
| Human Design | 399 us | 88 ns | 1.0x (baseline) |
| Gene Keys (Mode 2) | 11.7 us | 93 ns | 34x faster |
| Vimshottari (Mode 2) | 32.2 us | 90 ns | 12x faster |

**Notes:**
- HD is slowest due to Swiss Ephemeris calls (2x for personality + design time)
- GK Mode 2 (pre-computed gates) skips ephemeris entirely
- Vimshottari Mode 2 (moon_longitude) also skips ephemeris
- All validation times are sub-100ns (negligible)

---

## Performance vs Target Summary

| Metric | Target | Actual | Margin |
|--------|--------|--------|--------|
| HD Full Chart | <100ms | 0.46ms | 217x |
| GK Full Chart | <50ms | 0.012ms | 4,167x |
| Vim Full Timeline | <200ms | 0.035ms | 5,714x |
| HD Type/Authority | <1ms | 38ns | 26,316x |
| HD Ephemeris Lookup | <10ms | 6.7us | 1,493x |
| Vim 729 Pratyantardashas | <100ms | 16.1us | 6,211x |
| Vim Current Period | <5ms | 1.18us | 4,237x |

**All 32 benchmarks PASS. All targets exceeded by orders of magnitude.**

---

## Benchmark Files

- HD: `crates/engine-human-design/benches/hd_performance.rs` (12 benchmarks)
- GK: `crates/engine-gene-keys/benches/gk_performance.rs` (9 benchmarks + parameterized)
- Vim: `crates/engine-vimshottari/benches/vim_performance.rs` (13 benchmarks + parameterized)
- Comparison: `crates/noesis-api/benches/engine_comparison.rs` (6 benchmarks)

## Run Commands

```bash
cargo bench --package engine-human-design --bench hd_performance
cargo bench --package engine-gene-keys --bench gk_performance
cargo bench --package engine-vimshottari --bench vim_performance
cargo bench --package noesis-api --bench engine_comparison
```
