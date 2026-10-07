# Source readiness audit: I Ching and Gene Keys

**Historical baseline, superseded for repaired fields:** This audit describes commit74608e54 and the initial installed workflow, before source remediation. Current source/test and deployed I Ching status are in [source-remediation.json](source-remediation.json). Gene Keys activation repair is tested; production verification is pending, and its meaning dataset remains unverified/incomplete. The intended public scope retains all reviewed engines. Historical omission options below are not the selected plan.

Inspected on 2026-10-07. This is a read-only source audit following the primary operator's genuine installed-plugin `decision-support` execution. That execution returned all five expected engine outputs, but its I Ching payload reported `casting.line_values: [6, 7, 6, 6, 6, 6]`, primary hexagram `1 / The Creative`, and relating hexagram `64` with explicit stub wording. Those observations were supplied by the primary operator; this audit did not issue another calculation.

The inspected I Ching engine/data and Gene Keys engine/loader/data are byte-identical in MAIN and the plugin worktree. Worktree HEAD at inspection was `74608e5466033313b8b1a5216e12f764103bd736`. File links below point to the inspected local worktree. This audit does not independently establish the deployed upstream's source revision, validate the traditional material against an external edition, or replace the returned results with a newly computed reading.

**Publication status: source-quality acceptance remains incomplete for these affected capabilities.** Five present output objects establish transport membership, not five semantically complete readings.

## I Ching: algorithm and dataset

The issue is broader than a missing label fallback:

1. [Primary selection, engine.ts:54](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/engine.ts:54) takes an explicitly supplied hexagram ID, or samples an integer from 1 to 64. It then looks up that table entry. It never derives the primary hexagram from the six cast values.
2. [Independent line sampling, engine.ts:86](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/engine.ts:86) separately samples each line value with `rng.nextInt(6, 9)` and marks values 6 or 9 as changing. The source itself identifies 6 as old yin, 7 as young yang, 8 as young yin, and 9 as old yang. The reported cast values therefore do not describe the all-yang pattern stored for entry 1. No external mapping is required to observe this inconsistency.
3. [Relating selection, engine.ts:98](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/engine.ts:98) constructs a flipped `relatingLines` array from the primary entry and changing indices, then never uses it. The relating number is another independent random integer from 1 to 64. There is no matching-pattern lookup or implemented transformation mapping.
4. [Method labeling, engine.ts:139](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/engine.ts:139) copies the requested method into the output. The calculation contains no method-specific branch implementing coin tosses or yarrow operations. Thus `method: three_coins` is a returned label, not verification of that casting procedure. Uniform sampling of four values makes two changing values half of the outcomes; the source comment claiming about 25 percent is inconsistent with its implementation.
5. [Explicit table entries, wisdom.ts:16](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/wisdom.ts:16) provide individual names, meanings, judgments, images, and line arrays for entries 1–8. Their presence establishes local text coverage, not independently verified canonical accuracy.
6. [Stub generation, wisdom.ts:98](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/wisdom.ts:98) creates entries 9–64: **56 of 64 entries** have generic names, explicit `stub data` meanings, generic judgment/image strings, and line arrays generated from the bits of the numeric ID. This is neither a verified sequence mapping nor substantive source text.
7. [Witness transformation templates, witness.ts:16](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/src/engines/i-ching/witness.ts:16) narrate a transformation from primary to relating using these independently selected entries. A generated witness prompt cannot validate that relationship.
8. [Current integration assertions, integration.test.ts:191](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/ts-engines/tests/integration.test.ts:191) check number range, explicitly selected entry 1, invalid-ID refusal, a non-null relating entry when changing indices exist, and prompt presence. They do not test cast-to-primary consistency, all 64 pattern mappings, relating transformation, or meaningful complete text coverage.

| Returned field | Reliable scope | Reading restriction |
| --- | --- | --- |
| `casting.line_values` | Records the six values sampled by this code for this run. | Do not claim they generated the reported primary or relating hexagrams, or represent a verified named casting procedure. |
| `changing_lines` | Identifies positions sampled as 6 or 9 by this implementation. | Do not turn those indices into a validated transformation or line-specific traditional reading. |
| Primary number/name/text | Reports a selected local table entry. Entries 1–8 have individual local text. | Treat as a software-selected reflection prompt only if that limitation is explicit; do not call it the computed result of the cast. |
| Relating number/name/text | Reports another selected local table entry. | Do not claim this is derived from changing lines. Entry 64 has stub text here; do not silently replace it with a remembered traditional title or meaning. |
| Seed, timestamps, engine ID | Returned technical provenance for the software result. | Provenance does not establish semantic correctness or source completeness. |
| Witness prompts | Local template output. | Skip prompts premised on a valid primary-to-relating transformation; they inherit the source defect. |

For the observed result, preserve the raw values as reported technical output, state that the consultation is incomplete, and withhold an authentic cast/relating-hexagram interpretation. Do not re-cast, compute a replacement hexagram in ChatGPT, or fill stub text from memory to make the engine appear complete.

