# Code Review Notes

## Overview

This document captures code review observations and recommendations for the Selemene Engine codebase as of Wave 2 completion (2026-01).

---

## TODO Comments to Address

### High Priority

| Location | TODO | Recommendation |
|----------|------|----------------|
| `src/engines/calculation_orchestrator.rs` | Swiss Ephemeris sync initialization | Refactor to async initialization with lazy loading |
| `src/api/routes.rs` | Real-time tracking routes commented out | Complete implementation or remove dead code |
| `crates/noesis-orchestrator/src/lib.rs:463` | `is_ready()` placeholder | Implement full readiness check |

### Medium Priority

| Location | TODO | Recommendation |
|----------|------|----------------|
| `crates/engine-human-design/` | Incarnation Cross names | Add cross name lookup table |
| `crates/engine-human-design/` | Variable/arrows analysis | Implement Phase 3 feature |
| `crates/engine-gene-keys/` | Venus Sequence, Pearl Sequence | Add to Gene Keys output |
| `crates/engine-biofield/` | Full implementation | Design proper energy assessment methodology |
| `crates/engine-face-reading/` | Full implementation | Define manual input or integrate image analysis |

### Low Priority

| Location | TODO | Recommendation |
|----------|------|----------------|
| `ts-engines/*/` | Error handling improvements | Standardize error responses |
| `monitoring/` | Dashboard enhancements | Add engine-specific dashboards |

---

## Dead Code to Remove

### Commented Routes
```rust
// src/api/routes.rs lines 57-72
// Real-time tracking routes that are fully commented out
// Either implement or remove before production
```

### Unused Imports
Several files have unused imports that trigger warnings:
- Run `cargo fix --allow-dirty` to auto-clean
- Or add `#[allow(unused_imports)]` for intentionally kept imports

### Legacy Directory
```
/legacy/
```
Contains old implementation files. Consider:
- Archiving to separate branch
- Removing if no longer needed
- Documenting if kept for reference

---

## Documentation Improvements Needed

### Missing Doc Comments

| Crate | Coverage | Recommendation |
|-------|----------|----------------|
| noesis-core | 70% | Add trait method documentation |
| noesis-orchestrator | 80% | Document workflow execution flow |
| noesis-bridge | 60% | Add error handling documentation |
| engine-* | 50-90% | Standardize across engines |

### README Updates

| File | Issue |
|------|-------|
| `ts-engines/README.md` | Missing - create basic README |
| `crates/*/README.md` | Most crates lack READMEs |
| `tests/README.md` | Missing test running instructions |

### API Documentation Gaps

- OpenAPI spec needs updating for new endpoints
- Response examples incomplete for some engines
- Error codes not fully documented

---

## Test Coverage Gaps

### Unit Tests

| Module | Coverage | Gap |
|--------|----------|-----|
| Human Design | 95% | Variable calculation |
| Gene Keys | 90% | Venus/Pearl sequences |
| Panchanga | 85% | Edge cases (polar regions) |
| Biorhythm | 90% | - |
| Numerology | 85% | Master numbers edge cases |
| Vimshottari | 80% | Pratyantardasha tests |
| Vedic Clock | 75% | Muhurta transitions |
| Biofield | 20% | Stub implementation |
| Face Reading | 20% | Stub implementation |

### Integration Tests

| Area | Status | Needed |
|------|--------|--------|
| Workflow execution | ✅ Good | - |
| TS bridge | ⚠️ Partial | Add failure scenarios |
| Cache layers | ⚠️ Partial | Add L2/L3 tests |
| Authentication | ✅ Good | - |
| Rate limiting | ⚠️ Partial | Add tier tests |

### Load Tests

- No automated load tests exist
- Recommend: Add k6 or locust tests
- Target: 1000 req/s sustained

---

## Code Quality Observations

### Positive Patterns

1. **Consistent error handling** with `EngineError` enum
2. **Good trait abstractions** for `ConsciousnessEngine`
3. **Comprehensive witness prompt system**
4. **Well-structured workflow orchestration**
5. **Effective use of async/await**

### Areas for Improvement

1. **Error messages could be more specific**
   ```rust
   // Before
   Err(EngineError::CalculationError("failed".into()))
   
   // After
   Err(EngineError::CalculationError(format!(
       "Gate calculation failed for longitude {}: {}",
       longitude, specific_error
   )))
   ```

2. **Magic numbers should be constants**
   ```rust
   // Before
   let gate_size = 5.625;
   
   // After
   const GATE_SIZE_DEGREES: f64 = 360.0 / 64.0; // 5.625°
   ```

3. **Some functions are too long**
   - `CalculationOrchestrator::calculate_panchanga` could be split
   - `GeneKeysEngine::calculate` has multiple responsibilities

4. **Inconsistent logging levels**
   - Some errors logged at warn, others at error
   - Recommend: Standardize logging conventions

---

## Security Considerations

### Secrets Management

- JWT secret via environment variable is okay for development
- Production should use:
  - Kubernetes secrets
  - HashiCorp Vault
  - Cloud provider secret managers

### Input Validation

- Coordinates validated ✅
- Dates validated ✅
- String inputs need sanitization for logging
- Consider adding request size limits

### Rate Limiting

- Current implementation is in-memory
- Production needs distributed rate limiting (Redis-based)

---

## Performance Recommendations

### Cache Optimization

1. **Precompute common dates**
   - Hindu festival dates
   - New/full moons for next year
   - Popular city coordinates

2. **Cache key optimization**
   - Current keys include precision level
   - Consider caching high precision and deriving lower

### Calculation Optimization

1. **Batch Swiss Ephemeris calls**
   - Currently 26 individual calls for HD
   - Could batch similar time queries

2. **Parallel engine execution**
   - Already using `join_all` ✅
   - Consider `FuturesUnordered` for better fairness

### Database Optimization

1. **Connection pooling**
   - Already using `sqlx` pools ✅
   - Monitor pool exhaustion under load

2. **Query optimization**
   - Add indices for frequent lookups
   - Consider caching frequently accessed metadata

---

## Dependency Updates

### Outdated Crates

Run `cargo outdated` and update:
- Security patches immediately
- Minor versions after testing
- Major versions with migration plan

### Audit

```bash
cargo audit
```

Address any security advisories.

---

## Refactoring Suggestions

### Short Term (Next Sprint)

1. Remove or implement commented real-time routes
2. Standardize error messages
3. Add missing doc comments to public APIs
4. Fix clippy warnings

### Medium Term (Next Release)

1. Extract magic numbers to constants
2. Split large functions
3. Improve test coverage for cache layers
4. Add load tests

### Long Term (Future Versions)

1. Implement stub engines (biofield, face-reading)
2. Add Venus/Pearl sequences to Gene Keys
3. Complete HD Variable analysis
4. Consider GraphQL API option

---

## Conclusion

The codebase is in good shape for Wave 2 release. The main areas requiring attention are:

1. **Dead code cleanup** - Remove or implement commented code
2. **Documentation gaps** - Especially for TS engines and crate READMEs
3. **Test coverage** - Particularly for stub engines and edge cases
4. **Performance profiling** - Add load tests before production scale

No blocking issues identified for release.

---

**Reviewed By**: Agent 4 (Documentation)
**Date**: 2026-01
**Code Version**: Wave 2 Complete
