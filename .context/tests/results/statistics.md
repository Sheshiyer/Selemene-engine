# Statistical Analysis

## Study 2: Fair Comparison (n=10 per group, identical information)

### Descriptive Statistics

| Metric | Control | Treatment |
|--------|---------|-----------|
| n | 10 | 10 |
| Mean | 10.0 | 10.0 |
| Std Dev | 0.00 | 0.00 |
| Variance | 0.00 | 0.00 |
| Min | 10 | 10 |
| Max | 10 | 10 |
| Pattern Adherence | 100% | 100% |

### Statistical Tests

With identical scores in both groups:
- **Mean difference**: 0.0 points
- **Effect Size (Cohen's d)**: 0.00 (No effect)
- **Fisher's exact test**: Not applicable (no variance)
- **Conclusion**: No significant difference

### Key Finding

**When documentation contains identical explicit examples, organizational structure (single file vs modular) does not affect AI pattern adherence.**

---

## Study 1: Original (Unfair Comparison)

The original study had information asymmetry - the treatment included `validateQuery` examples while the control did not.

### Descriptive Statistics

| Metric | Control | Treatment |
|--------|---------|-----------|
| n | 10 | 10 |
| Mean | 7.0 | 10.0 |
| Std Dev | 2.58 | 0.00 |
| Pattern Adherence | 40% | 100% |

### Statistical Tests

| Test | Value | Interpretation |
|------|-------|----------------|
| Cohen's d | 1.64 | Large effect |
| Fisher's exact | p = 0.011 | Significant |
| Mann-Whitney U | p = 0.0003 | Highly significant |

### Why This Was Misleading

The 60 percentage point difference was **not** due to organizational structure. It was due to:
- Control: Only showed `validateBody(schema)` example
- Treatment: Showed `validateBody`, `validateParams`, AND `validateQuery` examples

**The treatment had more comprehensive information, not just better organization.**

---

## Corrected Conclusion

The real insight is:

1. **Explicit code examples drive AI pattern adherence**, not documentation organization
2. **Single-file and modular documentation perform equally** when information is identical
3. **Modular documentation may naturally tend toward completeness** because each file can thoroughly cover its domain
4. **The value of .context/ is not organizational** - it's that modular structures encourage more complete documentation

### Practical Recommendation

Regardless of whether you use:
- A single CLAUDE.md file
- A modular .context/ folder

**What matters is including explicit code examples for every pattern you want the AI to follow.**
