# Nadabrahman — Data Reference


> **Note (2026-06-30):** `apps/noesis-web` has been retired. Engine renderers are being ported to [Sankalpa](../../sankalpa/). Paths referencing `apps/noesis-web` below are historical.

Raga-based sound therapy: given a moment (and optional dosha / rasa / chakra), recommends Melakarta ragas by **prahar** (3-hour time block), plus a chakra Solfeggio frequency for tuning. "Nāda Brahman" = sound as the substrate of consciousness.

## 1. Identity
| | |
|---|---|
| `engine_id` | `nadabrahman` (engine.rs:37; asserted engine.rs:399) |
| Domain crate | `crates/engine-nadabrahman/src/` — output type `NadaBrahmanAnalysis` (models.rs:55) |
| Runtime serializer | **`engine.rs:171` `serialize_result`** — the JSON the API actually returns (authoritative; differs from the struct, see §2) |
| Data files (embedded) | `data/nadabrahman/{melakarta_ragas.json, time_raga_mappings.json, chakra_frequencies.json}` (data.rs:13–15) |
| Renderer | `apps/noesis-web/src/components/engines/Nadabrahman.tsx` |
| Renderer raga DB | `apps/noesis-web/src/lib/raaga/melakartas.ts` (`MELAKARTAS`, 72 entries — melakartas.ts:78) |
| Fixture (real values) | **none** — no `tests/` dir; reference values taken from unit-test assertions (data.rs:255–280, engine.rs:460–465) and the embedded JSON |
| OpenAPI stub | `noesis-core/src/types.rs:291` (`NadabrahmanResultSchema`) — **fictional, see §7** |
| Runtime source | `engine-nadabrahman` crate (standalone; no vedic-api path) |

## 2. Output schema

**Runtime JSON (authoritative — exactly what `serialize_result` emits, engine.rs:171–228):**
```jsonc
{
  "time_recommendation": {                          // always present
    "prahar_name": "Pratah (Morning)",              // models.rs:120-131 — one of 8
    "prahar_number": 1,                             // 1–8
    "time_range": "06:00-09:00",                    // models.rs:134-145
    "primary_raga": { "raga_number": 15, "raga_name": "Mayamalavagowla", "reason": "…" },
    "dosha_dominance": "kapha",                      // kapha|pitta|vata (engine.rs:147-154)
    "energy_quality": "ascending"                    // engine.rs:157-168
    // NOTE: secondary_ragas exists on the struct (models.rs:48) but is NOT serialized here
  },
  "recommendations": [                              // always present, ≤5, score-desc
    {
      "raga_number": 15, "raga_name": "Mayamalavagowla",
      "reason": "…", "score": 1.0,                  // f64, see §3
      "arohanam":  ["sa","ri1","ga3","ma1","pa","dha1","ni3","sa"],  // enriched if raga known (engine.rs:184-189)
      "avarohanam":["sa","ni3","dha1","pa","ma1","ga3","ri1","sa"],
      "mood": "devotional", "therapeutic_qualities": ["…"]
    }
  ],
  "chakra_frequency": {                             // OPTIONAL — only if `chakra` option given
    "chakra_name": "heart",                         // echoes the input string, not Sanskrit
    "solfeggio_hz": 639.0,                          // Hz, see §3
    "binaural_target_hz": 8.0                        // Hz beat-frequency
  },
  "dosha_recommendation": "vata",                   // OPTIONAL — a STRING (echo of input), not an object
  "rasa_mapping": "shanta"                          // OPTIONAL — a STRING (echo of input), not an object
}
```

**Domain struct `NadaBrahmanAnalysis` (models.rs:55):** `time_recommendation: PraharRecommendation` · `recommendations: Vec<RagaRecommendation>` · `chakra_frequency: Option<ChakraFrequency>` · `dosha_recommendation: Option<String>` · `rasa_mapping: Option<String>`.
Sub-types: `PraharRecommendation` (models.rs:43) carries `secondary_ragas` + `primary_raga` as full `RagaRecommendation`; `RagaRecommendation` (models.rs:26) = `{raga_number:u32, raga_name:String, reason:String, score:f64}`; `ChakraFrequency` (models.rs:35) = `{chakra_name, solfeggio_hz, binaural_target_hz}`; `Raga` (models.rs:9) = the full 12-field record (number, name, chakra, arohanam, avarohanam, madhyama_type, mood, time_of_day, therapeutic_qualities, dosha_affinity, rasa, consciousness_level) — used to **enrich** recommendations, never serialized whole.

**OpenAPI stub (types.rs:291):** `{ root_frequency_hz: 432.0, suggested_raga: "Bhairavi", mantra_seed: "AUM" }` — **none of these three fields is produced by the engine.** Pure documentation placeholder. See §7.

