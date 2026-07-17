#!/usr/bin/env bun
// scripts/ext-contract-harness.ts
// T-024 harness integration (MAIN, post-P2-merge): 4-engine roundtrips using FROZEN contracts (image_data, consent, generated_*)
// phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend
// Uses exact FROZEN from P1W1-CONTRACTS-FROZEN.md + merged P2 server schema (ts-engines/src/server/app.ts:196-207:
//   consciousness_level + parameters required; image_data/audio_ref/consent/quality top-level optional)
// Fail-open + local-first consent guards (goal-understanding.md invariant: explicit opt-in before network; Prong2 Sankalpa owns)
// Refs: ext-contract-harness.md , p1-w1-worker-bootstrap-packet.md , resources-and-assets.md , gaps-and-improvements.md ,
//   goal-understanding.md , EXECUTION-STATUS (T-024 + full-p2-checklist-live-roundtrips), detailed-task-list T-024 ,
//   .worktrees/T-002-copilot/docs/plans/engine-integration/P1W1-CONTRACTS-FROZEN.md , P1W2-HANDOFF.md , ext-p2-validation-checklist.md
// Run: (start ts-engines@3001 + python@8002) ; bun run scripts/ext-contract-harness.ts
// NOTE: face-reading is a Rust engine (crates/engine-face-reading), NOT registered on the TS server
//   (health shows tarot/i-ching/enneagram/sacred-geometry/sigil-forge/raaga). Its roundtrip is fail-open here;
//   live FROZEN evidence for face is cargo test -p engine-face-reading (test_calculate_with_frozen_image_data_consent_sample).

type Consent = { granted: boolean; scopes: string[]; timestamp: string; token?: string };
type MediaRef = { b64?: string; reference?: string; mime_type?: string; consent?: Consent };
type EngineInput = {
  consciousness_level: number;
  parameters: Record<string, any>;
  seed?: number;
  question?: string;
  image_data?: MediaRef;
  audio_ref?: MediaRef;
  consent?: Consent;
  quality?: any;
};

const FROZEN_TS_URL = process.env.TS_ENGINES_URL || 'http://localhost:3001';
const FROZEN_PY_URL = process.env.PYTHON_BIOFIELD_URL || 'http://localhost:8002';

// tiny valid 1x1 png b64 (FROZEN sample)
const TINY_PNG_B64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';

function ensureLocalFirstConsent(consent: Consent | undefined, requiredScope: string, engine: string): boolean {
  if (!consent || !consent.granted || !consent.scopes.includes(requiredScope)) {
    console.warn(`[GUARD][wave:integration-w2] ${engine}: consent not granted for ${requiredScope}. local-first only. SKIP network.`);
    return false;
  }
  const ageMs = Date.now() - new Date(consent.timestamp).getTime();
  if (ageMs > 30 * 60 * 1000) {
    console.warn(`[GUARD] ${engine}: consent stale (>30m).`);
  }
  return true;
}

async function roundtrip(engineId: string, url: string, input: EngineInput, requiredScope: string, note: string) {
  const consent = input.consent || input.image_data?.consent || input.audio_ref?.consent;
  if (!ensureLocalFirstConsent(consent, requiredScope, engineId)) {
    return { engineId, status: 'SKIPPED_GUARD', ok: false, note };
  }
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(input),
    });
    const data = await res.json();
    const hasEngine = (data.engine_id === engineId) || data.status === 'healthy' || !!data.result || !!data.metrics;
    const hasGenerated = !!data.generated_image || !!data.generated_audio || !!data.result?.generated_image || !!data.result?.generated_audio;
    const ok = res.ok && hasEngine;
    console.log(`[${engineId}] ${ok ? 'PASS' : 'FAIL-OPEN'} status=${res.status} ${note}${hasGenerated ? ' (generated_* present)' : ''}`);
    if (!ok) console.log('  sample-out:', JSON.stringify(data).slice(0, 280));
    return { engineId, status: res.status, ok, hasGenerated, note };
  } catch (e: any) {
    console.error(`[${engineId}] FAIL-OPEN: ${e.message} ${note}`);
    return { engineId, status: 'ERROR', ok: false, error: e.message, note };
  }
}

