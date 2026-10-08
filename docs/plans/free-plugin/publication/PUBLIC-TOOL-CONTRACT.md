# Public MCP operation remediation contract

Status: prepared; not implemented, deployed or rescanned. The current public draft remains v0.2.4 with four operation findings. The user requires them resolved before submission even though the portal classifies them as advisory.

## Endpoint and compatibility

Keep the existing private `/mcp` eight-tool registry and dispatch unchanged. Add `/mcp/public` in the same Worker with the same sponsor secret, input sanitizer, origin guards, request admission, calculation admission and Durable Object quota identity. Public package v0.2.5 retains the same package and public portal identity. Shared OmniRoute/provider routing and credential settings remain unchanged; adding the public Worker endpoint and migrating the public portal connection are intended changes.

The public registry will expose exactly39 tools: separate engine catalog and capability reads; three fixed operations for each11 reviewed engines; four existing workflow/chart tools. The four legacy generic tools must be absent from both public discovery and public dispatch, with no hidden aliases. Backend credentials and service-account permission lookup remain internal authorization.

## Engine operation map

| Reviewed engine | Explicit public operations |
| --- | --- |
| numerology | `selemene_numerology_info` / `selemene_numerology_prepare` / `selemene_numerology_calculate` |
| human-design | `selemene_human_design_info` / `selemene_human_design_prepare` / `selemene_human_design_calculate` |
| gene-keys | `selemene_gene_keys_info` / `selemene_gene_keys_prepare` / `selemene_gene_keys_calculate` |
| vimshottari | `selemene_vimshottari_info` / `selemene_vimshottari_prepare` / `selemene_vimshottari_calculate` |
| panchanga | `selemene_panchanga_info` / `selemene_panchanga_prepare` / `selemene_panchanga_calculate` |
| vedic-clock | `selemene_vedic_clock_info` / `selemene_vedic_clock_prepare` / `selemene_vedic_clock_calculate` |
| biorhythm | `selemene_biorhythm_info` / `selemene_biorhythm_prepare` / `selemene_biorhythm_calculate` |
| transits | `selemene_transits_info` / `selemene_transits_prepare` / `selemene_transits_calculate` |
| enneagram | `selemene_enneagram_info` / `selemene_enneagram_prepare` / `selemene_enneagram_calculate` |
| tarot | `selemene_tarot_info` / `selemene_tarot_prepare` / `selemene_tarot_calculate` |
| i-ching | `selemene_i_ching_info` / `selemene_i_ching_prepare` / `selemene_i_ching_calculate` |

Catalog tools are `selemene_engine_catalog` and `selemene_engine_capabilities`, each with one distinct backend read. Catalog membership does not prove readiness. Retained tools are `selemene_list_workflows`, `selemene_workflow_info`, `selemene_run_workflow`, and `selemene_vedic_chart`.

## Input and execution rules

Fixed tool closures bind the selected reviewed engine ID. Caller inputs cannot supply engine_id, destination, URL, phase, credential or unknown nested fields. Calculation schemas derive from the existing reviewed schemas while preserving every original refinement, including Vedic Clock timezone alternatives and Enneagram adjacency. Valid scalar fields remain typed and bounded in partial preparation schemas. Preparation performs only local validation and reports missing fields; it never calculates or saves. Invalid calculations fail before calculation admission and upstream calls.

Fixed info results return the actual public input shapes and explicit operation mappings. They must not direct the host to pass engine_id into a tool that rejects it. Engine identity in factual results remains unchanged. All descriptions and annotations reflect actual behavior. Persistence and shared-account disclosures remain explicit; interpretation remains in the current ChatGPT conversation with no hosted inference.

## Package and test coverage

Update only the public staging manifest, MCP URL and instruction files. Preserve verified publisher, category, all four policy/support URLs, icon, original private release and unrelated Witness work. Retain five positive/three negative cases, updated to cover all39 actual tools: discovery/info, all11 preflights, eight birth/time calculations, three standalone symbolic/type calculations, and decision-support plus Vedic chart. Actual saved-public host execution remains Not run until observed.

Acceptance requires local strict-input/refinement/destination/isolation/quota tests, typecheck, dry-run, SDK readback of both endpoints, a full public archive inspection, exact same-public-plugin saved readback, explicit portal connection migration, and a fresh scan with no corresponding operation findings. Renamed tools or a successful upload alone do not close the scan criterion.

## Current execution hold

Hosted Build stopped before generation for hosted-evidence-unavailable. One explicitly authorized process-local loopback retry retained all admission checks and was held for elevated macOS memory pressure. No provider resolved and no implementation edits occurred. Root has asked whether the user wants bounded native production or retains routed Build. No hold was bypassed; shared configuration is unchanged.
