# Twitter Thread: What ACTUALLY Matters for AI Code Generation

---

**Tweet 1 (Hook)**

I ran an empirical study on AI code generation documentation.

First result: .context/ folder beat CLAUDE.md 100% to 40%.

Then I caught a methodological flaw and re-ran it properly.

The real finding was surprising (thread)

---

**Tweet 2 (The Setup)**

The test: Give Claude a vague prompt like "Add task filtering" and see if it follows documented patterns.

The pattern to follow: Use `validateQuery` middleware.

I tested 10 trials per group. Control (single file) vs Treatment (modular files).

---

**Tweet 3 (First Result - Flawed)**

First study results:
- CLAUDE.md: 40% pattern adherence
- .context/: 100% pattern adherence

I was about to publish this as evidence that modular docs are superior.

But then someone asked a great question...

---

**Tweet 4 (The Question)**

"Did both groups have the same information?"

I checked. They didn't.

The treatment's validation.md showed:
```
validateBody(schema)
validateParams(schema)
validateQuery(schema)  // <-- THIS
```

The control only showed validateBody.

---

**Tweet 5 (The Real Test)**

So I added the validateQuery example to the control CLAUDE.md and re-ran.

Same model. Same prompt. Same 10 trials per group.

Only difference: organization (one file vs multiple files).

---

**Tweet 6 (Corrected Results)**

Results with identical information:
- CLAUDE.md: 100% pattern adherence
- .context/: 100% pattern adherence

No difference.

The original 60% gap wasn't about organization. It was about information completeness.

---

**Tweet 7 (The Real Insight)**

What actually matters for AI pattern adherence:

1. EXPLICIT code examples (not prose descriptions)
2. Show ALL variations (validateBody AND validateQuery)
3. Include anti-patterns ("don't do this")

Organization? Secondary.

---

**Tweet 8 (Why Modular Might Help)**

But wait - modular docs might still be valuable.

Not because of organization itself, but because:
- Dedicated files encourage thoroughness
- validation.md naturally covers ALL validation patterns
- A single CLAUDE.md might skip variations

---

**Tweet 9 (Practical Takeaway)**

Whether you use:
- Single CLAUDE.md
- Modular .context/ folder
- Both together

The key is: Show explicit code examples for EVERY pattern you want the AI to follow.

Don't expect inference. Be explicit.

---

**Tweet 10 (Science Wins)**

I almost published a misleading study.

The peer review process (even informal on Twitter) caught the flaw.

Science isn't about proving your hypothesis. It's about finding truth.

The truth: explicit examples > organization.

---

**Tweet 11 (Call to Action)**

Full test suite is open source.

github.com/[repo]/tests/

Replicate it. Challenge it. Improve it.

That's how we learn what actually works.

---

**Tweet 12 (End)**

TL;DR:

When AI documentation has identical information:
- Single file = modular files (no difference)

What matters:
- Explicit code examples
- All pattern variations shown
- Anti-patterns documented

Organization is secondary to completeness.

---
