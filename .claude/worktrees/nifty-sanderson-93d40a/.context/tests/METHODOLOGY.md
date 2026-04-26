# Test Methodology

This document explains the scientific methodology used to empirically compare structured `.context/` documentation against a single `CLAUDE.md` file.

## Research Question

**Does structured, modular documentation produce higher quality AI-generated code compared to equivalent information in a single file?**

## Hypothesis

**H1**: AI models produce code that better adheres to documented patterns when documentation is structured into domain-specific modules rather than consolidated into a single file.

**Rationale**:
- Modular documentation may improve context retrieval accuracy
- Structured format may reduce cognitive load on the model
- Domain separation may improve pattern recognition

## Experimental Design

### Independent Variable
Documentation structure:
- **Control**: Single `CLAUDE.md` file containing all documentation
- **Treatment**: Structured `.context/` folder with domain-specific files

### Dependent Variables
Quality scores across five dimensions:
1. **Pattern Adherence**: Following documented conventions
2. **Completeness**: Meeting all requirements
3. **Correctness**: Bug-free implementation
4. **Security**: Following security guidelines
5. **Consistency**: Matching existing code style

### Controlled Variables
- Same underlying information content
- Same synthetic project codebase
- Same LLM model and parameters (temperature=0)
- Same prompts/test cases
- Same scoring rubrics

## Information Parity

Critical to this experiment: both groups receive **semantically equivalent information**. The control `CLAUDE.md` contains the same facts, patterns, and guidelines as the treatment `.context/` folder, just in a different structure.

Differences:
- Control: ~2000 lines in one file, linear organization
- Treatment: Same content split across ~12 files, hierarchical organization

## Test Case Selection

Test cases span multiple dimensions:

| Category | Purpose | Example |
|----------|---------|---------|
| Simple | Baseline single-domain tasks | Add a GET endpoint |
| Security-critical | Tests security guideline adherence | Password hashing |
| Cross-domain | Tests multi-file context retrieval | Feature spanning auth + API + DB |
| Ambiguous | Tests pattern inference | Vague requirements |
| Refactoring | Tests anti-pattern recognition | Fix code smells |

## Statistical Approach

### Sample Size
- 10 runs per test case per group
- Accounts for LLM output variance at temperature=0 (which still exists)
- Total: 6 test cases × 2 groups × 10 runs = 120 API calls

### Analysis Methods

**Mann-Whitney U Test**
- Non-parametric test for comparing two groups
- Does not assume normal distribution
- Appropriate for ordinal score data

**Cohen's d (Effect Size)**
- Measures practical significance
- d < 0.2: negligible
- d = 0.2-0.5: small
- d = 0.5-0.8: medium
- d > 0.8: large

### Significance Threshold
- α = 0.05 (p < 0.05 for statistical significance)

## Scoring Process

### Automated Scoring (Default)
- LLM-based evaluation using scoring rubrics
- Provides consistency across large number of outputs
- Rubrics are explicit and deterministic

### Manual Scoring (Validation)
- Human evaluation for subset of outputs
- Validates automated scoring accuracy
- Blind evaluation (scorer doesn't know group)

### Blind Evaluation
When possible, evaluators should not know whether output came from control or treatment group. The scoring script supports this by:
1. Randomizing order of outputs
2. Removing group identifiers from scorer view
3. Only revealing groups after all scoring complete

## Validity Considerations

### Internal Validity
- **Randomization**: Run order randomized
- **Blinding**: Scorer doesn't see group assignment
- **Consistency**: Same rubrics for both groups
- **Replication**: Multiple runs account for variance

### External Validity
- Synthetic project designed to mirror real-world patterns
- Test cases cover common development tasks
- Results may vary with different:
  - Programming languages
  - Project complexity
  - Documentation content quality

### Construct Validity
- Scoring dimensions map to real quality concerns
- Rubrics derived from industry best practices
- Anti-patterns based on common AI code generation issues

## Limitations

1. **Model-specific**: Results may not generalize to other LLMs
2. **Domain-specific**: TypeScript/Express patterns may differ from other stacks
3. **Content quality**: Results depend on documentation quality, not just structure
4. **Synthetic project**: Real projects have more complexity and history

## Reproducing Results

```bash
# Install dependencies
cd tests
npm install

# Run full test suite
npm run test:context-comparison

# Or run specific test
npm run test:single -- --case=01-simple-endpoint --group=control

# Score outputs
npm run test:score -- --case=01-simple-endpoint --group=control --mode=auto

# Generate analysis
npm run test:analyze -- --output=results/report.md
```

## Expected Outcomes

If H1 is supported, we expect:
- Treatment group scores higher on pattern adherence
- Larger effect sizes on security-critical tests
- Ambiguous prompts show greatest difference
- Simple tasks show smallest difference (ceiling effect)

If H1 is not supported:
- No significant difference between groups
- Information content matters more than structure
- May indicate model handles both formats equally well

## Ethical Considerations

- No human subjects involved
- API usage follows terms of service
- Results shared openly for community benefit
- No cherry-picking of results
