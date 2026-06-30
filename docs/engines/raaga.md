# Raaga — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Carnatic **melakarta** sound-therapy: takes a melakarta number/name (or auto-selects by dosha × time-of-day) and returns its just-intonation swaras, shruti indices, chakra/dosha context, prahar timing, and Strudel-compatible ratio arrays so the web `RaagaPlayer` can render the raga as sound. Distinct from NadaBrahman (which *recommends* a raga); Raaga gives a chosen melakarta its full musical + therapeutic profile (`engine.ts:1-10`).

## 1. Identity
| | |
|---|---|
| `engine_id` | `raaga` (confirmed: `ts-engines/src/engines/raaga/engine.ts:9,29,202`; `crates/noesis-bridge/src/lib.rs:303,726,816`; renderer route `apps/noesis-web/app/engines/page.tsx:247`) — **lowercase, not kebab** |
| Crate | **— (no Rust engine; TypeScript engine)** |
| Runtime source | **`ts-engines/src/engines/raaga/engine.ts`** (`RaagaEngine`, TS) — served via the Bun TS-engines server (`http://localhost:3001`, `crates/noesis-bridge/src/lib.rs:42`) behind `crates/noesis-bridge/src/lib.rs:297-303` (`BridgeEngine::raaga`) → registered in the bridge manager at `lib.rs:542` |
| Data | `ts-engines/src/engines/raaga/wisdom.ts` — all **72 melakartas generated algorithmically** (Venkatamakhin/Katapayadi, `MELAKARTAS`, `wisdom.ts:180-207`) over a 22-shruti just-intonation table (`SHRUTIS`, `wisdom.ts:19-43`); dosha + prahar tables (`wisdom.ts:231-264`) |
| Witness prompts | `ts-engines/src/engines/raaga/witness.ts` (`generateWitnessPrompts`) |
| Renderer | `apps/noesis-web/src/components/engines/Raaga.tsx` (routed as `RaagaView`, `apps/noesis-web/app/engines/page.tsx:247-248`, `apps/noesis-web/app/readings/[id]/page.tsx:52`) |
| Web data libs | `apps/noesis-web/src/lib/raaga/` (`RaagaPlayer`, `MELAKARTAS`, `SHRUTIS` — mirror of the backend data; `index.ts:1-3`); V2 audio (`lib/raaga/v2/`) |
| OpenAPI stub | **— (not in `EngineResultData` enum; no `*ResultSchema`)** — confirmed absent from all of `crates/noesis-core/` |
| Fixture (real values) | none found |

## 2. Output schema

The bridge passes the TS result through **verbatim** — `result: ts_response.result` (`noesis-bridge/src/lib.rs:441`), `witness_prompts[]` → envelope `witness_prompt` (first only, `lib.rs:442-447`), `consciousness_level` = the engine's `required_phase` = **0** (`lib.rs:448`; `engine.ts:37`). So the renderer receives the producer's exact `result` shape, un-reshaped.

**Runtime JSON — what the TS producer emits (`engine.ts:178-199`):**
```jsonc
{
  "melakarta": { "num": 15, "name": "Mayamalavagaula", "chakra": 3, "ma_type": "shuddha" }, // ma_type ∈ "shuddha"|"prati"
  "swaras": [                                  // 8 entries, Sa Re Ga Ma Pa Dha Ni Sa' (engine.ts:143-155)
    { "swara": "Sa", "shruti_index": 0, "ratio_num": 1, "ratio_den": 1, "ratio_decimal": 1.0, "hz": 220.0 }
    // …7 more; hz = root_hz · ratio_decimal
  ],
  "strudel_ratios": [1.0, 1.066…, 1.25, 1.333…, 1.5, 1.6, 1.875, 2.0], // == swaras[].ratio_decimal; → RaagaPlayer.play(num,{rootHz})
  "root_hz": 220,
  "arohana_indices":  [0, 1, 7, 9, 13, 14, 20, 22],  // shruti indices, ascending (engine.ts:189)
  "avarohana_indices":[22, 20, 14, 13, 9, 7, 1, 0],  // descending — arohana reversed (wisdom.ts:192)
  "prahar": { "num": 1, "label": "Sunrise", "is_recommended_time": true }, // current 3-h watch (engine.ts:191-195)
  "dosha_affinities": { "vata": true, "pitta": false, "kapha": false },    // which doshas list this raga (engine.ts:163-167)
  "alternate_ragas": [ { "num": 22, "name": "Kharaharapriya", "chakra": 4, "ma_type": "shuddha" } ], // only if include_alternates && dosha (engine.ts:170-176)
  "total_melakartas": 72
}
```

