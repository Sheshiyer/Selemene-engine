# Witness Alchemist reference contract

Reference read in full: `../723/_legacy/witnessalchemist-reading/inputs/01_Reading_WitnessAlchemist.docx`, 707 nonempty OOXML paragraphs and approximately 12,598 words, prepared 9 May 2026. Extraction includes table-cell text in document order. The archived filename is not `Kundali_Integrated_WitnessAlchemist.docx`; its actual title is The Integrated Chart and subject is Cumbipuram Nateshan Sheshnarayan Iyer, The Witness Alchemist. Retain unchanged. Text extraction tool: `tools/humdes-extractor/extract-witness-reference.mts`.

## Method to preserve

The opening establishes the purpose of each language before interpreting the person. Vedic sidereal is framed as karma and timing, Western tropical as persona, Human Design as mechanics, Gene Keys as frequency, and live transits as weather. Vimshottari is a timing technique within the Vedic lens, rather than another independent vote. Convergence is repeated symbolic emphasis; disagreement is texture. The author explicitly opposes flattening the languages and asks the reader to hear their combined chord.

The first substantive chapter compares the same life layer across systems, then explains convergences, then divergences. The detailed Vedic foundation follows this map, so facts serve meaning rather than forcing the reader to interpret raw placements. Life-area chapters braid systems within each topic; they are not isolated monographs for each engine. The timeline makes natal pattern, period interpretation, and dated transit context distinct. Practices lead into final integrated meaning.

## Complete source structure

1. Opening Why Five Systems.
2. Part I Convergence Map: identity stack; agreements; disagreements and texture.
3. Part II Vedic Foundation: Lagna; Moon; Sun; placements table; house distribution; Saraswati Yoga; yoga/dosha strengths; functional benefics/malefics; Atmakaraka; single-sentence signature.
4. Part III Karmic Architecture: Rahu/Ketu; eighth-house inheritance; twelfth-house Pluto; lived dasha trail; repeating patterns.
5. Part IV Career and Dharma: career cluster; yoga; paths; periods; foreign themes; public recognition.
6. Part V Wealth and Money: accumulation; networks; property; inheritance; earning periods; debt cautions.
7. Part VI Love and Partnership: seventh house; Darakaraka; meeting routes; periods; quality/friction; separation themes; partner sketch.
8. Part VII Health and Energy: constitution; somatic themes; mental/emotional patterns; periods; routines.
9. Part VIII Family Roots and Lineage: mother; father; siblings; children; inherited patterns; spiritual growth.
10. Part IX Master Timeline: historical current window; pivot; antardasha map; Saturn overlay; year-by-year map; cross-reference dashboard.
11. Part X Multisystem Practices: Vedic rituals; numerology; HD decisions/environment; Gene Keys contemplation; practical corrections.
12. Part XI Synthesis: headline; life lesson; avoid/pursue; strengths; purpose; future scenarios; one unifying practice; closing.

## Concrete contradiction design

The source contrasts Western Leo visibility with sidereal Ashlesha privacy: visibility becomes a channel for protected inward content, with recovery time after public expression. It contrasts Scorpio intensity with Generator response and 2/4 retreat/network rhythm: apply depth to a response the person actually wants, rather than treating intensity as proof of consent. It contrasts Jupiter expansion with Gate 41 contraction/fantasy: examine which futures deserve actual commitment. Later it holds vocation expansion and structural restraint together rather than interpreting one as cancellation of the other.

For each new subject, use only supported placements. Name two signals in their own systems, explain the tension without claiming equivalence, locate where it may appear in lived experience, and offer a reversible experiment plus a witnessing question. A contradiction in computed facts is a calculation discrepancy to reconcile; it is not interpretive texture. Symbolic convergence is not independent scientific confirmation.

## Necessary adaptations

The source's May 2026 current-date language is historical on 4 October 2026. Do not copy its future September transition as an upcoming event. Its period tables contain internal inconsistencies: Jupiter/Venus appears as September 2033 to May 2036 in earlier sections but July 2034 to March 2037 in the detailed map. Its sidereal Saturn Leo 2027 claim must not be propagated without current computed support. All new dates must come from the present subjects' verified facts.

Retain contemplative language, but replace deterministic claims such as guaranteed destiny, structural privilege, unavoidable marriage, permanent bedrock, and chart protection with attributed symbolic interpretation and reader agency. Do not repeat health diagnoses, surgical timing restrictions, gemstone purchases, or financial guarantees. Use ordinary restorative practices and open questions. Avoid borrowed biography, family dynamics, spouse characteristics, and past-life assertions as factual knowledge.

For the triad, preserve the method and change the domains to collective meaning, each person's decision rhythm, pairwise contacts, Penta contribution and gaps, interpersonal friction, explicit responsibilities, delivery cadence, pricing/commitments, conflict repair, and experiments. A Penta gate is attributed chart context; it does not establish authority, professional competence, or an employment role.

## Document narrative layout

Reference page system is US Letter portrait, width 12240 and height 15840 twips, top/bottom 1440, side margins 1296, header/footer distance 708. Body uses Calibri 11 pt, 320-twip line spacing and 140-twip paragraph after spacing. Cover centers the title at 28 pt and metadata at 10–15 pt. Major chapter titles are 18 pt; subheads 14 pt. Dense placement matrices are used as reference aids, surrounded by explanatory prose. The renderer derives geometry and body rhythm from this source, with black titles/headings and light neutral table fills. It adds unobtrusive page numbers for a long reading. These are intentional presentation adaptations; the user's direction concerns narrative method rather than byte-preserving template edits.

## Rendering handoff

`tools/humdes-extractor/render-witness-documents.mts` accepts one finalized Markdown and output directory. It creates DOCX, converts using bundled LibreOffice, then renders every PDF page to PNG. Runtime paths must be injected through `WITNESS_NODE_PACKAGES`, `WITNESS_SOFFICE`, and `WITNESS_PDFTOPPM`, populated from the workspace dependency loader. Use bundled LibreOffice only; never the desktop application's installation. Package marker must run once immediately before first authoring command. No Python, npm, installation, or network is needed.

The marker paths are `documents/26.930.11008/skills/documents/container_tools/mark_artifact_operation_started.mjs` and `pdf/26.930.11008/skills/pdf/container_tools/mark_artifact_operation_started.mjs` under the primary-runtime plugin root. Execute with Bun, expected-output-count 8 and formats docx/pdf respectively. Render images remain QA intermediates. Visual inspection of every final page is required before a delivery claim.
