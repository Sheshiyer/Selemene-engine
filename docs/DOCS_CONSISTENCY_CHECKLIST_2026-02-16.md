# Docs Consistency Checklist (2026-02-16)

Reusable pass/fail checklist for messaging and link consistency across markdown docs.

## 1) Philosophy and tone

- [ ] Intro uses reflection-first framing where user-facing.
- [ ] Avoids prescriptive authority framing in narrative copy.
- [ ] Uses preferred terms where relevant: `authorship`, `reflection`, `inquiry`, `witness`, `synthesis`, `mirrors`.

## 2) Vocabulary guardrails

- [ ] No disallowed terms in user-facing intros unless technically required: `journey`, `manifesting`, `vibration`.
- [ ] If imperative wording appears, it is technical/operational (commands, auth requirements, runbook steps), not product-positioning language.

## 3) Link policy

- [ ] API runtime examples use `https://selemene.tryambakam.space`.
- [ ] Parent field context links use `https://tryambakam.space`.
- [ ] Somatic callouts use `https://1319.tryambakam.space` only in relevant embodiment contexts.

## 4) Technical integrity

- [ ] Commands, endpoints, and payload examples remain valid.
- [ ] No changes to technical semantics while editing framing text.
- [ ] Markdown renders with no broken images/links in edited sections.

## 5) Visual usage (if images are used)

- [ ] At most one visual block per section intro for readability.
- [ ] Image path resolves relative to file location.
- [ ] Visual supports the section meaning (not decorative bloat).

## File-by-file pass template

Use this template during QA runs:

```
File: <path>
Tone: PASS/FAIL
Vocabulary: PASS/FAIL
Link policy: PASS/FAIL
Technical integrity: PASS/FAIL
Visual usage: PASS/FAIL/N/A
Notes: <optional exceptions>
```