## 3. Ranges, constraints & invariants
| Field | Range / domain | Notes |
|---|---|---|
| `prahar_number` | **1–8** | 3-hour blocks; derived from `current_time.hour()` (engine.rs:44, models.rs:91-103). Boundaries: 1=06–09, 2=09–12, 3=12–15, 4=15–18, 5=18–21, 6=21–00, 7=00–03, 8=03–06 |
| `prahar_name` | 8 fixed strings | Pratah / Sangava / Madhyahna / Aparahna / Sayahna / Pradosha / Nisha / Brahma Muhurta (models.rs:120-131) |
| `dosha_dominance` | `kapha`\|`pitta`\|`vata` | prahar→dosha map (engine.rs:147-154); note prahars 7 & 8 are both `vata` |
| `energy_quality` | enum (8) | ascending/expansive/peak/sustaining/transitional/inward/deep/awakening (engine.rs:157-168) |
| `raga_number` | **1–72** | Melakarta system; full DB has 72 ragas (data.rs:211). Renderer also clamps 1–72 (Nadabrahman.tsx:38,44) |
| `recommendations[]` length | **0–5** | truncated to 5 after dedup+sort (engine.rs:103); renderer further `.slice(0,6)` (Nadabrahman.tsx:238) |
| `score` | **f64, ~0.5–1.0** | primary = `1.0 − i·0.05`; secondary = `0.7 − i·0.1` (data.rs:101,118); dosha/rasa hits = `0.7` (data.rs:146,163); fallback raga = `0.5` (engine.rs:53). **Not normalized**, can be ≤0 for long secondary lists |
| `solfeggio_hz` | **396 / 417 / 528 / 639 / 741 / 852 / 963 Hz** | the 7-chakra Solfeggio ladder (chakra_frequencies.json; root=396 & heart=639 asserted data.rs:261,278) |
| `binaural_target_hz` | **0.5 / 4 / 6 / 8 / 10 / 12 / 14 Hz** | beat-frequency per chakra (chakra_frequencies.json); delta/theta/alpha range, not audible tones |
| `chakra_name` | echo of input string | accepts Sanskrit ("Muladhara") **or** English ("root"); case-insensitive (data.rs:172-202). Unknown chakra → `chakra_frequency` omitted entirely |
| swaras (`arohanam`/`avarohanam`) | 8-note arrays | ascending/descending scale; tokens `sa ri1 ga3 ma1 pa dha1 ni3 sa` (varianted swaras 1–3) |
| `dosha_recommendation` / `rasa_mapping` | string or absent | present only when `dosha` / (`rasa`\|`mood`) option supplied (engine.rs:76-98) |

**Invariants.** `recommendations` is sorted score-descending after de-dup by `raga_number` (engine.rs:101-103). `time_recommendation` + `recommendations` are **always** present (validator enforces, engine.rs:300-308); `chakra_frequency`, `dosha_recommendation`, `rasa_mapping` are conditional on input options. The witness prompt is non-empty or the call errors (engine.rs:259-263). **Output co-varies with wall-clock hour** — the same birth data yields a different prahar/raga each 3-hour block (cache key is hour-granular, engine.rs:341-363).

## 4. Component & brand archetype
**Today** (`Nadabrahman.tsx`): an **all-text** view — a gold mantra box (only if `result.mantra` present, line 204), a 3-cell grid (Chakra+element / Dosha / Rasa+emotion, lines 211-232), and a "Sound Practices" tag row where each resolvable raga gets ▶ **play** and ⬇ **WAV** buttons backed by a real Web-Audio/Strudel synth (`getRaagaPlayer`, lines 234-272). Below: an **audio-source toggle** (Suno recording vs. live Strudel) and a collapsible **V2 control bar** — timbre (sine/sitar/bansuri/sarangi), gamaka (kampita/andolana/kurula/nokku/sphurita), tala (7 options), breath (7 prāṇāyāma patterns), lines 274-355. If none of `chakra_frequency` / `mantra` / `dosha` is present it falls back to `GenericEngineView` (line 200). **No visualization** — zero SVG/canvas, no waveform; it is a player + metadata panel, not a geometric figure.

**Wave-2 target:** a **harmonic waveform / frequency arc** — render `solfeggio_hz` as a base sine and stack its overtone series (2f, 3f, …) as concentric arcs or a layered wave; the active raga's `arohanam` swaras become tick marks / nodes along the arc; chakra color drives hue; the bija mantra + note sit at the core as an acoustic readout (not a decorative glyph). The play button should drive a live amplitude trace. Brand palette: Void `#070B1D`, Gold `#C5A017`, Emerald `#10B5A7`, Indigo `#0B50FB`, Violet `#2D0050`, Parchment `#F0EDE3`. This is the only engine whose primary output is genuinely **audio**, so the visual should be a faithful spectrogram/oscilloscope of what's playing, time-of-day–tinted by prahar.

