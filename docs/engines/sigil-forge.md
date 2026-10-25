# Sigil Forge — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

> **🧊 P1 W1 Frozen (2026-07-17):** Contracts per [P1W1-CONTRACTS-FROZEN.md](../plans/engine-integration/P1W1-CONTRACTS-FROZEN.md) (worktree `.worktrees/T-002-copilot`). EngineInput/Output media, capture lifecycle, image provider, raaga audio locked (T-002+). Cite extraction: goal-understanding.md, resources-and-assets.md, gaps-and-improvements.md. See updated engine-matrix.json. Do not mutate w/o re-freeze. Handoff W2.

Guided sigil creation: turns a present-tense **intention** into a chosen **method** (word-elimination, rose-wheel, pictographic, chaos-star) with process steps, charging suggestions, and witness prompts — plus an *optional* AI-generated sigil image (base64 PNG or URL from a configured provider). It does **not** auto-generate a vector glyph.

> **Lane S4 (2026-10-26) — the engine now tells the truth about its images.** Every output carries a contract `provenance` block (§2a); providers that cannot produce a real image say so instead of returning a placeholder PNG (§2b); the OpenAPI `SigilForgeResultSchema` now describes the real runtime result (the old `sigil_id` / `vector_path` fiction is gone).

## 1. Identity
| | |
|---|---|
| `engine_id` | `sigil-forge` (confirmed — producer `metadata().id` & output `engine_id`, `ts-engines/src/engines/sigil-forge/engine.ts:59,303`) |
| Crate | **— (no Rust engine; OpenAPI stub + renderer)** |
| Runtime source | **`ts-engines/src/engines/sigil-forge/engine.ts`** (`SigilForgeEngine`, L32) — TS sidecar, registered `ts-engines/src/index.ts:27` |
| Renderer | `apps/noesis-web/src/components/engines/SigilForge.tsx` |
| Fixture (real values) | **none found** (no `tests/fixtures` for sigil-forge; output shape taken from producer + `engine.test.ts`) |
| OpenAPI schema | `crates/noesis-core/src/types.rs` (`SigilForgeResultSchema` + `SigilForgeMethodSchema`, `SigilForgeSvgPreviewSchema`, `SigilForgeGeneratedImageSchema`, `SigilForgeImageMetadataSchema`) — matches the runtime result since lane S4 |

Supporting TS modules: `wisdom.ts` (4 `SIGIL_METHODS`, 5 `CHARGING_METHODS`, `processWordElimination`), `prompt-builder.ts` (`buildSigilPrompt`/`buildSigilEditPrompt`, NIM prompt + style), `witness.ts` (`generateWitnessPrompts`), `providers/image-provider.ts` (`ImageProvider` interface, canonical `NvidiaImageProvider`, `MockImageProvider` for tests, `createImageProvider`/`createDefaultImageProvider`), `providers/nano-banana.ts`, `providers/kimi.ts`, `utils/nvidia-image.ts` (low-level NIM client; it re-exports the canonical provider, the duplicate class was removed in lane S4).

## 2. Output schema

