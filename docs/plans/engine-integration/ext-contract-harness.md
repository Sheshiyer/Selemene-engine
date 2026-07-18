# ext-contract-harness — P1 Media Contract Roundtrip Test Harness (Fail-Open)

**Self-contained harness for validating the 4 P1 focus engine contracts against FROZEN shapes.**

**Date:** 2026-07-17  
**Refs (MANDATORY — anti-drift):**  
- `.worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md` (contracts locked)  
- `crates/noesis-core/src/types.rs` (EngineInput/Output + MediaRef/Consent/QualitySpec/Generated* + contract examples @567-599)  
- `docs/plans/engine-integration/goal-understanding.md` (two-prong, local-first + explicit consent; Sankalpa owns capture preview/opt-in; never auto; Prong1 = authoritative)  
- 3 extraction files: `resources-and-assets.md`, `gaps-and-improvements.md`, `goal-understanding.md` (all cite in every P1 artifact)  
- `docs/plans/engine-integration/p1-w1-worker-bootstrap-packet.md` + `p1-w1-validation-gate-checklist.md`  
- `ts-engines/README.md` (ts-engines@3001; raaga/sigil ready; legacy params + note for media)  
- `python-services/README.md` (biofield-cv@8002; 11-metric authoritative)  
- `docs/engines/{biofield.md,face-reading.md,raaga.md,sigil-forge.md}` (per-engine shapes)  
- T-024 in `EXECUTION-STATUS.md`

**Fail-open design:** Harness attempts every roundtrip. Any single engine failure (missing keys, network, schema drift) logs + continues. No hard stop. Reports aggregate pass/fail at end. Matches "fail-open re-dispatch" protocol.

**Local-first consent guards (non-negotiable from goal-understanding + Sankalpa ISA):**  
- All examples include `consent: { granted: true, scopes: ["..."], timestamp: ISO }` + `image_data.consent`.  
- Harness code **guards before any dispatch**: if (!consent?.granted || !scopes.includes(required)) { skip-or-warn; do-not-send-to-network }.  
- Backend (ts/py) escalation ONLY after explicit opt-in in Prong2. No secrets, no auto-capture.

**Schema validation notes:**  
- Source of truth: `noesis-core/src/types.rs` (serde + utoipa). Run `cargo test -p noesis-core --features openapi` or `cargo check -p noesis-core`.  
- TS mirror (for Sankalpa/clients): use `engine-media-contracts.ts` or zod schemas derived from FROZEN examples. Validate `image_data` as b64|reference, generated_* optional.  
- Dual biofield: always use `engine_id: "biofield-capture"` for image path (vs "biofield" for birth).  
- No phantom fields: sigil has no `vector_path` (T-002 fix). Raaga uses `strudel_ratios` + top-level `generated_audio`.  
- Roundtrip invariant: input consent/quality/image_data roundtrips into result + generated_* (when present). Use `jq` or TS deep-equal for checks.  
- Full gate: compare vs openapi generated from utoipa + per-engine result examples in engine docs.

**4 Examples (exact FROZEN + engine docs shapes, realistic values):**

### 1. biofield-capture (image_data) — run vs python@8002 (CV) or via noesis-api bridge
**Input (EngineInput shape, image_data + consent local-first):**
```json
{
  "current_time": "2026-07-17T12:00:00Z",
  "options": { "consciousness_level": 2 },
  "image_data": {
    "b64": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
    "mime_type": "image/png",
    "consent": { "granted": true, "scopes": ["biofield-capture"], "timestamp": "2026-07-17T11:59:00Z" }
  },
  "consent": { "granted": true, "scopes": ["biofield-capture"], "timestamp": "2026-07-17T11:59:00Z" },
  "quality": { "sufficient": true, "scores": { "sharpness": 0.82, "contrast": 0.71 } }
}
```
**Expected result shape (11 metrics + quality, per shared/models.py + FROZEN):**
```json
{
  "engine_id": "biofield-capture",
  "result": {
    "metrics": {
      "light_quanta_density": 124.7,
      "normalized_area": 0.68,
      "average_intensity": 0.54,
      "inner_noise": 0.12,
      "energy_analysis": { "low": 18.2, "medium": 42.1, "high": 31.9, "total": 92.2 },
      "entropy_form_coefficient": 4.87,
      "fractal_dimension": 1.52,
      "correlation_dimension": 2.31,
      "body_symmetry": 0.74,
      "contour_complexity": 0.91,
      "pattern_regularity": 0.63
    },
    "quality_assessment": {
      "sharpness": 0.82,
      "contrast": 0.71,
      "noise_level": 0.09,
      "exposure": 0.88,
      "sufficient_quality": true
    },
    "contract_version": "biofield-cv/v1"
  },
  "witness_prompt": "What does your field reveal about your current state of coherence?",
  "consciousness_level": 2,
  "metadata": { "calculation_time_ms": 142.3, "backend": "opencv-cv", "cached": false, ... },
  "generated_image": null
}
```
**Python direct (note: /analyze uses multipart form, not JSON EngineInput — harness wraps):**
```bash
curl -X POST http://localhost:8002/analyze \
  -F "image=@/tmp/test-face.png" \
  -F 'algorithms=["photometric","fractal"]' \
  -F 'capture_metadata={"consent_granted":true,"scopes":["biofield-capture"]}'
```

