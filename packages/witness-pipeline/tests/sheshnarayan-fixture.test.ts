// Operator birth-data fixture as the regression set for the 2026-09-27 pipeline changes:
// engine-facts injection, Jev pass gate (shadow), retry loop, and the mode matrix runner.
// Engine results are a production snapshot (selemene.tryambakam.space, 2026-09-27) for the
// birth data already present in tests/fixtures/humdes; no network is used here.
import { describe, it, expect, vi } from 'vitest';
import { readFileSync, mkdtempSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { IntegratedReadingOrchestrator, parseModeDoc, type SelemeneEngineOutput } from '../src/index.js';
import { buildEngineFactsBlock } from '../src/orchestrator/engine-facts.js';
import { createJevPassGate } from '../src/jev/pass-gate.js';
import { runFinalVerification } from '../src/orchestrator/final-verification.js';

const FIXTURE = resolve(__dirname, 'fixtures/sheshnarayan.engines.prod-2026-09-27.json');
const NAME = 'Sheshnarayan Cumbipuram Nateshan';
const engines = (): SelemeneEngineOutput[] => JSON.parse(readFileSync(FIXTURE, 'utf8'));
const modes = (f: string) => parseModeDoc(resolve(__dirname, `../modes/${f}`));

describe('operator fixture: engine snapshot', () => {
  it('has nine deterministic engines, none errored, with contract metadata', () => {
    const e = engines();
    expect(e.map((x) => x.engine_id).sort()).toEqual(['biorhythm', 'enneagram', 'gene-keys', 'human-design', 'numerology', 'panchanga', 'transits', 'vedic-clock', 'vimshottari']);
    expect(e.every((x) => !x._error && x.metadata?.backend && x.envelope_version)).toBe(true);
  });

  it('renders the known chart facts into the engine-facts block', () => {
    const block = buildEngineFactsBlock({ subjectNames: [NAME], subjectRoles: [{ role: 'subject', name: NAME }], engineResultsBySubject: [engines()] });
    for (const fact of [
      'Type Generator', 'Profile 2/4', 'Authority Sacral', 'Definition Split', 'Channels 43-23, 42-53',
      'Nakshatra Uttara Phalguni', 'Tithi Chaturthi (Shukla)', 'mahadasha Jupiter 2026-09-09→2042-09-09',
      "Life's Work 4/49", 'life path 5', 'Moon sign Virgo', 'Sade Sati not active',
    ]) expect(block).toContain(fact);
    expect(block).toContain('biorhythm [native-rust]: present (moment-bound; not a birth fact)');
  });
});

describe('operator fixture: orchestrator with engine facts and Jev shadow gate', () => {
  const jevClient = vi.fn(async (req: any) => {
    if (req.state.sentences) {
      return { model: 'jev-test', answers: Object.fromEntries(req.state.sentences.map((s: any) => [`s${s.i}`, { type: 'noul', noul: /will|likely/i.test(s.sentence) ? 0.9 : 0.1 }])) };
    }
    const predictive = /will|likely/i.test(req.state.section);
    return {
      model: 'jev-test',
      answers: {
        guardrail_clean: { type: 'noul', noul: predictive ? 0.2 : 0.9 },
        fact_grounding: { type: 'score', score: /Uttara Phalguni/.test(req.state.section) ? 2.9 : 0.5, legend: {}, probabilities: {}, confidence: 0.9 },
        register_fit: { type: 'choice', choice: 'l1_l3', probabilities: {}, confidence: 0.8 },
        ...(req.state.relationship_type ? { relationship_framing_ok: { type: 'noul', noul: 0.95 } } : {}),
      },
    };
  });

  it('birth-blueprint L1: every pass prompt carries the operator chart facts and grounding is judged high', async () => {
    const seen: string[] = [];
    const llm = vi.fn(async (_s: string, user: string) => { seen.push(user); return 'Born under Uttara Phalguni with Profile 2/4 Generator, Sacral authority, life path 5. Descriptive witness. No prediction.'; });
    const orch = new IntegratedReadingOrchestrator({ mode: modes('birth-blueprint.md'), llm, jevGate: createJevPassGate(jevClient as any, 'shadow') });
    const out = await orch.run({ subjectNames: [NAME], subjectRoles: [{ role: 'subject', name: NAME }], engineResultsBySubject: [engines()], consciousnessLevel: 1 });
    expect(seen.length).toBe(2);
    for (const p of seen) { expect(p).toContain('## Engine facts (deterministic, per subject)'); expect(p).toContain('Nakshatra Uttara Phalguni'); expect(p).toContain('Profile 2/4'); }
    expect(out.register).toBe('l1_l3');
    expect(out.jev_receipts?.length).toBe(2);
    expect(out.passes.every((p) => p.jev?.status === 'judged' && p.jev.verdicts?.grounding === 'pass' && p.jev.verdicts?.guardrail === 'pass')).toBe(true);
    expect(out.passes.every((p) => (p.rubric.chart_fidelity_score ?? 0) > 0)).toBe(true);
    expect(runFinalVerification({ passes: out.passes }).passed).toBe(true);
    expect(() => JSON.stringify(out)).not.toThrow();
  });

  it('integrated-reading L3 with a predictive first draft: retry quotes the flagged sentence and the revision is kept', async () => {
    const llm = vi.fn(async (_s: string, user: string) => user.includes('## Revision required')
      ? 'Revised. Uttara Phalguni Moon in Virgo and the Jupiter mahadasha are present in the chart. Descriptive witness. No prediction.'
      : 'Draft. Uttara Phalguni Moon in Virgo. The Jupiter mahadasha will likely bring expansion over sixteen years. No diagnosis.');
    const orch = new IntegratedReadingOrchestrator({ mode: modes('integrated-reading.md'), llm, jevGate: createJevPassGate(jevClient as any, 'shadow'), jevRetry: { maxRetries: 1 } });
    const out = await orch.run({ subjectNames: [NAME], subjectRoles: [{ role: 'subject', name: NAME }], engineResultsBySubject: [engines()], consciousnessLevel: 3 });
    expect(out.passes.length).toBe(3);
    for (const p of out.passes) {
      expect(p.jev?.chosen).toBe('revision');
      expect(p.jev?.original_guardrail_clean).toBe(0.2);
      expect(p.jev?.answers?.guardrail_clean).toBe(0.9);
      expect(p.jev?.retries?.[0].flagged_sentences.map((f) => f.sentence)).toEqual(['The Jupiter mahadasha will likely bring expansion over sixteen years.']);
      expect(p.output).toContain('Revised');
    }
    expect(out.assembled).not.toMatch(/will likely/);
    expect(JSON.parse(JSON.stringify(out)).passes[0].jev.retries[0].receipt.retries).toBeUndefined();
  });

  it('gate off adds nothing; gate without a client records skipped with the install command', async () => {
    const llm = vi.fn(async () => 'Descriptive witness with Uttara Phalguni. No prediction.');
    const off = await new IntegratedReadingOrchestrator({ mode: modes('birth-blueprint.md'), llm, jevGate: createJevPassGate(vi.fn() as any, 'off') })
      .run({ subjectNames: [NAME], engineResultsBySubject: [engines()], consciousnessLevel: 1 });
    expect(off.jev_receipts).toBeUndefined();
    const noKey = await new IntegratedReadingOrchestrator({ mode: modes('birth-blueprint.md'), llm, jevGate: createJevPassGate(null, 'shadow') })
      .run({ subjectNames: [NAME], engineResultsBySubject: [engines()], consciousnessLevel: 1 });
    expect(noKey.jev_receipts?.every((r) => r.status === 'skipped' && r.reason?.includes('npx skills add typesafe-ai/skills'))).toBe(true);
  });
});

describe('operator fixture: mode matrix runner (stub narrative, Jev off)', () => {
  it('runs solo and dyad modes from a manifest, including the migrated partner-synastry doc', () => {
    const dir = mkdtempSync(join(tmpdir(), 'wp-matrix-'));
    const enginesDir = join(dir, 'engines'); const out = join(dir, 'out');
    require('node:fs').mkdirSync(enginesDir);
    writeFileSync(join(enginesDir, 'operator.prod.json'), readFileSync(FIXTURE));
    const manifest = {
      engines_dir: enginesDir,
      runs: [
        { id: 'bb', mode_file: 'birth-blueprint.md', consciousness_level: 1, jev: 'off', subjects: [{ slug: 'operator', role: 'subject', name: NAME }], relationship_context: null },
        { id: 'l4', mode_file: 'integrated-reading-l4.md', consciousness_level: 4, jev: 'off', subjects: [{ slug: 'operator', role: 'subject', name: NAME }], relationship_context: null },
        { id: 'ps', mode_file: 'partner-synastry.md', consciousness_level: 2, jev: 'off', subjects: [{ slug: 'operator', role: 'partner', name: NAME }, { slug: 'operator', role: 'partner', name: 'B' }], relationship_context: { type: 'custom', mapping_goal: 'contract test', sensitivity_level: 'high' } },
      ],
    };
    const manifestPath = join(dir, 'manifest.json'); writeFileSync(manifestPath, JSON.stringify(manifest));
    execFileSync('npx', ['tsx', 'scripts/mode-matrix-runner.ts', manifestPath, out, '--stub'], { cwd: resolve(__dirname, '..'), stdio: 'pipe', timeout: 120_000 });
    const rows = JSON.parse(readFileSync(join(out, 'matrix-summary.json'), 'utf8'));
    expect(rows.map((r: any) => [r.id, r.status])).toEqual([['bb', 'ok'], ['l4', 'ok'], ['ps', 'ok']]);
    expect(rows[0].passes).toBe(2); expect(rows[0].register).toBe('l1_l3'); expect(rows[1].register).toBe('l4_l5');
    expect(rows[2].passes).toBe(4); expect(rows[2].header).toBe('yes');
    expect(existsSync(join(out, 'bb', 'reading.md'))).toBe(true);
    expect(readFileSync(join(out, 'matrix-summary.md'), 'utf8')).toContain('| bb |');
    rmSync(dir, { recursive: true, force: true });
  });
});
