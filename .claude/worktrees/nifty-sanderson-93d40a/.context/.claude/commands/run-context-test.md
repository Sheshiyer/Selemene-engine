You are running an empirical test comparing .context documentation vs single CLAUDE.md.

## Test Configuration

**Test Case**: $ARGUMENTS (e.g., "01-simple-endpoint control" or "02-auth-middleware treatment")

Parse the arguments to get:
- `case_id`: The test case (e.g., 01-simple-endpoint)
- `group`: Either "control" or "treatment"

## Instructions

1. **Load the test case** from `tests/cases/{case_id}.md` - extract the prompt

2. **Load the documentation** based on group:
   - If `control`: Read `tests/fixtures/control/CLAUDE.md`
   - If `treatment`: Read all files from `tests/fixtures/treatment/.context/`

3. **Load the project codebase** from `tests/fixtures/project/src/`

4. **Generate code** following the loaded documentation to complete the prompt task

5. **Save your output** to `tests/outputs/{case_id}/{group}/run-{timestamp}.md`

Include in your output:
- The generated code
- Brief explanation of your approach
- Which documentation sections you referenced

## Important

- Follow ONLY the patterns from the loaded documentation
- Do NOT use knowledge outside what was provided in the documentation
- Pretend you've never seen this codebase before
