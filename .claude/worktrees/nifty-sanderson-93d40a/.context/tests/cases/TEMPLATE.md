# Test Case Template

## Metadata
- **ID**: XX-test-name
- **Category**: simple | cross-domain | ambiguous | security-critical | refactoring
- **Complexity**: low | medium | high
- **Domains Tested**: auth, api, database, patterns, validation, security

## Prompt

The exact prompt given to the AI model.

## Expected Behaviors

What the correct output should include:

1. [ ] Specific behavior 1
2. [ ] Specific behavior 2
3. [ ] ...

## Scoring Criteria

### Pattern Adherence (0-10)
- 10: All patterns followed exactly
- 7-9: Minor deviations
- 4-6: Some patterns missed
- 0-3: Major pattern violations

### Completeness (0-10)
- 10: All requirements met
- 7-9: Most requirements, minor omissions
- 4-6: Missing significant elements
- 0-3: Incomplete implementation

### Correctness (0-10)
- 10: No bugs, handles all cases
- 7-9: Minor bugs or edge cases
- 4-6: Logic errors present
- 0-3: Fundamentally broken

### Security (0-10)
- 10: Follows all security guidelines
- 7-9: Minor security concerns
- 4-6: Some security issues
- 0-3: Critical security flaws

### Consistency (0-10)
- 10: Matches existing code style perfectly
- 7-9: Minor style differences
- 4-6: Inconsistent with codebase
- 0-3: Completely different style

## Red Flags (Automatic Deductions)

Things that should result in point deductions:
- Uses bcrypt instead of Argon2id: -5 security
- Returns password hash: -10 security
- Missing validation: -3 completeness
- Wrong error format: -2 pattern adherence
