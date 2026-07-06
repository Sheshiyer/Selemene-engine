# Dodo Payments — Incident Runbook Tabletop Exercise

> **Purpose:** validate that the incident runbook actually works under
> simulated stress, before a real incident hits.
> **Duration:** ~45 minutes total (4 scenarios × ~10 min each + 5 min debrief).
> **Cadence:** before merging PR #683 + once per quarter thereafter.
> **Cost:** zero (runs against staging in test mode).

---

## Roles

You need **two people**. Swap roles between scenarios so both get reps in
both seats.

- **Driver** — has the keyboard. Follows `runbooks/dodo-incident.md` step
  by step. Says aloud what they're typing and what they expect.
- **Caller** — knows the scenario, plays the "incident". Watches over
  the driver's shoulder for skipped steps. Times each scenario.

Caller has this doc open. Driver has only the incident runbook open
(simulating real on-call conditions).

---

## Setup (5 min, before the first scenario)

1. Both terminals connected to **staging**, not prod.
2. Staging Postgres has at least 3 test subscriptions (run the seed
   below if needed).
3. `dodo` CLI logged in to test mode.
4. Caller opens this doc; driver closes it.

```sql
-- Seed 3 test subs into staging if missing
INSERT INTO users (id, email, password_hash, full_name, tier,
                   consciousness_level, experience_points)
SELECT gen_random_uuid(),
       'tabletop-' || gs::text || '@selemene.test',
       'placeholder', 'Tabletop ' || gs, 'basic', 0, 0
FROM generate_series(1, 3) gs
ON CONFLICT DO NOTHING;
```

---

## Scenario 1 — Webhook outage (signatures rejected)

**Caller setup (don't tell the driver):**
Edit `DODO_PAYMENTS_WEBHOOK_KEY` on staging to a wrong value
(`whsec_INVALID`). Restart admin-web staging.

**Caller says:**
> "Sentry just paged. We're getting bursts of `signature verification
> failed` from admin-web. New Dodo events haven't landed in
> `processed_webhook_events` for 17 minutes. Customer support has 2
> tickets about 'I paid but I'm still on Free.' Go."

**Start timer.** Driver opens `runbooks/dodo-incident.md` and follows
Scenario 1.

**Pass criteria:**
- Driver runs the diagnostic SQL within 5 min
- Driver identifies "signing key drift" as likely cause
- Driver re-fetches the signing key via the curl in the runbook
- Driver updates env + restarts admin-web
- Test event from dashboard reaches Rust within 15 min total

**Caller cleanup:** revert env to the real signing key, confirm test
event lands.

---

## Scenario 2 — Postgres drift (legacy webhook missed)

**Caller setup:**
Pick one of the 3 staging test subs. In the Dodo dashboard, cancel that
subscription. Then edit Postgres directly to **revert** the
cancellation locally (so DB and Dodo disagree):

```sql
UPDATE billing_subscriptions
SET status = 'active', canceled_at = NULL, updated_at = NOW()
WHERE provider_subscription_id = '<sub-id-you-cancelled>';
```

This simulates a missed `subscription.cancelled` webhook during an
earlier outage.

**Caller says:**
> "User john@example.com just emailed support: 'I cancelled in your
> portal a week ago, but you charged me again today.' Go figure out
> what's wrong and fix it."

**Pass criteria:**
- Driver runs `dodo_reconcile` in read-only mode within 5 min
- Driver spots the row in `samples.local_only`
- Driver chooses one remediation path (redeliver from Dodo OR
  `DODO_RECONCILE_FORCE_CANCEL=true` OR manual SQL)
- Driver verifies tier dropped to `free` after fix

**Caller cleanup:** none — the fix is the cleanup.

---

## Scenario 3 — Signing key rotation drill

**Caller says:**
> "Compliance flagged that we haven't rotated the Dodo webhook signing
> key in 90 days. Rotate it now without dropping any in-flight events."

**Pass criteria:**
- Driver finds the "Roll signing key" button in Dodo dashboard
- Driver updates env on admin-web
- Driver restarts admin-web (NOT noesis-api — Rust doesn't read this
  key)
- Driver verifies a new test event delivers ok
- **Bonus:** driver thinks to check whether any in-flight retries from
  before the rotation are now failing (they will — Dodo retries those
  with the OLD signature). Acceptable answer: "those will go to dead
  letter; we'll redeliver from the dashboard logs after rotation
  settles."

---

## Scenario 4 — Extended outage recovery

**Caller setup:**
On staging, stop admin-web entirely for 5 minutes. During the
window, fire 3 test events from the Dodo dashboard (different event
types). Restart admin-web.

**Caller says:**
> "admin-web was down for 5 minutes. We just brought it back up.
> Tell me what events we missed and how to recover them."

**Pass criteria:**
- Driver runs `dodo_reconcile` to quantify drift
- Driver opens Dodo dashboard webhook logs and identifies the failed
  deliveries
- Driver clicks "Resend" on each (or knows the API equivalent)
- Driver re-runs `dodo_reconcile` and confirms drift returned to zero

---

## Debrief (5 min)

After all 4 scenarios, both people answer:

1. **Which step in the runbook was unclear or wrong?**
   File a PR to fix.
2. **Which step was missing entirely?**
   Add it to the runbook.
3. **What command did you have to look up?**
   Add the exact command to the runbook.
4. **Where would Sentry / Grafana have helped diagnose faster?**
   Note for next iteration of `monitoring/grafana/dashboards/dodo-billing.json`.

Commit any runbook updates with a `docs(runbooks):` prefix.

---

## Sign-off

| Date | Driver | Caller | All 4 scenarios passed? | Runbook updates committed? |
|---|---|---|---|---|
| | | | | |
| | | | | |

This sign-off is the gate to flip from **read-only** to
`DODO_RECONCILE_FORCE_CANCEL=true` in the cron. Don't enable the auto-
correction path until the runbook has been exercised at least once.