**Runtime JSON (authoritative — what `SigilForgeEngine.calculate()` returns in `result`):**
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
    "remaining_letters": "TRCPL",
    "letter_count": 5
  },
  "charging_suggestions": [                             // exactly 2, sampled from CHARGING_METHODS
    { "name": "Meditation Gnosis", "description": "…" },
    { "name": "Destruction Charging", "description": "…" }
  ],
  "guidance": {
    "note": "…",                                        // conditional on a REAL image (see §2a)
    "next_steps": ["…", "…"]                            // "contemplate the generated image…" only when a real image exists
  },
  "svg_preview": { "status": "absent" },               // OBJECT {status, reason?} — NOT an SVG string. status ∈ absent|accepted|rejected
  "generated_image": null,                             // null unless generate_image/edit_image_b64 set (shape below)
  "image_gen_available": false,                        // provider.isAvailable() for the provider actually used
  "provider": "nvidia",                                // provider actually used: nvidia | nano-banana | kimi | mock
  "provider_note": "…",                                // only when parameters.provider could not be honoured
  "seed": 1234567890
}
```

`result.generated_image` when an image was requested:
```jsonc
{
  "b64_json": "…",            // real image only (or "url")
  "url": "https://…",         // real image only (nano-banana may return a URL)
  "prompt_used": "…",
  "style": "chaos",
  "model": "black-forest-labs/flux.1-dev",
  "error": "…",               // present on failure / unconfigured / mock — then b64_json + url are absent
  "metadata": {
    "provider": "nvidia",                  // provider actually used
    "model": "black-forest-labs/flux.1-dev",
    "finish_reason": "SUCCESS",            // provider completion status (see §2b)
    "provider_requested": "kimi",          // only when parameters.provider was overridden
    "provider_note": "Requested provider 'kimi' is not configured; using nvidia."
  }
}
```

Envelope is the **TS** shape: `{ engine_id, result, witness_prompts, calculated_at, processing_time_ms, provenance, generated_image? }`. The top-level FROZEN `generated_image` slot is populated **only for a real image** (`fallback_used === false`); mock, placeholder, unconfigured and errored attempts stay inside `result.generated_image` with their `error`. The Rust bridge stashes `provenance` under the reserved key `__contract_provenance` in `result`, and `noesis-api` lifts it back out to the envelope's top-level `provenance` (and strips it before readings are persisted).

### 2a. Provenance (contract `ContractProvenance`, seven fields max)

| Field | Sigil Forge value |
|---|---|
| `runtime_kind` | always `"typescript"` |
| `implementation_version` | `metadata().version` (`"2.0.0"`) |
| `cached` | always `false` |
| `fallback_used` | `false` **only** when a real image was produced; `true` for placeholder, mock, error, unconfigured provider, **and when no generation was requested** (guidance-only mode) |
| `backend_id` | not emitted by this engine |
| `provider_id` | the provider that actually produced image bytes: `nvidia` \| `nano-banana` \| `kimi` \| `mock`; **omitted** when no image was produced |
| `confidence` | `0.9` on a real image, `0` otherwise |

The block is attached exactly as tarot does (`output.provenance`). The Rust parser (`noesis_core::contract::Provenance`, `deny_unknown_fields`) accepts exactly these seven field names, so never add others.

### 2b. Provider truth table

| Provider | `isAvailable()` | Real image when | Without credentials / on failure | `finish_reason` values |
|---|---|---|---|---|
| `nvidia` (default when no `RUNCOMFY_TOKEN`) | `NVIDIA_API_KEY` set | NIM returns an artifact | throws → engine records `{error}`; `finish_reason: "ERROR"` / `"PROVIDER_UNAVAILABLE"` | NIM `finishReason` (e.g. `SUCCESS`) |
| `nano-banana` (default when `RUNCOMFY_TOKEN` set) | `RUNCOMFY_TOKEN` set | runcomfy CLI returns an image path/URL | `{error, mock: true}` — **never a placeholder PNG** | `SUCCESS`, `MOCK_NO_TOKEN`, `ERROR` |
| `kimi` | `KIMI_API_KEY` set | never (image endpoint unresolved) | `{error, mock: true}` | `MOCK_NO_KEY`, `KIMI_IMAGE_ENDPOINT_UNAVAILABLE` |
| `mock` (`MockImageProvider`, tests only) | always | n/a — returns a 1x1 PNG that the engine treats as a fallback (`provider_id: "mock"`, `fallback_used: true`, not surfaced top-level) | — | `SUCCESS_MOCK`, `SUCCESS_MOCK_EDIT` |

`parameters.provider` selects a provider **only when that provider is configured and available**; otherwise the engine's default provider is used and `result.provider_note` + `generated_image.metadata.provider_note` explain why.

**Renderer's expected shape (`SigilForge.tsx` — not the producer's):**
Reads `result.sigil.{name,symbol_set,numerological_base,activation_phrase,svg_preview}` *or* flat `result.{sigil_name,symbol_set,numerological_base,activation_phrase,svg_preview}`, plus `result.activated_elements ?? result.elements` and `result.intention_field ?? result.intention`. It expects `svg_preview` to be an **HTML/SVG string** and injects it via `dangerouslySetInnerHTML` (L35). **None** of `sigil.name`, `symbol_set`, `numerological_base`, `activation_phrase`, `activated_elements`, `intention_field` are produced by `SigilForgeEngine`. Only `result.intention` matches. The renderer should be rewritten against the producer shape above (render `generated_image.b64_json`/`url` when `provenance.fallback_used === false`).

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `intention` | non-empty string | required; from `input.question` or `parameters.{intention,intent,intent_text,question}` (`engine.ts:115-120`); trimmed; empty → `MISSING_INTENTION` validation error |
| `method.id` | `word-elimination` \| `rose-wheel` \| `pictographic` \| `chaos-star` | `wisdom.ts:13-75`; unknown `parameters.method` → `INVALID_SIGIL_METHOD` |
| method auto-select | length>50 → word-elimination; ≤3 words → pictographic; else seeded random pick | `engine.ts:152-161` |
| `processing` | object \| **null** | non-null only for `word-elimination`; `letter_count == remaining_letters.length` |
| `charging_suggestions` | **exactly 2** | seeded sample of 5 `CHARGING_METHODS` (`engine.ts:170-171`) |
| `svg_preview.status` | `absent` \| `accepted` \| `rejected` | `absent` if no `parameters.svg_template`; **rejected** if not a complete `<svg>…</svg>`, or if it contains `<script`/`onload=` (`safeSvgPreview`, `engine.ts:33-51`). Server-side allowlist guard. |
| `generated_image` | **null** \| `{b64_json?,url?,prompt_used?,style?,model?,error?,metadata}` | null in default guidance mode; populated only when `parameters.generate_image=true` or `parameters.edit_image_b64` set. Unconfigured/failed/mock → `{error, metadata.finish_reason}` and **no** `b64_json`/`url` |
| `image_gen_available` | boolean | `isAvailable()` of the provider actually used (credential presence) |
| `provider` | `nvidia` \| `nano-banana` \| `kimi` \| `mock` | provider actually used (after `parameters.provider` resolution) |
| `provenance` (envelope) | see §2a | `fallback_used` is the single source of truth for "is there a real image" |
| `seed` | number | echoes input seed for reproducibility |
| `image_style` (input) | `ceremonial`\|`chaos`\|`organic`\|`geometric`\|`runic`\|`ethereal`\|`yantra` | default method-dependent (`prompt-builder.ts`) |
| `provider` (input) | `nvidia`\|`nano-banana`\|`kimi` | honoured only if configured; else default + `provider_note` |

**Determinism:** identical `(intention, method, seed)` → identical guidance, processing, charging picks, witness prompts. **Image generation is the one non-deterministic, network-dependent, slow (5–15 s) path** (`engine.test.ts:82`). Default mode is pure/fast.

**Invariants:** there is **no canonical sigil identifier** and **no generated vector geometry**. The engine's stated contract is that the human draws the glyph ("this personal investment is essential to the magic"). A top-level `generated_image` exists **iff** `provenance.fallback_used === false`.

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
- **No dedicated Rust engine (resolved fiction).** There is **no Rust `engine-sigil-forge` crate**; the real producer is the **TS sidecar** `ts-engines/src/engines/sigil-forge/engine.ts`. Until lane S4 the OpenAPI `SigilForgeResultSchema` described `sigil_id` / `vector_path`, which no code ever produced; the schema now mirrors the runtime result (`intention`, `method`, `svg_preview`, optional `generated_image`, optional `provenance`). This engine deliberately makes the *human* draw the glyph; the only machine-generated visual is an optional raster image, never an SVG path.
- **Producer ↔ renderer divergence (still open).** Renderer expects `{sigil.{name,symbol_set,numerological_base,activation_phrase,svg_preview}, activated_elements, intention_field}`; only `intention` overlaps with the producer. Rewrite the renderer against §2 (or add a mapper).
- **Artifact persistence (follow-up, not in lane S4).** Real images are returned inline (`b64_json`/`url`) and are not persisted to R2. Persisting them depends on lane S3's `ts-engines/src/utils/r2.ts`.
- **🔒 SECURITY — injecting a server-provided SVG.** `SigilForge.tsx:35` injects `svg_preview` via **`dangerouslySetInnerHTML`**. If the renderer is ever fed a *string* `svg_preview` (as the renderer expects, and as the stub's "vector_path"/SVG idea implies), that is a stored-XSS vector: a raw `<svg>` can carry `<script>`, `onload=`, `<foreignObject>`/`<a href=javascript:>`, etc. The producer's `safeSvgPreview` (`engine.ts:42-50`) only checks `<svg…</svg>` + blocks `<script`/`onload=` — **substring-based and easily bypassed** (`onerror=`, `onclick=`, `on…` handlers, `<animate>`/`<set attributeName=href>`, encoded payloads). Net: today the renderer receives an *object* (so it stringifies harmlessly to `[object Object]`), but the moment a string path/template flows through, **the current guard is insufficient** — sanitize (e.g. DOMPurify with an SVG profile) before any `dangerouslySetInnerHTML`, or render a typed `vector_path` as an SVG `<path d>` attribute (data-only, never markup) instead of injecting markup. If a `vector_path` producer is added, prefer the **typed `d`-attribute** route precisely to avoid this.
- **`ln` / `RaagaEngine` red herring.** A stale code-index hit suggested `engine_id: 'ln'` for this engine; the on-disk file is unambiguous — `id:'sigil-forge'` (L59), `engine_id:'sigil-forge'` (L303). The `ln`/`lnEngine` references belong to a *different* (Raaga/Nadabrahman-adjacent) engine and are unrelated to Sigil Forge.
- **Envelope mismatch.** TS returns `witness_prompts:string[]`, `calculated_at`, `processing_time_ms`; the Rust `EngineOutput` (README envelope) expects `witness_prompt`, `consciousness_level`, `metadata`. The adapter that maps the sidecar response into the canonical envelope (and back to `engine_id "sigil-forge"`) was not located — **unconfirmed**; confirm how `noesis-api` proxies `ts-engines` before relying on envelope fields for this engine.
- **No fixture.** Unlike the Vedic engines, there's no reference fixture; the schema here is read from the producer source + `engine.test.ts`. Capture a real default-mode response as a fixture during Wave-2.
- **Synthesis reads other keys.** `creative_expression.rs` probes `sigil_data.{intention_or_intent, energy, keywords, distilled, available}` — **none of which the current producer emits** (those came from an older/mock shape, e.g. `mock_sigil_output` at `creative_expression.rs:622`). So `CreativeExpressionSynthesis` likely degrades to its empty/"available:false" path with real data — confirm and align if synthesis output matters for Wave-2.
