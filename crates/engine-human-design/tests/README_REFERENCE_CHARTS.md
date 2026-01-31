# Human Design Reference Charts Dataset

## Overview
This dataset contains 16 reference charts for validating the Human Design engine accuracy. Charts are generated from the Selemene HD Engine and represent synthetic reference data for internal consistency validation.

**Status**: ⚠️ Synthetic data - validated for internal consistency, pending professional HD software verification.

## Coverage Summary

### Types (3/5)
- ✅ Generator (6 charts)
- ✅ Manifesting Generator (5 charts)
- ✅ Projector (5 charts)
- ❌ Manifestor (not found in search space)
- ❌ Reflector (not found in search space)

### Authorities (2/8)
- ✅ Sacral Authority (11 charts)
- ✅ G-Center Authority (5 charts)
- ❌ Emotional Authority (implementation pending)
- ❌ Splenic Authority (implementation pending)
- ❌ Heart Authority (implementation pending)
- ❌ Mental Authority (implementation pending)
- ❌ Lunar Authority (implementation pending)

### Profiles (11 different)
All charts have diverse profiles covering:
- 1/3 (Investigator/Martyr)
- 2/4 (Hermit/Opportunist)
- 3/5 (Martyr/Heretic)
- 3/6 (Martyr/Role Model)
- 4/1 (Opportunist/Investigator)
- 4/6 (Opportunist/Role Model)
- 5/1 (Heretic/Investigator)
- 5/2 (Heretic/Hermit)
- 6/2 (Role Model/Hermit)
- 6/3 (Role Model/Martyr)
- 6/6 (Role Model/Role Model)

### Definition Types
- Single Definition: Present (via single/few channels)
- Split Definition: Present (via channel patterns)
- Triple Split: Not explicitly covered
- No Definition (Reflector): Not present

## Data Fields

Each chart includes:

### Birth Information
- `birth_date`: YYYY-MM-DD format
- `birth_time`: HH:MM:SS in 24-hour UTC
- `timezone`: Always "UTC" for consistency
- `latitude`: 0.0 (arbitrary reference)
- `longitude`: 0.0 (arbitrary reference)

### Expected Activations
- `personality_sun`: {gate, line}
- `personality_earth`: {gate, line}
- `design_sun`: {gate, line}
- `design_earth`: {gate, line}

### Expected Analysis Results
- `type`: Generator/ManifestingGenerator/Projector
- `authority`: Sacral/GCenter
- `profile`: "line/line" format (e.g., "1/3", "5/2")
- `defined_centers`: Array of center names (e.g., ["Root", "Sacral", "G", "Throat"])
- `active_channels`: Array of channel descriptors (e.g., ["3-60", "9-52", "7-31"])

## Usage in Tests

```rust
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct ReferenceDataset {
    charts: Vec<ReferenceChart>,
    metadata: Metadata,
}

#[test]
fn test_against_reference_charts() {
    let data: ReferenceDataset = serde_json::from_str(
        &fs::read_to_string("tests/reference_charts.json").unwrap()
    ).unwrap();
    
    for chart_ref in data.charts {
        let birth_time = /* parse chart_ref.birth_date + chart_ref.birth_time */;
        let result = generate_hd_chart(birth_time, "");
        
        // Validate against expected values
        assert_eq!(result.hd_type, chart_ref.expected.type);
        assert_eq!(result.authority, chart_ref.expected.authority);
        // ... more assertions
    }
}
```

## Known Limitations

1. **Missing Types**: No Manifestor or Reflector charts due to current implementation limitations
2. **Limited Authorities**: Only Sacral and G-Center - other authorities require implementation
3. **Synthetic Data**: Charts generated from internal engine, not professionally verified
4. **UTC Only**: All times in UTC with arbitrary lat/lon - real charts would use actual locations
5. **Limited Channel Diversity**: Most charts have 1-3 channels; rare to find complex definitions

## Future Improvements

### Phase 1: Complete Implementation
- [ ] Implement remaining Type/Authority determination logic
- [ ] Add Emotional, Splenic, Heart, Mental, Lunar authority detection
- [ ] Add Manifestor type detection
- [ ] Enable Reflector charts (all centers undefined)

### Phase 2: Professional Validation
- [ ] Obtain 10+ charts from professional HD software (Jovian Archive, Genetic Matrix)
- [ ] Cross-validate synthetic charts against professional tools
- [ ] Document discrepancies and adjust engine logic
- [ ] Add professionally-verified charts to dataset

### Phase 3: Extended Coverage
- [ ] Add all 64 gate activations (currently focused on Sun/Earth)
- [ ] Add 13 planetary activations per chart
- [ ] Cover rare edge cases (complex definitions, multiple splits)
- [ ] Add charts with specific channel combinations

## Generation Process

Charts were generated using:
```bash
cargo run --release --example generate_final_reference > tests/reference_charts.json
```

The generation script:
1. Searches 8000+ date/time combinations (1970-2005)
2. Filters for charts with defined channels (excludes empty Reflectors)
3. Selects diverse examples covering different Types, Authorities, Profiles
4. Records Sun/Earth activations and analysis results
5. Outputs JSON with metadata

## Validation Status

✅ JSON format validated  
✅ Internal consistency verified (type matches defined centers)  
✅ Profile lines in valid range (1-6)  
✅ Channel gate numbers valid (1-64)  
⚠️ Professional HD software validation pending  
⚠️ Earth calculation accuracy pending fix (minor issue noted in Sprint 3)  

## Contact & Updates

This dataset will be updated as:
- More Type/Authority implementations are completed
- Professional verification is obtained
- Additional edge cases are identified
- Engine accuracy improves

Last Updated: 2026-01-31  
Generated By: Selemene HD Engine v0.1.0  
Sprint: W1-S4 (Phase 2 - Human Design Engine - Validation)
