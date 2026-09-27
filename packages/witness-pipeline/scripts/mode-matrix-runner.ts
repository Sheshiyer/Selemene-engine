/**
 * Mode matrix runner: drive every witness mode from one manifest with real engine data.
 *
 * Usage: tsx scripts/mode-matrix-runner.ts <manifest.json> <out-dir> [--only id,id] [--stub]
 *
 * Manifest: { engines_dir, runs: [{ id, mode_file, report_level?, consciousness_level, language?,
 *   subjects: [{ slug, role, name, label? }], relationship_context?, jev: off|shadow|active, jev_retry?: number }] }
 * Engine results are read from <engines_dir>/<slug>.prod.json.
 */
import { promises as fs } from 'node:fs';
import { join, resolve } from 'node:path';
import { IntegratedReadingOrchestrator, parseModeDoc, type SelemeneEngineOutput } from '../src/index.js';
import { runFinalVerification } from '../src/orchestrator/final-verification.js';
import { createJevClient, loadTypesafeKey } from '../src/jev/client.js';
import { createJevPassGate, type JevGateMode } from '../src/jev/pass-gate.js';
import { createLlmCall } from './lib/multi-llm.js';

const [MANIFEST, OUT] = process.argv.slice(2);
const STUB = process.argv.includes('--stub');
const onlyIdx = process.argv.indexOf('--only');
const ONLY = onlyIdx > -1 ? new Set(process.argv[onlyIdx + 1].split(',')) : null;
if (!MANIFEST || !OUT) { console.error('Usage: tsx scripts/mode-matrix-runner.ts <manifest.json> <out-dir> [--only a,b] [--stub]'); process.exit(1); }

interface RunSpec {
  id: string; mode_file: string; report_level?: string; consciousness_level: number; language?: string;
  subjects: Array<{ slug: string; role: string; name: string; label?: string }>;
  relationship_context?: { type: string; mapping_goal: string; sensitivity_level: 'low' | 'medium' | 'high' } | null;
  jev?: JevGateMode; jev_retry?: number;
}