**Inputs (all optional, `engine.ts:38-70`):** `melakarta` (1–72), `name` (partial match, overrides number), `dosha` (`vata|pitta|kapha`), `root_hz` (default 220 = A3), `include_alternates` (bool). With none given it auto-selects by current prahar; with `dosha` only, by dosha × prahar (`engine.ts:96-131`).

**Envelope note (TS vs Rust):** the TS `EngineOutput` is `{ engine_id, result, witness_prompts[], calculated_at, processing_time_ms }` (`ts-engines/src/types/engine.ts:32-43`) — it has **no** `consciousness_level`/`metadata`; the bridge synthesizes those (`lib.rs:439-456`). The README envelope describes the post-bridge Rust shape.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `melakarta.num` | **1–72** | validated integer 1–72 (`engine.ts:110`); 1–36 = shuddha Ma, 37–72 = prati Ma (`wisdom.ts:181`) |
| `melakarta.chakra` | **1–12** | indu(1)…aditya(12); = `chakraInSet + (prati?7:1)` (`wisdom.ts:197`) |
| `melakarta.ma_type` | `shuddha` \| `prati` | shuddha = M1 (shruti 9), prati = M2/tivra (shruti 12) (`wisdom.ts:188,201`) |
| `swaras` | **8 entries** | labels `Sa Re Ga Ma Pa Dha Ni Sa'` (`engine.ts:143`); one per arohana degree |
| `swaras[].shruti_index` | **0–22** | index into the 22-shruti table (+Sa' at 22) (`wisdom.ts:19-43`) |
| `swaras[].ratio_num/den` | JI integers | hard-coded parallel arrays indexed by `shruti_index` (`engine.ts:147-152`) — **must stay in sync with `SHRUTIS`** (§7) |
| `swaras[].ratio_decimal` | **1.0 → 2.0** | one octave; monotonic non-decreasing along arohana; = `m.ratios[i]` (`engine.ts:153`) |
| `swaras[].hz` | `root_hz · ratio_decimal` | rounded to 3 dp (`engine.ts:154`); with default root_hz 220 → 220.0…440.0 |
| `arohana_indices` / `avarohana_indices` | 8 shruti indices each | avarohana = arohana reversed (`wisdom.ts:192`) — melakartas are sampurna (all 7 swaras, both directions) |
| `prahar.num` | **1–8** | 8 watches × 3 h; `getPraharForHour` (`wisdom.ts:255-276`) |
| `prahar.is_recommended_time` | bool | true iff this melakarta is in the current prahar's `recommended[]` (`engine.ts:160`) |
| `dosha_affinities.{vata,pitta,kapha}` | bool each | from the `DOSHA_AFFINITY` table (`wisdom.ts:231-238`); a raga may match 0, 1, or several |
| `root_hz` | > 0 (default 220) | not range-validated; echoes input (`engine.ts:80`) |

**Invariants.** (a) `swaras` (8), `arohana_indices` (8), `strudel_ratios` (8) are all derived from the same `m.arohana` — they co-vary, never independent (`engine.ts:144,189; wisdom.ts:190`). (b) The melakarta is fully determined by `num` via the Venkatamakhin formula (R-G pair from chakra-set, D-N pair from position, Ma from >36) — name/swaras/ratios are pure functions of `num` (`wisdom.ts:180-203`). (c) Auto-selection is **time-dependent** (reads `new Date().getHours()`, `engine.ts:120,128,158`) — same inputs at a different hour can pick a different raga and always recompute `prahar`. (d) The renderer recomputes the descending row itself (`avaroha`, `Raaga.tsx:75-78`) and **ignores** the producer's `avarohana_indices`.

## 4. Component & brand archetype
**Today** (`Raaga.tsx`): a text-card grid (melakarta #+name, chakra, Ma-type, root Hz, current prahar, dosha tags, vadi·samvadi, rasa/mood) + two **8-cell swara strips** — Ārohana (`Raaga.tsx:197-209`) and Avarohana (muted, `:212-224`), each cell = swara name + Hz — plus a Strudel **audio bar** (Play/Stop/WAV) with an optional **V2 panel** (timbre, gamaka, tala, breath selectors; `Raaga.tsx:237-296`). On-brand colors via CSS vars (`--gold`, `--gold-soft`, `--line-gold`), but **no geometry yet** — the swaras are a flat grid, not an arc/ring; chakra, prahar, vadi/samvadi are text only. Falls back to `GenericEngineView` when `result.melakarta` is missing (`Raaga.tsx:144`).

**Wave-2 target (tonal arc / swara wheel):** the 7 swaras placed on an **octave arc/ring** (Sa→Sa', 1.0→2.0), the raga's **present notes lit on the Ba-Arc** (shruti positions from `arohana_indices`), the **vadi/samvadi emphasized** as the dominant nodes, and the **current prahar shown as the wheel's orientation** (time-of-day rotation). Melodic, sacred-geometry, not decorative. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `swaras[].ratio_decimal` (1.0→2.0) | angular position of each swara node on the octave arc/ring (target) |
| `arohana_indices` (0–22) | shruti slots lit on the Ba-Arc; present notes glow, absent shruti dim (target) |
| `swaras[].swara` + `.hz` | node label (name) + tooltip/sub (Hz) — today the 8 strip cells (`Raaga.tsx:201-206`) |
| `vadi` / `samvadi` | (target) two emphasized nodes (vadi = brightest, samvadi at the consonant ~4th/5th) — **producer does not emit these (§7)** |
| `arohana` vs `avarohana` | ascending vs descending sweep around the ring; today two strips (`Raaga.tsx:197-224`) |
| `melakarta.ma_type` | Ma node placement: shuddha M1 vs tivra M2 (shruti 9 vs 12); today text `Tīvra/Shuddha Ma` (`Raaga.tsx:158`) |
| `melakarta.chakra` (1–12) | (target) outer ring sector / hue band; today text (`Raaga.tsx:158`) |
| `prahar.label` + `.is_recommended_time` | (target) wheel orientation = time-of-day; today the gold prahar box (`Raaga.tsx:165-175`) |
| `dosha_affinities` | active dosha tags (`Raaga.tsx:176-181`) — could tint the core |
| `melakarta.num` + `root_hz` | header + Strudel `play(num,{rootHz})` (`Raaga.tsx:109`) |

## 6. Dynamics
**One-shot per request, but inputs include the wall clock.** A reading is computed once from `(melakarta?|name?, dosha?, root_hz, include_alternates)` (`engine.ts:74-208`); not a live stream. However, when `melakarta`/`name` are omitted the selection and the reported `prahar` depend on `new Date().getHours()` (`engine.ts:120,128,158`), so re-running at a different watch can change the raga — a *soft* time-of-day cadence (unlike vedic-clock's continuous one). Audio is **interactive, not data**: the user drives Play/Stop/WAV and the V2 controls (timbre/gamaka/tala/breath) in the client `RaagaPlayer` (`Raaga.tsx:90-142`) — none of that feeds back into `result`. Natural one-time animation: the swara arc drawing in (stroke/scale), present-note glow, and the ascending→descending sweep. `consciousness_level` is fixed at 0 here (`required_phase`, `engine.ts:37`) so it gates nothing for this engine.

## 7. Open questions / assumptions
- **⚠️ NO Rust engine, NO OpenAPI stub.** Raaga is absent from the `EngineResultData` enum and has no `*ResultSchema` anywhere in `crates/noesis-core/` (verified). Its only producer is the **TypeScript** `RaagaEngine`, reached through `noesis-bridge` (`lib.rs:297-303`) → Bun server (`:3001`). The bridge passes `result` through unchanged (`lib.rs:441`), so the producer + renderer are the **only two contracts** — both authoritative.
- **⚠️ Producer ↔ renderer mismatch — `mood` / `rasa` / `vadi` / `samvadi` are read but never emitted (likely a real gap; the headline issue).** The renderer reads `result.mood`, `result.rasa`, `result.vadi`, `result.samvadi` (`Raaga.tsx:69-72`) and renders Vadi·Samvadi and Rasa/Mood cards (`Raaga.tsx:182-193`). **The producer's `result` object contains none of these** (`engine.ts:178-199`) — and `wisdom.ts` carries no rasa/vadi/samvadi data at all (the `Melakarta` interface is `num,name,chakra,arohana,avarohana,ratios,ma_type`, `wisdom.ts:170-178`). So today those cards are **always hidden**. This directly blocks the Wave-2 "vadi/samvadi emphasized" archetype: the data must be added to `wisdom.ts` + emitted in `engine.ts`, or the renderer must derive/drop them. **Resolve before building the swara wheel.**
- **Domain facts the task asked to verify — what actually exists vs not:**
  - *Raga name / thaat:* this is the **Carnatic melakarta** system (72 parent scales), the South-Indian analogue of Hindustani **thaat** — the melakarta *is* the parent scale (`engine.ts:1-7`; `wisdom.ts:73-92`). Individual `name`s are the canonical Venkatamakhin set (`wisdom.ts:95-168`).
  - *Swaras (Sa Re Ga Ma Pa Dha Ni):* present as the 8-degree arohana with shruti variants — komal/shuddha/tivra encoded as **shruti indices** (e.g. shuddha Ma = 9, tivra Ma = 12; R/G/D/N variants via the `SW` map, `wisdom.ts:52-70`), **not** as komal/tivra labels. The renderer surfaces only Ma-type as a word (`Raaga.tsx:158`).
  - *Aroha / avaroha:* yes — `arohana_indices` / `avarohana_indices` (`engine.ts:189`); melakartas are sampurna so avarohana = arohana reversed (`wisdom.ts:192`).
  - *Vadi / samvadi:* **NOT present** anywhere (see mismatch above) — would be new data.
  - *Time-of-day:* yes — 8 prahars (`PRAHARS`, `wisdom.ts:255-264`), reported as `prahar.{num,label,is_recommended_time}`.
  - *Season / rasa:* **NOT present** (no season field; rasa only read, never produced).
  - *Root frequency:* yes — `root_hz`, default **220 Hz (A3)** for Sa (`engine.ts:80`; `Raaga.tsx:158-164`).
- **`prahar.num` shape match:** producer emits `prahar.num` (`engine.ts:192`) and renderer reads `prahar.num` as optional (`Raaga.tsx:57`) — ✅ aligned (no mismatch here).
- **Duplicate/divergent data sources.** The 22-shruti table + 72 melakartas exist **twice** — backend `ts-engines/.../wisdom.ts` and web `apps/noesis-web/src/lib/raaga/{shrutis,melakartas}.ts` (the renderer's `RaagaPlayer` uses the web copy, `lib/raaga/index.ts:1-3`). The header comment claims they're kept "numerically identical" (`wisdom.ts:8-9`), but that's a manual invariant — **drift risk** (e.g. the hard-coded `ratio_num/ratio_den` arrays in `engine.ts:147-152` must also track `SHRUTIS`). Not verified equal here.
- **Vivadi aliases / "missing" geometry.** Some shruti slots collapse (R3≡G1 at index 5, D3≡N1 at index 18 — `wisdom.ts:36,38,52-70`); a swara-wheel must decide whether to draw 22 distinct positions or the collapsed 16. Flag for the Wave-2 ring design.
- **`avarohana_indices` is emitted but unused.** The renderer recomputes the descending row from `swaras` reversed (`Raaga.tsx:75-78`) rather than reading the producer's `avarohana_indices` — harmless today (melakarta avarohana = reverse), but the producer field is dead from the web's perspective.
