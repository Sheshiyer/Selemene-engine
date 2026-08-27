# Synthetic Fixture Engine Receipt

Internal evidence artifact for the whitepaper demonstration section. Do not copy endpoint URLs, auth details, raw prompts, or this receipt framing into the public paper. The public paper should use only bounded, source-status-labeled claims.

## Request

| Field | Value |
|---|---|
| Date run | 2026-08-26 |
| Request class | Synthetic workflow execution |
| Auth | `X-API-Key: [redacted]` |
| Subject | Mira Synthetic |
| Birth date | 1991-04-18 |
| Birth time | 06:42 |
| Coordinates | latitude 12.0000, longitude 77.0000 |
| Timezone | Asia/Kolkata |
| Consent/public use scope | temporal-symbolic demonstration only |

## Response receipt

| Field | Value |
|---|---|
| HTTP status | 200 |
| Workflow id | birth-blueprint |
| Response timestamp | 2026-08-26T13:37:28.382081918Z |
| Reported total time | 1.945127 ms |
| Returned output keys | face-reading, human-design, numerology, vimshottari, biofield |

## Public-admitted fields

Only the following fields were admitted into the public whitepaper trace.

| Lens family | Admitted values | Public status |
|---|---|---|
| Time normalization | 1991-04-18T01:12:00Z | deterministic |
| Number-symbol | life path 6; birthday 9; expression 11; personality 5; soul urge 6 | deterministic/selected |
| Period-timing | birth nakshatra Rohini; run-timestamp stack Jupiter / Mercury / Rahu | deterministic/selected |
| Archetypal type | Generator; Sacral authority; profile 2/4; Split definition; active channels 33-13 and 3-60 | deterministic/selected |

## Omitted or reviewed fields

| Material | Action | Reason |
|---|---|---|
| Body/media-adjacent outputs | Omitted from public miniature | Public example consent scope is temporal-symbolic only. |
| Raw lightweight witness prompts | Not quoted directly | Returned prompts require public voice review before publication. |
| Body-signal interpretations | Not admitted | Public demonstration did not include body signal consent or evidence. |
| Face/media traditions | Not admitted | Public demonstration did not include image consent or evidence. |

## Reproduction note

The receipt was produced by loading `SELEMENE_API_KEY` from the local Claude environment file and sending a synthetic birth-data request. The key value was never printed.
