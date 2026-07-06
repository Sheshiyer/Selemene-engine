# Nādashakti V2 — Usage Guide

> **Note (2026-06-30):** `apps/noesis-web` has been retired from this repo.
> The `@/lib/raaga` paths below are historical. The standalone HTML demos in
> this directory remain runnable; the production renderer will be ported to
> [Sankalpa](../../sankalpa/).

**For:** developers consuming `@/lib/raaga` in `noesis-web`, or anyone embedding the standalone HTML demos.
**Companion:** [`RAAGA_ENGINE.md`](./RAAGA_ENGINE.md), [`SHRUTI_THEORY.md`](./SHRUTI_THEORY.md), [`V2_AUDIO_RICHNESS_PLAN.md`](./V2_AUDIO_RICHNESS_PLAN.md)
**Verifier:** `node src/lib/raaga/verify-v2.mjs` — 52 contract assertions, must stay green.

---

## 30-Second Quickstart

```ts
import { getRaagaPlayer } from "@/lib/raaga";

const player = getRaagaPlayer();

// V1 (default — pure sine, exact 22-shruti just-intonation)
await player.play(15);  // Mayamalavagaula

// V2 (opt-in — sitar timbre, kampita on every swara, adi tala, ujjayi breath)
await player.play(15, {
  v2: true,
  timbre: "sitar",
  defaultGamaka: { kind: "kampita" },
  tala: "adi",
  breath: "ujjayi",
});

// Stop
await player.stop();

// Download as WAV
const url = await player.renderWav(15, { v2: true, timbre: "sitar" });
// → blob URL, attach to <a href={url} download="..."> for save
```

---

## API Reference

### `getRaagaPlayer(): RaagaPlayer`

Singleton accessor. Always returns the same `RaagaPlayer` instance per page.

### `player.play(melakartaNum: number, opts?: PlayOptions): Promise<void>`

Plays melakarta `1..72`. First call lazy-initializes Strudel + WebAudio (~2-3s on first user gesture). Subsequent calls are instant.

**`PlayOptions`:**

| Option | Type | Default | Notes |
|---|---|---|---|
| `rootHz` | `number` | `220` | Hz for Sa. Use 261.63 (C4) for piano-keyboard mental mapping; 220 (A3) is a comfortable male tonic. |
| `cps` | `number` | `0.5` | Strudel cycles per second. Overridden by `breath` if set. |
| `sound` | `string` | `'sine'` | Strudel `.s()` sound name. V1 path only. |
| `direction` | `'arohana' \| 'avarohana' \| 'both'` | `'both'` | Ascend, descend, or both stitched. |
| `v2` | `boolean` | `undefined` | Force v2 path. If unset, falls back to global flag. |
| `timbre` | `'sine' \| 'sitar' \| 'tanpura' \| 'mridangam' \| 'bansuri' \| 'sarangi'` | `'sitar'` (when v2) | Sample bank. Loads from CDN with failover. |
| `gamakas` | `GamakaAnnotation[]` | `[]` | Per-swara ornament overrides. See below. |
| `defaultGamaka` | `Gamaka` | `{ kind: 'none' }` | Applied to every swara when no per-swara annotation matches. |
| `tala` | `'adi' \| 'rupakam' \| 'misra-chapu' \| 'khanda-chapu' \| 'tisra-eka' \| 'jhampa'` | `undefined` | Rhythmic accent layer. Samam +3.5dB, edam +1.5dB. |
| `breath` | `'box-4' \| 'calming-4-7-8' \| 'bhastrika' \| 'ujjayi' \| 'brahmari' \| 'shitali' \| ...` | `undefined` | Sets cps + ADSR from breath cycle (overrides explicit `cps`). |
| `tanpura` | `boolean` | `false` | Layer Pa-Sa-Sa-Sa drone underneath. |

### `player.stop(): Promise<void>`

Stops all currently-scheduled patterns. Non-blocking; safe to call even before first play.

### `player.renderWav(melakartaNum, opts?): Promise<string \| null>`

Renders the same composition offline via `OfflineAudioContext` and returns a Blob URL pointing at a 16-bit / 48kHz / stereo WAV. Returns `null` on browsers without `OfflineAudioContext` (very rare; covers IE / old Safari).

```ts
const url = await player.renderWav(15, { v2: true, timbre: "sitar" });
const a = document.createElement("a");
a.href = url;
a.download = "Mayamalavagaula-sitar.wav";
a.click();
URL.revokeObjectURL(url);
```

---

## Gamakas

Five canonical Carnatic ornaments, each parameterized:

```ts
import type { Gamaka, GamakaAnnotation } from "@/lib/raaga";
import { V2 } from "@/lib/raaga";

// Light vibrato around the swara
const kampita: Gamaka = { kind: "kampita", cents: 20, rateHz: 6 };

// Slow sway between swara and a neighbor
const andolana: Gamaka = { kind: "andolana", cents: 50, rateHz: 1.5 };

// Curved slide into the swara from a neighbor
const kurula: Gamaka = { kind: "kurula", cents: 100, glideFraction: 0.4 };

// Grace note above target, briefly held then resolved
const nokku: Gamaka = { kind: "nokku", graceCents: 100, graceFraction: 0.08 };

// Bent rising attack — sitar-like mizrab
const sphurita: Gamaka = { kind: "sphurita", startCents: -60, attackFraction: 0.1 };
```

### Per-swara annotation

