# V2 Rollback Runbook

> **Note (2026-06-30):** `apps/noesis-web` has been retired from this repo.
> The production rollback paths below that reference `apps/noesis-web` are
> historical. The standalone HTML demos in this directory are still usable.

**Audience:** anyone needing to flip Nādashakti V2 off in production within 5 minutes.
**Trigger:** v2 audio path causing browser hangs, audible glitches across multiple users, sample CDN downtime, or a critical pitch-correctness regression caught by `verify-v2.mjs` after deploy.

---

## TL;DR

**One env-var flip disables every v2 surface across the app:**

```bash
NEXT_PUBLIC_RAAGA_V2_ENABLED=false
```

Redeploy. All callers of `RaagaPlayer.play()` fall back to v1 sine-wave just-intonation (the proven path verified end-to-end). No code changes required.

---

## What V2 Touches (Surface Inventory)

| Surface | Affected? | Rollback effect |
|---|---|---|
| [`RaagaPlayer.play()`](../Selemene-engine/apps/noesis-web/src/lib/raaga/RaagaPlayer.ts) `v2` opt-in | YES | Falls through to `playV1` (sine + just-intonation Hz, unchanged) |
| [`Nadabrahman.tsx`](../Selemene-engine/apps/noesis-web/src/components/engines/Nadabrahman.tsx) v2 toggle | YES | UI checkbox stays visible but defaults off; no audio change |
| [`melakarta_body_map.html`](melakarta_body_map.html) v2 toggle | YES (standalone) | UI checkbox stays visible; uncheck to use v1 path |
| [`strudel-demo-v2.html`](strudel-demo-v2.html) | DEMO ONLY | Standalone demo; not user-facing |
| Selemene Rust engine `engine-nadabrahman` | NO | Engine emits raga metadata only — never affected by audio rollback |
| `noesis-orchestrator` workflows | NO | Audio is purely consumer-side |

---

## Step-by-step

### A. Fastest path — env flip (production)

1. **Set the flag** in the noesis-web deployment environment:
   ```bash
   # Vercel / Railway / wherever noesis-web is hosted
   NEXT_PUBLIC_RAAGA_V2_ENABLED=false
   ```
2. **Trigger redeploy** (Vercel auto-redeploys on env change; Railway requires `railway up` or restart).
3. **Hard-refresh** any open noesis-web tabs (⌘⇧R) so the new env baked into the client bundle is loaded.
4. **Verify** via DevTools console on a Nadabrahman result page:
   ```js
   // Should now return false even when the v2 checkbox is ticked, because the
   // checkbox just sets the per-call override; the global remains off.
   window.localStorage.getItem('RAAGA_V2_ENABLED')  // should be null
   ```
5. **Click ▶** on a raga recommendation — should play sine-wave (v1) audio.

**Rollback time:** 2–5 minutes including deploy.

### B. Per-user override (during incident triage)

If a single user reports issues but others are fine, instruct them to:

```js
localStorage.setItem('RAAGA_V2_ENABLED', 'false');  // forces v1 in their browser only
location.reload();
```

This bypasses the global flag without affecting anyone else.

### C. Code-level kill switch (if env flip is not enough)

If a v2 codepath is *crashing the JS bundle* before the feature flag can be read, the killswitch is to short-circuit `playV2` in [`RaagaPlayer.ts`](../Selemene-engine/apps/noesis-web/src/lib/raaga/RaagaPlayer.ts):

```ts
private async playV2(m: Melakarta, opts: PlayOptions): Promise<void> {
  return this.playV1(m, opts);  // <-- TEMP HOTFIX: force v1 path unconditionally
}
```

Commit, redeploy. Time: 5–10 minutes.

### D. Sample CDN unreachable

If `loadRaagaSamples()` is throwing because every CDN in `FALLBACK_CDNS` is down:

1. Users will see `Audio error: loadRaagaSamples: all CDN candidates failed`.
2. Click ▶ still works for `timbre: 'sine'` (no sample load required).
3. Hotfix: tell users to deselect any non-sine timbre OR flip the global flag (Step A).
4. Add a new CDN to `FALLBACK_CDNS` in [`loader.ts`](../Selemene-engine/apps/noesis-web/src/lib/raaga/v2/samples/loader.ts) and redeploy.

---

## Verification After Rollback

Run all three gates to confirm v1 still works:

```bash
cd Selemene-engine/apps/noesis-web

# 1. v1 contract gate
node src/lib/raaga/verify.mjs
# Expected: ✅ All assertions passed (Mayamalavagaula, Sankarabharanam, etc.)

# 2. v2 contract gate (still passes — rollback is data-only, not code)
node src/lib/raaga/verify-v2.mjs
# Expected: ✅ V2 Phase-1 + Phase-2 contracts VERIFIED (52/52)

# 3. Typecheck
pnpm typecheck
# Expected: clean

# 4. Browser smoke test (open noesis-web, run a Nadabrahman workflow,
#    click ▶ on any recommendation; v1 sine-wave should play within ~2s)
```

---

## Common Failure Modes & Specific Triggers

| Failure | Detected by | Specific rollback |
|---|---|---|
| `verify-v2.mjs` red after deploy | CI gate | Step A — env flip |
| Browsers report `freq is not a function` | Sentry / user reports | Step C — force playV1 |
| Sample CDN 404 storms | Loader error rate spike | Step D — add CDN or env flip |
| OfflineAudioContext renders silent WAVs | QA report | Step A; investigate `render/offline.ts` separately |
| Pitch-tracker drift > ±5¢ on rendered audio | T-059 audit | Step A; gamaka renderer math review |
| AudioContext stuck `suspended` after tab blur | User reports | Already handled by visibility auto-resume; retry click |

---

## Post-Rollback Hygiene

1. Open a GitHub issue with the trigger evidence (logs, repro steps, affected user count).
2. Tag with `severity:rollback` and the failed wave label (`wave:2.1` etc.).
3. Schedule a fix forward — do **not** roll forward into production without re-running both gate scripts.
4. Update `V2_AUDIO_RICHNESS_PLAN.md` § 12 (Risks) with the new lesson learned.

---

## Recovery (rolling back the rollback)

When the underlying issue is fixed and verified:

1. Re-run `node src/lib/raaga/verify-v2.mjs` — must pass 52/52.
2. Smoke-test in a non-production environment with `NEXT_PUBLIC_RAAGA_V2_ENABLED=true`.
3. Stage rollout: 10% → 50% → 100% via a percentage-based env split or feature-flag service.
4. Watch the same metrics that triggered rollback for at least 1h at each stage.

---

🎵 *"The rāga can wait. The body is the priority."*