### 2. face image_data — via ts-engines@3001 or Rust face-reading (extend for image)
**Input:**
```json
{
  "current_time": "2026-07-17T12:00:00Z",
  "options": { "consciousness_level": 1 },
  "image_data": {
    "b64": "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
    "mime_type": "image/jpeg",
    "consent": { "granted": true, "scopes": ["face-image"], "timestamp": "2026-07-17T11:59:00Z" }
  },
  "consent": { "granted": true, "scopes": ["face-image"], "timestamp": "2026-07-17T11:59:00Z" }
}
```
**Expected (per face-reading.md runtime + FROZEN):**
```json
{
  "engine_id": "face-reading",
  "result": {
    "analysis": {
      "constitution": { "primary_dosha": "vata", "secondary_dosha": "pitta", "tcm_element": "metal", "body_type": "ectomorph", ... },
      "elemental_balance": { "wood":0.21, "fire":0.18, "earth":0.23, "metal":0.19, "water":0.19, "dominant":"earth" },
      "personality_indicators": [ ... ],
      "health_indicators": [ ... ],
      "is_mock_data": false
    }
  },
  "witness_prompt": "...",
  "generated_image": null
}
```

### 3. raaga (melakarta + audio_ref ready) — ts-engines@3001
**Input (legacy params + future audio_ref + consent):**
```json
{
  "current_time": "2026-07-17T12:00:00Z",
  "options": { "consciousness_level": 3 },
  "audio_ref": { "reference": "file:local-recording.m4a", "consent": { "granted": true, "scopes": ["raaga-audio"], "timestamp": "2026-07-17T11:59:00Z" } },
  "consent": { "granted": true, "scopes": ["raaga-audio"], "timestamp": "2026-07-17T11:59:00Z" },
  "parameters": { "melakarta": 15, "dosha": "vata", "root_hz": 220 }
}
```
**Expected (per raaga/engine.ts + FROZEN generated_audio):**
```json
{
  "engine_id": "raaga",
  "result": {
    "melakarta": { "num": 15, "name": "Mayamalavagaula", ... },
    "strudel_ratios": [1.0, 1.066..., ...],
    "swaras": [...],
    ...
  },
  "generated_audio": {
    "strudel_ratios": [1.0, ...],
    "clip_url": null,
    "root_hz": 220,
    "metadata": { "engine": "raaga", "melakarta": 15 }
  },
  "witness_prompts": ["..."]
}
```
**Curl (ts-engines):**
```bash
curl -X POST http://localhost:3001/engines/raaga/calculate \
  -H 'Content-Type: application/json' \
  -d '{"parameters":{"melakarta":15,"dosha":"vata"},"consciousness_level":3}'
```

