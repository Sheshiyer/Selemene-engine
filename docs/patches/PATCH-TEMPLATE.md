# Patch Template

Use this template when documenting patches discovered through reading verification.

---

## Header (copy to verification report)

```markdown
# Selemene Engine Patch List — {Subject Name}

**Verification date:** YYYY-MM-DD
**Subject:** {Full Name}
**Birth data:** {Date}, {Time} {Timezone}, {Location} ({Lat}°N/S, {Lon}°E/W), {Ayanamsa}
**Cross-validation sources:**
1. Selemene engine v{X.Y.Z} (live API)
2. humdes.com raw ravecart
3. Bodygraph PNGs / screenshots
4. Kundli tool (astrosage / Callisto / etc.)
5. Source readings (if any derived prose exists)
```

---

## Patch Entry Format

Each patch should include these 5 sections:

```markdown
### P{severity}.{number} — {Short Title}

**Found:** What was discovered during cross-validation. Be specific — include actual values from engine vs reference sources.

**Where it breaks:** The downstream impact. Who is affected? What downstream outputs are wrong?

**Likely cause:** (Optional) If you can identify the root cause, describe it here.

**Patch required:**
- File(s) to modify
- Specific changes needed
- Test case to add (use the current reading's birth data)

**Severity:** CRITICAL | HIGH | MEDIUM | LOW | DOCS
```

---

## Severity Definitions

| Level | Code | Criteria |
|-------|------|----------|
| **P0 Critical** | Blocking | Prevents premium-asset generation. Materially incorrect output that will confuse users. |
| **P1 High** | Urgent | Should be patched before next release. Significant accuracy gap vs reference sources. |
| **P2 Medium** | Quality | Improves output quality. Missing fields, incomplete data structures. |
| **P3 Low** | Polish | Convenience improvements. Naming, aliases, transliteration. |
| **P4 Docs** | Documentation | Audit trail, transparency, calculation traces. |

---

## Adding to Registry

After documenting patches, add each to `PATCH-REGISTRY.md`:

```markdown
| P{X}.{Y} | {SEVERITY} | {engine} | open | {subject}-{date} | {summary} | — |
```

Example:
```markdown
| P0.1 | CRITICAL | gene-keys | open | sahil-2026-06-24 | Full Hologenetic Profile missing | — |
```

---

## Verification Report Sections

A complete verification report should cover each engine system:

### System 1 — Vedic Sidereal (D1 chart)
Compare: Planet positions, signs, houses, nakshatras, dignity flags

### System 2 — Vimshottari Dasha
Compare: Mahadasha/Antardasha sequence and date boundaries

### System 3 — Panchanga
Compare: Vara, Tithi, Nakshatra, Yoga, Karana, Charan, Yoni, Gan, Nadi

### System 4 — Human Design
Compare: Type, Authority, Profile, Definition, Cross, Variables, Defined Centers, Active Channels

### System 5 — Gene Keys
Compare: Full Hologenetic Profile (11 positions), Pearl Sequences, terminology

### System 6 — Numerology
Compare: Life Path, Expression, Soul Urge, Personality, Birthday, Chaldean name

### System 7 — (Future) Astrocartography
Compare: Planetary lines, ASC/MC/DSC/IC crossings

---

## Cross-Validation Sources

Authoritative sources for each system:

| System | Primary Source | Secondary Source |
|--------|----------------|------------------|
| Vedic Chart | Selemene engine | Kundli tool (Astrosage/Callisto) |
| Vimshottari | Kundli tool | Selemene engine |
| Panchanga | Drikpanchang.com | Selemene engine |
| Human Design | humdes.com | Bodygraph PNGs |
| Gene Keys | genekeys.com Golden Path | Bodygraph Hologenetic PNG |
| Numerology | Manual calculation | Selemene engine |

**Trust hierarchy:** Visual/screenshot > External authoritative API > Selemene engine (when verifying)

---

## Test Case Format

Each patch should produce a test case using the original reading's birth data:

```rust
#[test]
fn test_p1_1_vara_timezone_sahil() {
    let birth = BirthData {
        date: "1992-03-14",
        time: "02:22:00",
        timezone: "Asia/Kolkata", // UTC+5:30
        location: Location { lat: 12.9716, lon: 77.5946 },
    };
    
    let result = panchanga::calculate(&birth);
    
    // 14 March 1992 at 02:22 IST is Saturday (Shanivara), not Friday
    assert_eq!(result.vara_name, "Shanivara (Saturday)");
}
```

---

## Workflow Checklist

- [ ] Fetch Selemene API outputs for birth data
- [ ] Fetch humdes.com ravecart (if Human Design)
- [ ] Capture bodygraph screenshots
- [ ] Cross-validate each system
- [ ] Document findings in verification-report.md
- [ ] Extract patches to patch-list.md
- [ ] Add patches to PATCH-REGISTRY.md
- [ ] Mark reading as verified in registry
