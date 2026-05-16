# Suno v5.5 — Manual Generation Guide

> Reference for crafting raga/consciousness clips manually in the Suno Pro interface.
> Pairs with the automated pipeline in `ts-engines/scripts/suno-bulk-gen.ts`.

---

## UI Overview (v5.5 Pro)

| Tab | What it does |
|-----|-------------|
| **Simple** | Describe music in plain text — Suno generates lyrics + style automatically |
| **Advanced** | Full control: separate Lyrics + Style fields (use this for ragas) |
| **Sounds** | Upload audio references or use Suno's sound library |

### Toolbar Buttons

| Button | Description |
|--------|-------------|
| **+ Audio** | Attach an audio reference clip to guide the generation |
| **+ Voice** *(New in v5.5)* | Clone or select a voice profile for vocal tracks |
| **+ Inspo** | Pull inspiration from an existing Suno track by URL |

---

## Advanced Mode Fields (what the bulk-gen script fills programmatically)

### Lyrics (`prompt` in API)
- Write structured lyrics using Suno meta-tags
- Leave **blank** for instrumental (but pair with `make_instrumental: true`)
- **Max ~300 chars** for style prompt after lyrics are stripped

**Suno meta-tag syntax:**
```
[Intro]
[Verse 1]
[Pre-Chorus]
[Chorus]
[Verse 2]
[Bridge]
[Outro]
[Break]
[Instrumental]
[A Cappella]
[Fade out]
```

### Styles (`tags` in API)
Comma-separated descriptors. Suno v5.5 is highly tag-responsive.

**Structure used in the raaga engine:**
```
{genre}, {mood/texture}, {instruments}, {tempo hint}, {other descriptors}
```

---

## Raaga-Specific Style Presets

These are the exact tag sets used by `buildSunoPrompt()` in the codebase:

### `ambient`
```
indian classical, ambient, drone, instrumental, raga
```
> BPM: 60–70 | Instruments: sustained tanpura drone, sitar lead
> Mood: spacious, contemplative, slow-evolving

### `meditative`
```
indian classical, meditation, raga, instrumental, devotional
```
> BPM: 50–60 | Instruments: tanpura drone, bansuri flute, soft tabla
> Mood: devotional, peaceful, breath-paced

### `cinematic`
```
indian classical, cinematic, orchestral, raga, instrumental
```
> BPM: 80–90 | Instruments: orchestral strings, sarangi, sitar, soft percussion
> Mood: epic, grand, narrative-arc

### `acid`
```
indian classical, acid house, electronic, raga, instrumental, charanjit singh
```
> BPM: 124–132 | Instruments: TR-808 drums, TB-303 squelch bassline, Jupiter-8 lead, sustained drone
> Mood: hypnotic, driving, charanjit-singh-1982-style

---

## Prompt Template (Advanced Mode — Lyrics field)

```
Indian classical raga {RAGA_NAME} (Carnatic melakarta #{NUM}), 
{MOOD} {STYLE} instrumental, no vocals, played at {BPM} 
with {INSTRUMENTS}. 
Mood: {CHAKRA_MOOD}. 
Approximately {DURATION} seconds.
```

### Chakra Mood Map (by melakarta group of 6)

| Melakartas | Chakra | Mood |
|-----------|--------|------|
| 1–6   | 1 | earth-foundation, grounding, root-chakra |
| 7–12  | 2 | flow, eyes-open-perception, sacral |
| 13–18 | 3 | fire-activation, transformative, solar-plexus |
| 19–24 | 4 | wisdom-seat, hip-grounded, structural |
| 25–30 | 5 | directed-intention, lower-belly, focused |
| 31–36 | 6 | rhythmic-center, navel, seasonal |
| 37–42 | 7 | sage-discernment, solar-plexus, clarity |
| 43–48 | 8 | heart-opening, devotional-warmth |
| 49–54 | 9 | creative-expansion, chest, breath-of-creation |
| 55–60 | 10 | directional-power, shoulders, expansive |
| 61–66 | 11 | fierce-expression, throat, voice-unblocked |
| 67–72 | 12 | solar-consciousness, crown, luminous |

---

## Example: Manual Creation for Mayamalavagaula (#15) — Ambient

**Lyrics field:**
```
Indian classical raga Mayamalavagaula (Carnatic melakarta #15), spacious, contemplative, slow-evolving ambient instrumental, no vocals, played at 60-70 BPM with sustained tanpura drone, sitar lead. Mood: fire-activation, transformative, solar-plexus. Approximately 45 seconds.
```

**Styles field:**
```
indian classical, ambient, drone, instrumental, raga
```

**Song Title:**
```
Mayamalavagaula (#15) — ambient
```

**Settings:**
- ✅ Instrumental (toggle ON or leave lyrics blank with `make_instrumental`)
- Model: **v5.5** (select in top-right dropdown)
- Duration: target ~45s (v5.5 auto-determines length from prompt cues)