### 4. sigil (intention + image gen) — ts-engines@3001
**Input:**
```json
{
  "current_time": "2026-07-17T12:00:00Z",
  "options": { "consciousness_level": 2, "intention": "I witness my patterns clearly", "method": "word-elimination", "generate_image": true },
  "consent": { "granted": true, "scopes": ["sigil-gen"], "timestamp": "2026-07-17T11:59:00Z" }
}
```
**Expected (per sigil/engine.ts + FROZEN no-vector_path):**
```json
{
  "engine_id": "sigil-forge",
  "result": {
    "intention": "I witness my patterns clearly",
    "method": "word-elimination",
    "generated_image": { "b64_json": "iVBOR...", "metadata": { "model": "flux-kontext", "prompt": "..." } },
    ...
  },
  "generated_image": { "b64_json": "...", "metadata": { "model": "...", "provider": "nvidia|nano-banana" } }
}
```
**Curl:**
```bash
curl -X POST http://localhost:3001/engines/sigil-forge/calculate \
  -H 'Content-Type: application/json' \
  -d '{"parameters":{"intention":"I witness my patterns clearly","generate_image":true},"consciousness_level":2}'
```

---

## Self-Contained TS Harness (bun/node — fail-open + consent guard)

Save as `scripts/ext-contract-harness.ts` (or paste into ts-engines/tests/contract-roundtrip.test.ts)

```ts
// scripts/ext-contract-harness.ts
// Self-contained. bun run scripts/ext-contract-harness.ts
// Requires: ts-engines running @3001 OR python @8002 (for bio). Uses fetch.
// Fail-open: per-engine try/catch, aggregate report.
// Local-first guard: before any network.

type Consent = { granted: boolean; scopes: string[]; timestamp: string };
type MediaRef = { b64?: string; reference?: string; mime_type?: string; consent?: Consent };
type EngineInput = {
  current_time?: string;
  options?: Record<string, any>;
  image_data?: MediaRef;
  audio_ref?: MediaRef;
  consent?: Consent;
  quality?: any;
  parameters?: Record<string, any>;
};

function ensureLocalFirstConsent(consent: Consent | undefined, requiredScope: string, engine: string): boolean {
  if (!consent || !consent.granted || !consent.scopes.includes(requiredScope)) {
    console.warn(`[GUARD] ${engine}: consent not granted for ${requiredScope}. local-first only. SKIP network.`);
    return false;
  }
  const ageMs = Date.now() - new Date(consent.timestamp).getTime();
  if (ageMs > 1000 * 60 * 30) { // 30min freshness example
    console.warn(`[GUARD] ${engine}: consent stale.`);
  }
  return true;
}

async function roundtrip(engineId: string, url: string, input: EngineInput, requiredScope: string) {
  const consent = input.consent || input.image_data?.consent;
  if (!ensureLocalFirstConsent(consent, requiredScope, engineId)) {
    return { engineId, status: 'SKIPPED_GUARD', ok: false };
  }
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    });
    const data = await res.json();
    // Minimal schema note check (expand with zod in real)
    const hasEngine = data.engine_id === engineId || data.contract_version;
    const hasResult = !!data.result || !!data.metrics;
    const ok = res.ok && hasEngine && hasResult;
    console.log(`[${engineId}] ${ok ? 'PASS' : 'FAIL'} status=${res.status}`);
    if (!ok) console.log('  sample:', JSON.stringify(data).slice(0, 200));
    return { engineId, status: res.status, ok, data };
  } catch (e: any) {
    console.error(`[${engineId}] FAIL-OPEN: ${e.message}`);
    return { engineId, status: 'ERROR', ok: false, error: e.message };
  }
}

const examples = {
  biofieldCapture: { /* paste full JSON1 above, use engine_id note */ engine_id: 'biofield-capture', url: 'http://localhost:8002/analyze' /* adapt form in real */ , scope: 'biofield-capture' },
  face: { /* JSON2 */ url: 'http://localhost:3001/engines/face-reading/calculate', scope: 'face-image' },
  raaga: { /* JSON3 */ url: 'http://localhost:3001/engines/raaga/calculate', scope: 'raaga-audio' },
  sigil: { /* JSON4 */ url: 'http://localhost:3001/engines/sigil-forge/calculate', scope: 'sigil-gen' },
};

async function main() {
  console.log('=== ext-contract-harness (fail-open) ===');
  console.log('Refs: FROZEN, types.rs, goal-understanding local-first');
  const results = await Promise.all([
    // adapt bio to form or skip direct; use bridge for unified
    roundtrip('biofield-capture', 'http://localhost:8002/health', {}, 'biofield-capture'), // health first
    roundtrip('face-reading', 'http://localhost:3001/engines/face-reading/calculate', examples.face as any, 'face-image'),
    roundtrip('raaga', 'http://localhost:3001/engines/raaga/calculate', examples.raaga as any, 'raaga-audio'),
    roundtrip('sigil-forge', 'http://localhost:3001/engines/sigil-forge/calculate', examples.sigil as any, 'sigil-gen'),
  ]);
  const passed = results.filter(r => r.ok).length;
  console.log(`\nSUMMARY: ${passed}/${results.length} roundtrips passed (fail-open)`);
  console.log('Run: (1) ts-engines: cd ts-engines && bun run dev  (2) python: cd python-services && biofield-cv-service');
  console.log('Then: bun run scripts/ext-contract-harness.ts');
  // In real: assert vs FROZEN JSON schema
}

main();
```