async function main() {
  const manifest = JSON.parse(await fs.readFile(MANIFEST, 'utf8')) as { engines_dir: string; runs: RunSpec[] };
  await fs.mkdir(OUT, { recursive: true });
  const key = await loadTypesafeKey();
  const rows: any[] = [];
  const llmLive = createLlmCall({ temperature: 0.7, timeout_ms: 240_000 });

  for (const run of manifest.runs) {
    if (ONLY && !ONLY.has(run.id)) continue;
    const dir = join(OUT, run.id);
    await fs.mkdir(dir, { recursive: true });
    const started = Date.now();
    const row: any = { id: run.id, mode: run.mode_file, subjects: run.subjects.map((s) => s.name), relationship: run.relationship_context?.type ?? 'solo', level: run.consciousness_level, jev: run.jev ?? 'shadow', jev_retry: run.jev_retry ?? 0 };
    try {
      const mode = parseModeDoc(resolve(process.cwd(), 'modes', run.mode_file));
      const engineResultsBySubject: SelemeneEngineOutput[][] = [];
      for (const s of run.subjects) engineResultsBySubject.push(JSON.parse(await fs.readFile(join(manifest.engines_dir, `${s.slug}.prod.json`), 'utf8')));
      const jevClient = (run.jev ?? 'shadow') === 'off' ? null : createJevClient(key);
      const gate = createJevPassGate(jevClient, run.jev ?? 'shadow');
      const llm = STUB ? stubLlm : llmLive;
      const orch = new IntegratedReadingOrchestrator({ mode, llm, jevGate: gate, jevRetry: { maxRetries: run.jev_retry ?? 0 } });
      const out = await orch.run({
        subjectNames: run.subjects.map((s) => s.name),
        subjectRoles: run.subjects.map((s) => ({ role: s.role, name: s.name, label: s.label })),
        relationshipContext: run.relationship_context ?? undefined,
        language: run.language ?? 'en',
        consciousnessLevel: run.consciousness_level,
        engineResultsBySubject,
      });
      const verification = runFinalVerification({ passes: out.passes });
      await fs.writeFile(join(dir, 'reading.md'), out.assembled, 'utf8');
      await fs.writeFile(join(dir, 'result.json'), JSON.stringify({ ...out, passes: out.passes.map((p) => ({ id: p.id, title: p.title, rubric: p.rubric, jev: p.jev })) }, null, 2), 'utf8');
      if (out.jev_receipts) await fs.writeFile(join(dir, 'jev-receipts.json'), JSON.stringify(out.jev_receipts, null, 2), 'utf8');
      const receipts = out.jev_receipts ?? [];
      const judged = receipts.filter((r) => r.status === 'judged');
      Object.assign(row, {
        status: 'ok',
        passes: out.passes.length,
        words: out.assembled.split(/\s+/).filter(Boolean).length,
        target: `${mode.frontmatter.target_words.min}-${mode.frontmatter.target_words.max}`,
        register: out.register,
        header: out.relationship_header ? 'yes' : 'n/a',
        verification: verification.passed ? 'PASS' : `FAIL ${verification.blockers.join(',')}`,
        rubric: {
          word_fit: out.passes.filter((p) => p.rubric.word_count_fit === 'pass').length,
          facts: out.passes.filter((p) => p.rubric.deterministic_fact_gate === 'pass').length,
          layers: out.passes.filter((p) => p.rubric.integrated_layering_gate === 'pass').length,
          guard: out.passes.filter((p) => p.rubric.guardrail_gate === 'pass').length,
        },
        jev_summary: {
          judged: judged.length, skipped: receipts.filter((r) => r.status === 'skipped').length, errors: receipts.filter((r) => r.status === 'error').length,
          guardrail: judged.map((r) => r.verdicts!.guardrail), framing: judged.map((r) => r.verdicts!.framing), grounding: judged.map((r) => r.verdicts!.grounding), register: judged.map((r) => `${r.answers!.register_fit.choice}@${r.answers!.register_fit.confidence.toFixed(2)}`),
          guardrail_clean: judged.map((r) => r.answers!.guardrail_clean.toFixed(2)),
          retries: receipts.filter((r) => r.retries?.length).length,
          revisions_accepted: receipts.filter((r) => r.chosen === 'revision').length,
          original_guardrail_clean: receipts.filter((r) => r.retries?.length).map((r) => `${r.pass_id}:${r.original_guardrail_clean?.toFixed(2)}→${r.answers?.guardrail_clean.toFixed(2)}`),
          disagreements: receipts.flatMap((r) => r.disagreements),
        },
        seconds: Math.round((Date.now() - started) / 1000),
      });
    } catch (e: any) {
      Object.assign(row, { status: 'error', error: e.message.slice(0, 300), seconds: Math.round((Date.now() - started) / 1000) });
    }
    rows.push(row);
    console.log(JSON.stringify(row));
    await fs.writeFile(join(OUT, 'matrix-summary.json'), JSON.stringify(rows, null, 2), 'utf8');
    await fs.writeFile(join(OUT, 'matrix-summary.md'), renderTable(rows), 'utf8');
  }
}

function renderTable(rows: any[]): string {
  const lines = ['| id | mode | subjects | level/reg | passes | words (target) | rubric w/f/l/g | Jev guardrail (clean) | framing | grounding | register@conf | retries acc | disagreements | verification | s |', '|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|'];
  for (const r of rows) {
    if (r.status !== 'ok') { lines.push(`| ${r.id} | ${r.mode} | ${r.subjects.join(', ')} | ${r.level} | error | | | | | | | | ${r.error} | | ${r.seconds} |`); continue; }
    const j = r.jev_summary;
    lines.push(`| ${r.id} | ${r.mode} | ${r.subjects.join(', ')} | ${r.level}/${r.register} | ${r.passes} | ${r.words} (${r.target}) | ${r.rubric.word_fit}/${r.rubric.facts}/${r.rubric.layers}/${r.rubric.guard} | ${j.guardrail.map((v: string, i: number) => `${v}(${j.guardrail_clean[i]})`).join(', ')} | ${j.framing.join(', ')} | ${j.grounding.join(', ')} | ${j.register.join(', ')} | ${j.retries}/${j.revisions_accepted} ${j.original_guardrail_clean.join(' ')} | ${j.disagreements.length} | ${r.verification} | ${r.seconds} |`);
  }
  return lines.join('\n') + '\n';
}

async function stubLlm(_s: string, user: string, opts: { max_tokens: number }): Promise<string> {
  const target = Math.max(120, Math.floor((opts.max_tokens || 600) / 2.4));
  const revised = user.includes('## Revision required');
  const filler = `${revised ? 'Revised descriptive witness.' : 'Descriptive witness.'} Vedic Lagna nakshatra dasha Human Design gate channel profile authority Gene Keys Life's Work transit panchanga tithi. No prediction. No diagnosis. `;
  let out = ''; while (out.split(/\s+/).length < target) out += filler; return out;
}

main().catch((e) => { console.error(e); process.exit(1); });