```ts
const mayaGamakas: GamakaAnnotation[] = [
  { swaraIndex: 1, direction: "both", gamaka: { kind: "kampita", cents: 15, rateHz: 5 } },  // Re1
  { swaraIndex: 2, direction: "up",   gamaka: { kind: "sphurita", startCents: -50, attackFraction: 0.12 } },  // Ga3 ascent
  { swaraIndex: 3, direction: "both", gamaka: { kind: "andolana", cents: 40, rateHz: 1.8 } },  // Ma1
  // Sa (0), Pa (4), Sa' (7) left plain — achala
];

await player.play(15, {
  v2: true,
  timbre: "sitar",
  gamakas: mayaGamakas,
});
```

Indices 0..7 map to Sa, R, G, M, Pa, D, N, Sa' for any melakarta. Direction:
- `'up'` — only when ascending (arohana phase)
- `'down'` — only when descending (avarohana phase)
- `'both'` — both phases

The traditional Mayamalavagaula preset is shipped pre-baked at [`presets/mayamalavagaula.ts`](../Selemene-engine/apps/noesis-web/src/lib/raaga/v2/presets/mayamalavagaula.ts):

```ts
import { V2 } from "@/lib/raaga";
await player.play(15, { v2: true, timbre: "sitar", gamakas: V2.MAYAMALAVAGAULA_GAMAKAS });
```

---

## Talas

```ts
import { V2 } from "@/lib/raaga";

V2.TALAS.adi          // { beats: 8, structure: [4,2,2], accentBeats: [0,4,6] }
V2.TALAS["misra-chapu"]  // 7 beats: 3+4
V2.TALAS.jhampa       // 10 beats: 7+1+2
```

The accent layer adds a `gain()` chain on top of the freq() pattern. Samam (downbeat) is +3.5dB, edam (anti-samam) is +1.5dB.

---

## Breaths

```ts
import { V2 } from "@/lib/raaga";

V2.BREATHS.bhastrika   // 0.5-0-0.5-0 → cps≈1.0, very fast staccato
V2.BREATHS.brahmari    // 4-0-12-0   → cps≈0.03, very slow sustain
V2.BREATHS.ujjayi      // 4-0-6-0    → moderate cps, ocean-breath ADSR

V2.breathForChakra(8)  // → BREATHS["heart-coherence-5-5"] (chakra 8 = Vasu / Heart)
```

Setting `breath` overrides any explicit `cps`. The breath cycle determines the swara hold rhythm + envelope shape.

---

## Compose without playing

Useful for previews, server-side preprocessing, or unit tests:

```ts
import { V2, MELAKARTAS } from "@/lib/raaga";

const m = MELAKARTAS[14];  // #15 Mayamalavagaula
const out = V2.compose({
  melakarta: m,
  rootHz: 220,
  timbre: "sitar",
  defaultGamaka: { kind: "kampita" },
  tala: "adi",
  breath: "ujjayi",
});

console.log(out.ragaCode);  // Strudel eval string
console.log(out.tanpuraCode);  // null unless tanpura: true
console.log(out.durationSeconds);
console.log(out.meta.appliedGamakas);  // ['none', 'kampita', 'kampita', ..., 'none']
```

---

## Standalone HTML demos

Three test pages live in `raagaegnin/`:

| Page | Purpose |
|---|---|
| [`strudel-demo.html`](./strudel-demo.html) | V1 baseline — sine + just-intonation only. Regression reference. |
| [`strudel-demo-v2.html`](./strudel-demo-v2.html) | V2 playground — 17 ragas × 4 timbres × 6 gamakas × 6 talas × 6 breaths + 6 ritual presets + WAV download. |
| [`melakarta_body_map.html`](./melakarta_body_map.html) | Mahapurusha body visualization. V2 toggle in header reveals the same control bar. |

Serve any of them with:

```bash
cd raagaegnin
python3 -m http.server 8765
# open http://localhost:8765/strudel-demo-v2.html
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| First click does nothing | Strudel CDN still loading | Wait ~2s; status pill shows "🎵 Audio ready" when ready |
| `Audio error: window.evaluate is not a function` | CDN script blocked | Disable adblockers for `unpkg.com` |
| Plays in V1 silently | Browser tab muted | Click the tab's mute icon |
| Tab blur → silence on return | AudioContext suspended | Already auto-resumed via `visibilitychange` listener; click ▶ again |
| Sample timbres play but sound like sine | `samples()` failed silently | Check console for `loadRaagaSamples: all CDN candidates failed`; see [V2_ROLLBACK.md](./V2_ROLLBACK.md) §D |
| `Mayamalavagaula sounds identical to Kanakangi` | 12-TET fallback active (CRITICAL) | Run `node src/lib/raaga/verify.mjs` — should show 92¢ Δ. If 0¢, the freq path is broken; rollback. |
| WAV download missing some swaras | OfflineAudioContext duration miscalculation | File issue with raga number + opts; renderer is deterministic so reproducible |

---

## Pinned Versions

- `@strudel/web@1.3.0` — pinned in [package.json](../Selemene-engine/apps/noesis-web/package.json). Do **not** auto-bump; verify the new version's globals match `(initStrudel, evaluate, freq, s, hush)` before changing.
- Sample manifest format `0.1.0` — schema at [`v2/samples/manifest.schema.json`](../Selemene-engine/apps/noesis-web/src/lib/raaga/v2/samples/manifest.schema.json).

---

🎵 *"From shruti to gamaka to rasa — sound becomes feeling becomes body."*
