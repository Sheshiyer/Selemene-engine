# I Ching source transcription receipt

Prepared on 2026-10-07. This is a source preparation receipt, not a publication,
deployment, or licensing attestation.

## Selected historical source

- James Legge, *The Sacred Books of China, Part II: The Yi King*, Oxford,
  Clarendon Press, 1882; *Sacred Books of the East*, volume XVI.
- Independent original-edition scan contributed by Princeton Theological
  Seminary Library: <https://archive.org/details/sacredbooksofchi16conf>.
- The Archive item identifies volume 16, records its source as an 1882 copy, and
  labels its possible US copyright status `NOT_IN_COPYRIGHT`. The general
  collection date 1879 is not the date used for this volume: the actual title
  pages and item copyright-evidence field establish 1882.
- Source inputs: the item's original scan PDF, OCR XML and scan page inventory.
  Exact retrieval URLs and SHA256 hashes are in `i-ching-source-provenance.json`.
- The current Sacred Texts reader/title page was useful for locating the edition
  and contents. None of its modern prepared transcription files was copied into
  this dataset. Its title-page notice includes noncommercial/attribution wording;
  the independent original scan avoids relying on that prepared-file permission.

## Scope and record identities

`ts-engines/src/engines/i-ching/legge-wisdom.json` contains exactly 64 numbered
records, each with a primary judgment and a Great Symbolism passage.

The judgments are the opening text for each I–LXIV hexagram in the edition's
Text sections, printed pages 57–208. The images are the opening Great Symbolism
paragraphs for each I–LXIV section in Appendix II, printed pages 267–346.
Here "image" means the traditional symbolic passage, not a copied graphic.

Each record identifies its printed pages, PDF pages, Archive leaf URLs, and
curated OCR paragraph/line spans. IDs were matched to the source's numbered
sections. The extraction also checks all 64 Roman ID markers in Appendix II.
The separately verified King Wen line-pattern mapping belongs to the engine
adapter and is not inferred by this text extractor.

This preparation does **not** contain the six moving-line texts, the Small
Symbolism/line commentary, translator footnotes, new interpretations, modern
hexagram meanings, or a Wilhelm/Baynes translation. It should not be described
as a complete I Ching edition or a complete moving-line corpus.

## Transcription and uncertainty

All 128 selected passages were visually reviewed against the pinned original
scan, including page continuations. The prepared text preserves the archaic
English wording and source punctuation. It is a normalized transcription:

- Whitespace is collapsed and printed end-line word splits are joined.
- Section numerals and editorial headings are excluded from the passages.
- Historical transliteration tone marks and italic consonant typography are
  represented in readable ASCII. The historical z-shaped consonant is rendered
  `Z`, and `Lu` represents the tone/umlaut-marked printed name. These are
  typography changes, not substitutions of modern Chinese names or meanings.
- 46 substitutions correct OCR errors or normalize the historical
  transliterations. Every substitution has before/after text and source-page
  evidence in the provenance JSON. Source spans exclude accidentally merged
  footnotes and line commentary rather than treating those as wisdom text.
- The unusual unmatched closing parenthesis after `this` in the Great Symbolism
  for hexagram 13, page 284, is retained as printed and noted in its record.

The extractor's first heuristic candidate missed text in 27/37/52/61 and attached
commentary to 22/28/29/34/44/52/63. These candidates were not shipped. Final
extraction uses reviewed, explicit spans; the source hash must match before it
can run. The original scan remains authoritative. No unresolved English-word
OCR ambiguity was found in this review, but an independent second proofread has
not been completed. `scan_reviewed_normalized_transcription` must not be
relabelled as a character-perfect diplomatic edition.

## Reproduce the artifact

Download the three publicly available files identified in the provenance
manifest to a temporary source directory. The extractor performs no networking
and uses Python 3's standard library:

```sh
python3 docs/plans/free-plugin/publication/extract-legge-wisdom.py \
  --xml /tmp/selemene-legge1882-audit/sacredbooksofchi16conf_djvu.xml \
  --scandata /tmp/selemene-legge1882-audit/sacredbooksofchi16conf_scandata.xml \
  --output /tmp/legge-wisdom-reproduced.json \
  --provenance /tmp/legge-provenance-reproduced.json
```

Compare the reproduced files with the repository artifacts before accepting
changes. The extractor checks pinned source hashes, 524 access-format pages,
all 64 IDs and Great Symbolism markers, nonblank bounded passage lengths, unique
judgments/images, and unwanted OCR/commentary markers. Altered source hashes or
failed correction anchors cause a hard failure. It never invents missing text.

The original PDF SHA256 is
`216991696663472339e90f1a64aad45998fbe9de711885ab00744572475896c6`.
The dataset SHA256 is recorded in the provenance JSON after generation.

## Rail and authority receipt

A single loopback `noesis-build` parser dispatch was attempted with the local
tool surface, workspace-write scope, one attempt and a 300-second hard bound.
It timed out before producing an accepted code artifact or provider-attribution
receipt. Its own process group was terminated; no shared service was restarted.
The attempt log is `/tmp/selemene-legge-parser-build.log`. Direct preparation
then completed the reviewed spans, extraction helper and data artifacts within
the assigned ownership.

Publisher identity, countries of availability, applicable reuse/terms decisions,
and any marketplace attestations remain the publishing owner's decisions.
This source receipt records historical source facts and current Archive
metadata; it does not claim global legal clearance. No backend deployment or
provider mutation was performed during this preparation.