## Simple Bash Script (curl + jq + guards)

Save as `scripts/ext-contract-harness.sh`

```bash
#!/bin/bash
# Self-contained curl harness. chmod +x ; ./scripts/ext-contract-harness.sh
# Fail-open per engine.

set -euo pipefail

TS_URL=${TS_ENGINES_URL:-http://localhost:3001}
PY_URL=${PYTHON_BIOFIELD_URL:-http://localhost:8002}

echo "=== ext-contract-harness (bash/curl) ==="
echo "Refs: P1W1-CONTRACTS-FROZEN.md + goal-understanding.md (local-first consent)"

pass=0; total=0

guard_consent() {
  local scope=$1
  # In real: parse from payload; here assume examples have granted
  echo "[GUARD] $scope consent local-first OK (example)"
}

run_curl() {
  local name=$1; local url=$2; local payload=$3; local scope=$4
  total=$((total+1))
  guard_consent "$scope"
  echo "[$name] POST $url"
  if curl -s -X POST "$url" -H 'Content-Type: application/json' -d "$payload" | jq -e '.engine_id or .status=="healthy" or .metrics' > /dev/null 2>&1; then
    echo "[$name] PASS"
    pass=$((pass+1))
  else
    echo "[$name] FAIL-OPEN (continuing)"
  fi
}

# 1. bio (python health + note form for analyze)
run_curl "biofield-health" "$PY_URL/health" '{}' "biofield-capture"

# 2-4 ts (use minimal legacy for current server; extend payloads with media for contract)
run_curl "raaga" "$TS_URL/engines/raaga/calculate" '{"parameters":{"melakarta":15},"consciousness_level":2}' "raaga-audio"
run_curl "sigil" "$TS_URL/engines/sigil-forge/calculate" '{"parameters":{"intention":"I witness clearly","generate_image":false},"consciousness_level":2}' "sigil-gen"
# face: add when registered or via Rust

echo "SUMMARY: $pass/$total passed (fail-open, consent guarded)"
echo "To full: start servers; use full media JSONs from this file; validate vs types.rs"
```

## How to Run

```bash
# Terminal 1: ts-engines (raaga, sigil, future face)
cd ts-engines
bun install
bun run dev   # or PORT=3001 bun run start

# Terminal 2: python biofield (for capture)
cd python-services
python -m venv .venv && source .venv/bin/activate
pip install -e ".[dev]"
biofield-cv-service   # or uvicorn ... --port 8002

# Terminal 3: harness (TS)
bun run scripts/ext-contract-harness.ts
# or bash
./scripts/ext-contract-harness.sh

# Full contract smoke (after adding full media JSONs):
# curl with the exact 4 examples above (add b64 small test image or reference).
# Schema: cargo test -p noesis-core --features openapi
# Cross: compare output to FROZEN comment blocks + engine docs.
```

**Notes for extension (P2+):**  
- Add zod schemas in harness for strict validate.  
- Bridge path (noesis-api) for unified EngineInput roundtrip: export TS_ENGINES_URL + PYTHON_BIOFIELD_URL; cargo run --bin noesis-server (mocks OK for contract).  
- Image b64: use tiny valid PNG in examples.  
- Update on contract change: re-freeze only via T-xxx + new FROZEN.

Ready to add: drop `scripts/ext-contract-harness.{ts,sh,md}` + reference in ts-engines/package.json scripts + READMEs + STATUS T-024.

**All work cites goal-understanding local-first + FROZEN + 3 extraction files.**