## Gene Keys: lines and wisdom coverage

The [wisdom loader, wisdom.rs:24](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/crates/engine-gene-keys/src/wisdom.rs:24) embeds `data/gene-keys/archetypes.json` at compile time. Its validation checks that all 64 numeric IDs exist. It does not establish semantic correctness, authenticity of labels, source licensing, or completeness of key-specific descriptions.

A read-only count of the embedded dataset established:

- There are 64 key records. Keys 1–6 have individual descriptions; keys 7–64, **58 records**, each use the same shadow-description template, the same gift-description template, and the same siddhi-description template. Each description field has only seven unique strings across the entire dataset.
- Those 58 records also use generic `Gene Key N` names. [Key 7, archetypes.json:140](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/data/gene-keys/archetypes.json:140) is the first such record and demonstrates the generic templates.
- Forty records have identical gift and siddhi labels within the record. Repetition alone does not prove every label wrong, but these data have not been independently checked against an authoritative source in this audit. They cannot support a claim of verified complete traditional teaching.
- The JSON's [source metadata, archetypes.json:2](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/data/gene-keys/archetypes.json:2) names a synthesis by Richard Rudd. This metadata assertion is not a retrieval receipt, quotation citation, or independent source validation.

The line issue also affects birth-data mode. [Birth-data calculation, engine.rs:307](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/crates/engine-gene-keys/src/engine.rs:307) invokes Human Design, extracts four gate numbers, and calls `create_chart_from_gates`. [The constructor's explicit limitation, engine.rs:117](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/crates/engine-gene-keys/src/engine.rs:117) says the line is a placeholder; [all four assignments, engine.rs:153](/Users/sheshnarayaniyer/.codex/worktrees/selemene-free-plugin/Selemene-engine/crates/engine-gene-keys/src/engine.rs:153) set `line: 3`. This is not calculated line data. The longitude/line information available in Human Design is discarded on this path.

| Returned field | Reliable scope | Reading restriction |
| --- | --- | --- |
| Key numbers and activation-source labels | Values selected from the four Human Design gate IDs and assigned to the named source positions by this code. | Preserve as returned technical values with their source. This audit does not independently validate the upstream Human Design chart or the full sequence model. |
| `active_keys[*].line` | Constant 3 on the inspected birth-data/gates constructor path. | Mark unavailable as a calculated line; never deliver line-specific interpretation from it. |
| Name, shadow/gift/siddhi labels | Looked-up embedded dataset strings. | Attribute as service-supplied text, not independently verified authentic source teaching. Do not repair labels silently from memory. |
| Descriptions for keys 7–64 | Repeated generic templates rather than key-specific substance. | Do not present as a detailed account of the active key or amplify their repetition into cross-system corroboration. |
| Frequency/witness commentary | Rule/template output using account phase and dataset material. | Do not infer the reader's current spiritual state or verified developmental stage from the shared administrator account's phase. Withhold conclusions premised on incomplete meanings or placeholder lines. |

## Effect on the current workflow reading

The current execution may truthfully be described as returning all five expected engine objects. It must not be described as five source-complete interpretations or a validated integrated decision reading. Mark I Ching cast/transformation interpretation and Gene Keys line/key-specific wisdom as unavailable or incomplete. Keep other successful systems separate and assess them from their actual returned evidence; this narrow audit does not certify them.

The host's Aletheios, Pichet, and synthesis stages must not use placeholder labels, generic descriptions, or the sampled relating entry to manufacture themes, agreement, predictions, or corroboration. Explain the limitation plainly, retain the factual source map, and provide only a qualified partial reading from unaffected material. No extra inference call or silent re-computation can cure this source issue.

## Narrow remediation options

1. **Correct the upstream engines and datasets.** I Ching requires an actual chosen casting procedure, a tested six-line-to-hexagram mapping, relating derivation from the changed pattern, and substantive verified coverage for all 64 entries. Gene Keys requires preserving real gate lines from Human Design, validated sequence inputs, and reviewed key-specific source material. Use approved authoritative source data and meaningful regression checks for mapping, transformations, line preservation, dataset coverage, and placeholder absence. Then verify the corrected deployed engine revisions through the installed plugin and re-run affected saved public cases. This is separate implementation and deployment work, not completed by this audit.
2. **Omit affected public capabilities for this release.** Remove I Ching and affected Gene Keys reading claims from the public surface until corrected. `decision-support` includes both engines, so do not retain or advertise the existing five-engine workflow as fully reviewed merely by hiding its incomplete results. A reduced workflow would need an explicit new composition/contract and its own review; alternatively omit this workflow from the public release. Align catalog filtering, tools, skills, listing/review cases, and deployment evidence with the actual reduced scope before submission.

No engine, Worker, manifest, or installed package was changed by this audit. No calculation, external source lookup, deployment, or remote mutation was performed. Public source-quality acceptance remains incomplete until a remediation path is implemented and verified.