async function roundtripBiofieldCapture(consent: Consent, note: string) {
  if (!ensureLocalFirstConsent(consent, 'biofield-capture', 'biofield-capture')) {
    return { engineId: 'biofield-capture', status: 'SKIPPED_GUARD', ok: false, note };
  }
  try {
    const png = Uint8Array.from(atob(TINY_PNG_B64), (c) => c.charCodeAt(0));
    const form = new FormData();
    form.append('image', new Blob([png], { type: 'image/png' }), 'tiny.png');
    form.append('capture_metadata', JSON.stringify({ consent, source: 'ext-contract-harness', consciousness_level: 2 }));
    const res = await fetch(`${FROZEN_PY_URL}/analyze`, { method: 'POST', body: form });
    const data = await res.json();
    const metricKeys = data.metrics ? Object.keys(data.metrics) : [];
    const ok = res.ok && !!data.metrics && metricKeys.length >= 11;
    console.log(`[biofield-capture] ${ok ? 'PASS' : 'FAIL-OPEN'} status=${res.status} ${note} (metrics=${metricKeys.length})`);
    if (!ok) console.log('  sample-out:', JSON.stringify(data).slice(0, 280));
    return { engineId: 'biofield-capture', status: res.status, ok, metrics: metricKeys.length, note };
  } catch (e: any) {
    console.error(`[biofield-capture] FAIL-OPEN: ${e.message} ${note}`);
    return { engineId: 'biofield-capture', status: 'ERROR', ok: false, error: e.message, note };
  }
}

async function main() {
  console.log('=== T-024 ext-contract-harness (MAIN, fail-open, FROZEN contracts, post-P2-merge) ===');
  console.log('phase:integration-p1 wave:integration-w2 area:engine-integration swarm:selemene-backend');
  console.log('Refs: P1W1-CONTRACTS-FROZEN.md + goal-understanding.md (local-first) + ext-contract-harness.md + 3 extraction + EXECUTION-STATUS');
  console.log(`TS: ${FROZEN_TS_URL} | PY: ${FROZEN_PY_URL}`);

  const now = new Date().toISOString();
  const consentBio: Consent = { granted: true, scopes: ['biofield-capture'], timestamp: now };
  const consentFace: Consent = { granted: true, scopes: ['face-image'], timestamp: now };
  const consentRaaga: Consent = { granted: true, scopes: ['raaga-audio'], timestamp: now };
  const consentSigil: Consent = { granted: true, scopes: ['sigil-gen'], timestamp: now };

  // 1. biofield-capture (python sidecar /analyze multipart; FROZEN image_data + consent in capture_metadata)
  const bioRes = await roundtripBiofieldCapture(consentBio, 'FROZEN image_data+consent via /analyze (11-metric capture contract)');

  // 2. face (FROZEN image_data + consent; Rust engine not on TS server -> fail-open expected)
  const faceInput: EngineInput = {
    consciousness_level: 1,
    parameters: {},
    image_data: { b64: TINY_PNG_B64, mime_type: 'image/png', consent: consentFace },
    consent: consentFace,
  };
  const faceRes = await roundtrip('face-reading', `${FROZEN_TS_URL}/engines/face-reading/calculate`, faceInput, 'face-image', 'FROZEN image_data+consent (Rust engine; not registered on TS server — see cargo test)');

  // 3. raaga (melakarta + dosha + audio_ref per T-005/T-031 FROZEN generated_audio)
  const raagaInput: EngineInput = {
    consciousness_level: 3,
    parameters: { melakarta: 1, dosha: 'vata', generated_audio: true },
    audio_ref: { reference: 'file:local.m4a', consent: consentRaaga },
    consent: consentRaaga,
  };
  const raagaRes = await roundtrip('raaga', `${FROZEN_TS_URL}/engines/raaga/calculate`, raagaInput, 'raaga-audio', 'FROZEN strudel_ratios + generated_audio');

  // 4. sigil (intention + provider; generated_image via abstraction T-003/T-035; no vector_path)
  const sigilInput: EngineInput = {
    consciousness_level: 2,
    parameters: { intention: 'I witness my patterns clearly', method: 'word-elimination', generate_image: true, image_style: 'runic' },
    consent: consentSigil,
  };
  const sigilRes = await roundtrip('sigil-forge', `${FROZEN_TS_URL}/engines/sigil-forge/calculate`, sigilInput, 'sigil-gen', 'FROZEN generated_image (provider abstraction)');

  const results = [bioRes, faceRes, raagaRes, sigilRes];
  const passed = results.filter((r) => r.ok).length;
  console.log(`\nSUMMARY T-024: ${passed}/${results.length} roundtrips passed (fail-open, consent guarded)`);
  console.log('FROZEN shapes exercised: image_data, consent, generated_* (P1W1-CONTRACTS-FROZEN.md; types.rs:437+ in T-002 wt; ts mirror merged)');
  console.log('face-reading: Rust engine — live FROZEN evidence via cargo test -p engine-face-reading (T-027 merged).');
  console.log('To re-run full: start ts-engines@3001 + python@8002; bun run scripts/ext-contract-harness.ts');
}

main().catch(console.error);