---

## Suno API Bridge Reference

Our bridge at `SUNO_BRIDGE_URL` (self-hosted) exposes:

| Endpoint | Purpose | Cost |
|----------|---------|------|
| `POST /api/custom_generate` | Advanced mode — you supply lyrics + tags | 10 credits (2 songs) |
| `POST /api/generate` | Simple mode — Suno writes lyrics from description | 10 credits (2 songs) |
| `GET /api/get?ids={id}` | Poll for completion | free |
| `GET /api/get_limit` | Check credits remaining | free |

### `POST /api/custom_generate` — Full Payload

```json
{
  "prompt": "Indian classical raga... no vocals...",
  "tags": "indian classical, ambient, drone, instrumental, raga",
  "title": "Raga Name (#N) — style",
  "make_instrumental": true,
  "model": "chirp-v3-5",
  "wait_audio": false
}
```

> **Note on `model`:** The bridge default is `chirp-v3-5`. Suno's internal v5.5 model name
> may differ — when generating manually through the UI, the dropdown shows **v5.5**.
> Force it in API calls by passing `"model": "chirp-v4-5"` or whatever Suno exposes
> (check the response `model_name` field after a manual generation to confirm the string).

### Song Status Values (from polling)

| Status | Meaning |
|--------|---------|
| `submitted` | In queue |
| `queue` | Waiting for GPU slot |
| `streaming` | Generating (audio available incrementally) |
| `complete` | Done, `audio_url` is permanent |
| `error` | Failed — retry |

---

## Smoke Test (Single Raga)

```bash
# Test one raga end-to-end (requires env vars)
bun ts-engines/scripts/suno-smoke.ts 15 ambient 45

# Dry-run — just print the prompt, no API call
SUNO_SMOKE_DRY_RUN=1 bun ts-engines/scripts/suno-smoke.ts 15 ambient 45
```

## Bulk Generation (72 Melakartas)

```bash
# Full run, ambient style
bun ts-engines/scripts/suno-bulk-gen.ts ambient

# Specific range
bun ts-engines/scripts/suno-bulk-gen.ts ambient 1 36

# Traditional style, second half
bun ts-engines/scripts/suno-bulk-gen.ts traditional 37 72
```

### Tuning Env Vars

| Variable | Default | Purpose |
|----------|---------|---------|
| `SUNO_BATCH_SIZE` | `2` | Parallel submissions per batch |
| `SUNO_POLL_INTERVAL_MS` | `5000` | How often to poll for completion |
| `SUNO_TIMEOUT_MS` | `180000` | Max wait per song (3 min) |
| `MAX_CREDITS_PER_RUN` | `200` | Hard stop per script run |
| `MIN_CREDITS_TO_START` | `100` | Refuse to run if quota too low |
| `SUBMIT_STAGGER_MS` | `3000` | Delay between submissions (prevents CF 429) |
| `INTER_BATCH_DELAY_MS` | `30000` | Pause between batches |

---

## v5.5 Pro Model — What's New / Key Features

From the screenshot UI and Suno's public docs:

- **Sounds tab** — attach reference audio, instruments, or field recordings
- **+ Voice** — new in v5.5; clone/reuse a voice for consistent vocal character
- **+ Inspo** — reference another Suno song to inherit its sonic DNA
- **More Options** — exposes duration hints, BPM nudges, and continuation controls
- **Styles field now has tag pills** — click existing tags or type new ones; they auto-suggest
- **Better Indian classical fidelity** — v5.5 handles microtones (gamaka) more reliably than v3.5

### Credit Cost (Pro Plan)
- Each generation = **2 songs** = **10 credits**
- Manual UI: same cost as API
- Pro monthly: ~2,500 credits (~250 generations)

---

## Workflow: Manual → DB Pipeline

If you generate manually in the UI and want to persist to the DB:

1. Generate in Suno UI → copy the **Song ID** from the URL (`suno.com/song/{id}`)
2. Download the MP3
3. Upload to Supabase `raga-clips` bucket: `clips/{style}/{NN}-{songId}.mp3`
4. POST to the internal API:

```bash
curl -X POST https://selemene.tryambakam.space/internal/raga/clip \
  -H "Content-Type: application/json" \
  -H "x-internal-key: $INTERNAL_SERVICE_KEY" \
  -d '{
    "melakarta_num": 15,
    "style": "ambient",
    "suno_song_id": "PASTE_SONG_ID_HERE",
    "cdn_url": "https://YOUR_SUPABASE_URL/storage/v1/object/public/raga-clips/clips/ambient/15-SONGID.mp3",
    "duration_sec": 45,
    "status": "pending"
  }'
```

5. Admin approves it → status changes to `approved` → it serves from `/api/v1/raga/15/clip?style=ambient`
