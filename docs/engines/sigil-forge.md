# Sigil Forge — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

> **🧊 P1 W1 Frozen (2026-07-17):** Contracts per [P1W1-CONTRACTS-FROZEN.md](../plans/engine-integration/P1W1-CONTRACTS-FROZEN.md) (worktree `.worktrees/T-002-copilot`). EngineInput/Output media, capture lifecycle, image provider, raaga audio locked (T-002+). Cite extraction: goal-understanding.md, resources-and-assets.md, gaps-and-improvements.md. See updated engine-matrix.json. Do not mutate w/o re-freeze. Handoff W2.

Guided sigil creation: turns a present-tense **intention** into a chosen **method** (word-elimination, rose-wheel, pictographic, chaos-star) with process steps, charging suggestions, and witness prompts — plus an *optional* AI-generated sigil image (NVIDIA NIM, base64 PNG). It does **not** auto-generate a vector glyph.

> ⚠️ **Read [§7](#7-open-questions--assumptions) first.** The OpenAPI stub (`sigil_id` / `intention` / `vector_path`) describes a glyph-generating engine that **does not exist in this repo**. No code anywhere produces a `vector_path` / SVG `d` string. Build against the **TS producer** + **renderer**, not the stub.

## 1. Identity
| | |
|---|---|
| `engine_id` | `sigil-forge` (confirmed — producer `metadata().id` & output `engine_id`, `ts-engines/src/engines/sigil-forge/engine.ts:59,303`) |
| Crate | **— (no Rust engine; OpenAPI stub + renderer)** |
| Runtime source | **`ts-engines/src/engines/sigil-forge/engine.ts`** (`SigilForgeEngine`, L32) — TS sidecar, registered `ts-engines/src/index.ts:27` |
| Renderer | `apps/noesis-web/src/components/engines/SigilForge.tsx` |
| Fixture (real values) | **none found** (no `tests/fixtures` for sigil-forge; output shape taken from producer + `engine.test.ts`) |
| OpenAPI stub | `crates/noesis-core/src/types.rs:357` (`SigilForgeResultSchema` — `sigil_id`/`intention`/`vector_path`) — **examples only, and they do not match runtime** |

Supporting TS modules: `wisdom.ts` (4 `SIGIL_METHODS`, 5 `CHARGING_METHODS`, `processWordElimination`), `prompt-builder.ts` (`buildSigilPrompt`/`buildSigilEditPrompt`, NIM prompt + style), `witness.ts` (`generateWitnessPrompts`), `utils/nvidia-image.ts` (`generateImage`/`editImage`/`isImageGenAvailable`).

## 2. Output schema

**Runtime JSON (authoritative — what `SigilForgeEngine.calculate()` returns in `result`, `engine.ts:255-300`):**
```jsonc
{
  "intention": "I attract peace and clarity",        // cleaned input string (present-tense statement)
  "method": {                                          // selected sigil method (object, NOT a string)
    "id": "word-elimination",                          // ∈ word-elimination | rose-wheel | pictographic | chaos-star
    "name": "Word Elimination Method",
    "description": "…",
    "steps": ["…", "…"]                                // ordered how-to steps
  },
  "processing": {                                       // ONLY when method.id === "word-elimination"; else null
    "type": "word_elimination",
    "original": "I attract peace and clarity",
    "remaining_letters": "TRCPL",                       // dedup'd consonants (processWordElimination)
    "letter_count": 5
  },
  "charging_suggestions": [                             // exactly 2, sampled from CHARGING_METHODS
    { "name": "Meditation Gnosis", "description": "…" },
    { "name": "Destruction Charging", "description": "…" }
  ],
  "guidance": {
    "note": "This engine provides the process… the actual visual sigil must be created by you.",
    "next_steps": ["…", "…"]                            // branches on whether an image was generated
  },
  "svg_preview": { "status": "absent" },               // OBJECT {status, reason?} — NOT an SVG string. status ∈ absent|accepted|rejected
  "generated_image": null,                             // null unless generate_image/edit_image_b64 set (see §3)
  "image_gen_available": false,                        // isImageGenAvailable() — NVIDIA_API_KEY present?
  "seed": 1234567890                                    // SeededRandom seed (input.seed ?? getDefaultSeed())
}
```
Envelope is the **TS** shape (`engine.ts:302-308`): `{ engine_id, result, witness_prompts: string[], calculated_at, processing_time_ms }` — note `witness_prompts` (plural array), `calculated_at`/`processing_time_ms`, **not** the Rust `EngineOutput` (`witness_prompt`, `consciousness_level`, `metadata`). Whatever proxies the sidecar into the Rust envelope is unconfirmed.

**OpenAPI stub (`types.rs:357` — DOES NOT MATCH runtime; treat as fiction, not even valid examples):**
```rust
pub struct SigilForgeResultSchema {
    pub sigil_id: String,    // example "SIG-20260303-9A7"  ← no producer emits this
    pub intention: String,   // example "clarity"           ← runtime key matches, value is a full sentence
    pub vector_path: String, // example "M1 L10,2 L15,8 …"  ← NO code generates an SVG path anywhere
}
```
Only `intention` overlaps (and even then the stub's one-word `"clarity"` misrepresents the real full-sentence value). `sigil_id` and `vector_path` appear in **zero** runtime code paths (verified: `rg "vector_path|sigil_id"` across `ts-engines` + `crates/*/src` returns nothing outside the stub).

**Renderer's expected shape (`SigilForge.tsx` — a THIRD shape, also not the producer's):**
Reads `result.sigil.{name,symbol_set,numerological_base,activation_phrase,svg_preview}` *or* flat `result.{sigil_name,symbol_set,numerological_base,activation_phrase,svg_preview}`, plus `result.activated_elements ?? result.elements` and `result.intention_field ?? result.intention`. It expects `svg_preview` to be an **HTML/SVG string** and injects it via `dangerouslySetInnerHTML` (L35). **None** of `sigil.name`, `symbol_set`, `numerological_base`, `activation_phrase`, `activated_elements`, `intention_field` are produced by `SigilForgeEngine`. Only `result.intention` matches. With the real payload the guard at L25 (`if (!sigil && !elements && !intention)`) passes via `intention`, then renders essentially just the Intention Field box — and the `svg_preview` object would be coerced to `[object Object]` text inside `dangerouslySetInnerHTML`.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `intention` | non-empty string | required; from `input.question` or `parameters.{intention,intent,intent_text,question}` (`engine.ts:115-120`); trimmed; empty → `MISSING_INTENTION` validation error |
| `method.id` | `word-elimination` \| `rose-wheel` \| `pictographic` \| `chaos-star` | `wisdom.ts:13-75`; unknown `parameters.method` → `INVALID_SIGIL_METHOD` |
| method auto-select | length>50 → word-elimination; ≤3 words → pictographic; else seeded random pick | `engine.ts:152-161` |
| `processing` | object \| **null** | non-null only for `word-elimination`; `letter_count == remaining_letters.length` |
| `charging_suggestions` | **exactly 2** | seeded sample of 5 `CHARGING_METHODS` (`engine.ts:170-171`) |
| `svg_preview.status` | `absent` \| `accepted` \| `rejected` | `absent` if no `parameters.svg_template`; **rejected** if not a complete `<svg>…</svg>`, or if it contains `<script`/`onload=` (`safeSvgPreview`, `engine.ts:33-51`). Server-side allowlist guard. |
| `generated_image` | **null** \| `{b64_json?,url?,prompt_used?,style?,model?,error?}` | null in default guidance mode; populated only when `parameters.generate_image=true` (new) or `parameters.edit_image_b64` set (edit). If `NVIDIA_API_KEY` missing → `{error:"…"}` (`engine.ts:193-251`) |
| `image_gen_available` | boolean | mirrors `NVIDIA_API_KEY` presence |
| `seed` | number | echoes input seed for reproducibility |
| `image_style` (input) | `ceremonial`\|`chaos`\|`organic`\|`geometric`\|`runic`\|`ethereal` | NIM style; default method-dependent (`engine.ts:84-89`, `prompt-builder.ts`) |

**Determinism:** identical `(intention, method, seed)` → identical guidance, processing, charging picks, witness prompts. **Image generation is the one non-deterministic, network-dependent, slow (5–15 s) path** (`engine.test.ts:82`). Default mode is pure/fast.

**Invariant the stub violates:** there is **no canonical sigil identifier** and **no generated vector geometry**. The engine's stated contract is that the human draws the glyph ("this personal investment is essential to the magic", `engine.ts:279`).

## 4. Component & brand archetype
**Today** (`SigilForge.tsx`): if `svg_preview` is a string, inject it raw in `svgBox`; then a small grid of `Sigil` (name + symbol_set) and `Num. Base` cells, an italic **Intention Field** box, **Activated Elements** as gold tags, and a centered **activation phrase**. Falls back to `GenericEngineView` when none of sigil/elements/intention present. Mostly text on `var(--field)` with gold accents — **no geometry today**, and (per §2) most cells stay empty against the real payload.

**Wave-2 target:** a **sigil construction diagram**. Since the producer yields **no `vector_path`**, the brand's "geometry drawing itself" must be sourced one of two honest ways: (a) render the optional **`generated_image.b64_json`** (NIM PNG) as the central glyph; or (b) if/when a real `vector_path` producer is added, draw that SVG `d` string with **Anime.js `stroke-dashoffset`** line-draw. Readout: **intention** + **method.name** + (for word-elimination) the **`remaining_letters`** as the distilled core. Optional faint **construction grid** (rose-wheel / chaos-star geometry implied by `method.id`) behind the drawn form. Charging suggestions become a footer ritual. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`.

## 5. Data → visual mapping
| Field | Visual |
|---|---|
| `intention` | center readout / title; the "seed phrase" the glyph encodes |
| `method.id` | construction-grid template behind the glyph (rose-wheel = petalled wheel, chaos-star = 8-ray star, pictographic = combinatorial, word-elimination = letterform) |
| `method.name` + `method.steps` | side panel "how this was forged" (step list) |
| `processing.remaining_letters` | distilled letter-core rendered as the glyph's strokes (word-elimination only); animate draw-in |
| `generated_image.b64_json` | **the drawn glyph** (NIM PNG) when present — fade/scale-in on load |
| *(hypothetical) `vector_path`* | **does not exist** — would be the Anime.js `stroke-dashoffset` line-draw target if a producer is added |
| `charging_suggestions[]` | footer ritual chips (2) |
| `svg_preview.status` | dev/QA badge only (`accepted`/`rejected`/`absent`) — **never inject `.reason` or template blindly** (see §7) |
| `seed` | small "reproducible" marker |

## 6. Dynamics
**One-shot** per `(intention, method, seed)`. Not live; no time-of-day recompute (unlike vedic-clock), no baseline/delta (unlike biofield). Recompute only when the intention/method/seed/image-flags change. Two cost tiers: **default guidance** = synchronous, deterministic, sub-ms-ish; **image mode** (`generate_image`/`edit_image_b64`) = async NIM call, 5–15 s, network- and key-dependent, may return `{error}`. On render: animate the glyph (PNG fade-in, or — if a vector_path ever exists — a single line-draw); no perpetual loop except an optional slow brand "breath." Envelope `consciousness_level` is absent from the TS output, so depth-gating (if any) is applied by whatever wraps the sidecar, not here.

## 7. Open questions / assumptions
- **🚨 NO DEDICATED ENGINE + STUB IS FICTION (most important).** There is **no Rust `engine-sigil-forge` crate**. The only Rust artifact is the OpenAPI stub `SigilForgeResultSchema` (`types.rs:357`), and its fields **`sigil_id` and `vector_path` are produced by no code in this repo** (verified by grep across `ts-engines` and all `crates/*/src`). The real producer is the **TS sidecar** `ts-engines/src/engines/sigil-forge/engine.ts`. **Do not build anything against `sigil_id`/`vector_path`.** The brief's premise that `vector_path` is "the generated glyph" is **not implemented** — this engine deliberately makes the *human* draw the glyph; the only machine-generated visual is an optional NIM **PNG**, never an SVG path.
- **Three-way schema divergence (confirmed).** Producer emits `{intention, method{}, processing|null, charging_suggestions[], guidance{}, svg_preview{status}, generated_image|null, …}`. Stub claims `{sigil_id, intention, vector_path}`. Renderer expects `{sigil.{name,symbol_set,numerological_base,activation_phrase,svg_preview}, activated_elements, intention_field}`. **Only `intention` is common to all three.** The renderer would display almost nothing useful from the real payload. Wave-2 must reconcile renderer ↔ producer (rewrite the renderer to the producer shape, or add a mapper).
- **🔒 SECURITY — injecting a server-provided SVG.** `SigilForge.tsx:35` injects `svg_preview` via **`dangerouslySetInnerHTML`**. If the renderer is ever fed a *string* `svg_preview` (as the renderer expects, and as the stub's "vector_path"/SVG idea implies), that is a stored-XSS vector: a raw `<svg>` can carry `<script>`, `onload=`, `<foreignObject>`/`<a href=javascript:>`, etc. The producer's `safeSvgPreview` (`engine.ts:42-50`) only checks `<svg…</svg>` + blocks `<script`/`onload=` — **substring-based and easily bypassed** (`onerror=`, `onclick=`, `on…` handlers, `<animate>`/`<set attributeName=href>`, encoded payloads). Net: today the renderer receives an *object* (so it stringifies harmlessly to `[object Object]`), but the moment a string path/template flows through, **the current guard is insufficient** — sanitize (e.g. DOMPurify with an SVG profile) before any `dangerouslySetInnerHTML`, or render a typed `vector_path` as an SVG `<path d>` attribute (data-only, never markup) instead of injecting markup. If a `vector_path` producer is added, prefer the **typed `d`-attribute** route precisely to avoid this.
- **`ln` / `RaagaEngine` red herring.** A stale code-index hit suggested `engine_id: 'ln'` for this engine; the on-disk file is unambiguous — `id:'sigil-forge'` (L59), `engine_id:'sigil-forge'` (L303). The `ln`/`lnEngine` references belong to a *different* (Raaga/Nadabrahman-adjacent) engine and are unrelated to Sigil Forge.
- **Envelope mismatch.** TS returns `witness_prompts:string[]`, `calculated_at`, `processing_time_ms`; the Rust `EngineOutput` (README envelope) expects `witness_prompt`, `consciousness_level`, `metadata`. The adapter that maps the sidecar response into the canonical envelope (and back to `engine_id "sigil-forge"`) was not located — **unconfirmed**; confirm how `noesis-api` proxies `ts-engines` before relying on envelope fields for this engine.
- **No fixture.** Unlike the Vedic engines, there's no reference fixture; the schema here is read from the producer source + `engine.test.ts`. Capture a real default-mode response as a fixture during Wave-2.
- **Synthesis reads other keys.** `creative_expression.rs` probes `sigil_data.{intention_or_intent, energy, keywords, distilled, available}` — **none of which the current producer emits** (those came from an older/mock shape, e.g. `mock_sigil_output` at `creative_expression.rs:622`). So `CreativeExpressionSynthesis` likely degrades to its empty/"available:false" path with real data — confirm and align if synthesis output matters for Wave-2.