## 5. Data → visual mapping
| Field | Visual (Wave-2 target) |
|---|---|
| `chakra_frequency.solfeggio_hz` (Hz) | base wave frequency / radius of the fundamental arc; pitch label at core |
| overtone series (2f,3f,4f… of solfeggio_hz) | stacked concentric arcs / layered sine partials — the "frequency arc" |
| `chakra_frequency.binaural_target_hz` (Hz) | slow beat modulation (envelope pulse) of the base wave |
| `chakra_name` | hue of the wave (root→crown chakra color ramp) |
| `recommendations[].arohanam` / `avarohanam` | swara nodes/ticks along the arc (ascending vs descending sweep) |
| `prahar_number` (1–8) | 8-segment day-ring; active prahar lit; overall scene tint |
| `energy_quality` | amplitude / animation tempo of the trace |
| `score` (per rec) | node size / opacity in the recommendation row |
| bija mantra + `note` (from data file) | acoustic readout at the core — **needs engine fix to ship, see §7** |

## 6. Dynamics
**Time-driven, near-live.** Unlike the one-shot Vedic engines, output depends on `current_time.hour()` → the recommended prahar/raga **changes every 3 hours**; the cache key is hour-granular (engine.rs:341-363), so a re-fetch after a block boundary returns new data. Within a block it's stable. The **audio layer is fully interactive** (client-side): play/stop/download, source switching, and V2 timbre/gamaka/tala/breath all mutate playback live via `getRaagaPlayer()` without re-fetching engine output (Nadabrahman.tsx:104-198). `dosha`/`rasa`/`chakra` are **request options**, not birth data — changing them re-runs the engine. `consciousness_level` (0–5) is envelope-level and may gate interpretive depth; `required_phase` is 0 (available to everyone, engine.rs:249-251). Wave-2: the waveform should animate in sync with actual playback (oscilloscope), idle to a slow breath when stopped.

## 7. Open questions / assumptions
- **OpenAPI stub is fiction (confirmed).** `NadabrahmanResultSchema` (types.rs:291) advertises `root_frequency_hz` (432.0), `suggested_raga` ("Bhairavi"), `mantra_seed` ("AUM") — **the engine emits none of them.** There is no 432 Hz anywhere (frequencies are the 396–963 Solfeggio ladder), no top-level `suggested_raga` (it's `time_recommendation.primary_raga.raga_name` + `recommendations[]`), and no `mantra_seed`. Do **not** build against the stub. ⚠️
- **Renderer reads fields the engine never emits (confirmed, highest-impact).** `Nadabrahman.tsx` reads `chakra_frequency.mantra` (81), `.frequency_hz` (82), `.note` (83), `.element` (216); top-level `result.mantra` (81); and treats `rasa_mapping` as an **object** with `.rasa/.name/.emotion` (79,228-229). The engine emits `chakra_frequency` = `{chakra_name, solfeggio_hz, binaural_target_hz}` only (engine.rs:212-216) and `rasa_mapping` as a **plain string** (engine.rs:223). Net effect at runtime: the **mantra box never shows** (no `mantra` field), the **Hz line never shows** (renderer reads `frequency_hz`, engine emits `solfeggio_hz`), `note`/`element` are blank, and the Rasa cell renders `String(undefined)` → "undefined". The data **exists** in `chakra_frequencies.json` (each chakra has `note` C–B, `bija_mantra` LAM/VAM/RAM/YAM/HAM/OM/Silence, `element`) but `serialize_result` drops it. **Fix is one-sided: enrich the serializer** to pass through `note`, `mantra`/bija, `element`, and rename/alias `solfeggio_hz`→`frequency_hz` (or fix the renderer). Until then the panel is mostly empty unless a `chakra` option is sent. ⚠️
- **`secondary_ragas` is computed but not serialized.** `PraharRecommendation.secondary_ragas` is populated (engine.rs:61) but `serialize_result` omits it from `time_recommendation` (engine.rs:196-207). They do still reach the client folded into the top-level `recommendations[]`. Confirm Wave-2 doesn't need the primary/secondary split.
- **`score` is unbounded below.** Secondary scores `0.7 − i·0.1` go negative for the 8th+ secondary raga; truncation to 5 usually hides this, but don't assume `score ∈ [0,1]` when mapping to opacity/size.
- **No fixture exists.** Values in §3 are from unit-test assertions + the embedded JSON, not a captured API response. A golden fixture (`crates/engine-nadabrahman/tests/`) would lock the runtime shape — currently the only guard is the validator (engine.rs:291-339).
- **Renderer dispatch path unverified.** Could not locate the `engine_id → <Nadabrahman>` mapping via grep (likely a dynamic registry); assumed standard routing on `engine_id === "nadabrahman"`. Confirm in the engine-result router before relying on it.
- **`chakra_name` is the raw input echo**, not normalized to Sanskrit (engine.rs:113). If the renderer keys colors off a canonical name, normalize first (the lookup already knows both forms, data.rs:182-186).
