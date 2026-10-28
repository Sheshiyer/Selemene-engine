# Cloud task prompt template

Paste this as the first message of a new cloud session in the `thoughtseed`
environment, with the issue's repository selected. Fill in the three
placeholders; leave the rest as written.

```text
Work GitHub issue <ISSUE_URL> in <REPO>.

Before coding:
- Read the issue, its parent wave issue, and docs/cloud/README.md (in
  Sheshiyer/Selemene-engine) for the environment's verification commands and
  known caveats.
- Confirm the issue is not already satisfied on main. If it is, comment the
  evidence (paths, test names) on the issue and stop without opening a PR.

Scope:
- Only what the issue's acceptance or exit criteria ask for. No drive-by
  refactors, no formatting of untouched files.
- Never change production config, deploy, rotate keys, or run anything against
  selemene.tryambakam.space or app.urania.tryambakam.space.
- Billing is deferred: do not add Dodo, checkout, pricing or upgrade paths.
- Birth data and reading bodies never go into fixtures, logs, issues or PRs.

Verification (the repo has no CI; these local checks are the gate):
- Run the repo's checks for every crate or package you touched, as listed in
  docs/cloud/README.md, and include the exact commands and results in the PR.
- For a bug fix, show the failing test first, then the passing run.
- <EXTRA_VERIFICATION, or delete this line>

Deliver:
- One draft PR on your session branch that links the issue ("Refs #N"; use
  "Closes #N" only if every criterion is met), with a criteria checklist
  ticked against evidence.
- If a criterion needs production access, secrets or a human decision, say
  so in the PR and leave it unticked.
```

## Picking the next task

Open issues labelled `cloud-ready` in Sheshiyer/Selemene-engine are the
queue, in the order given in `docs/cloud/README.md`. Launch one session per
issue. Independent issues can run in parallel; issues that touch the same
engine crate should run one after another.
