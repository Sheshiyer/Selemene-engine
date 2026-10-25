/** Reproducible continuation of the eight-report set; no credentials in logs. */
import { resolve, join } from 'node:path';
import { mkdir, writeFile, readFile } from 'node:fs/promises';
import { loadTypesafeKey } from '../../packages/witness-pipeline/src/jev/client.js';
const args = process.argv.slice(2);
const value = (flag: string, fallback?: string) => {
  const i = args.indexOf(flag);
  return i < 0 ? fallback : args[i + 1];
};
const root = resolve(value('--package-dir', 'tools/humdes-extractor/output/gary-abitbol-2026-09-28')!);
const output = resolve(value('--out', join(root, 'completion-2026-10-04'))!);
await mkdir(output, { recursive: true });
const previous = JSON.parse(await readFile(join(root, 'reports-reference/runs/sheshnarayan-en-2026-09-30T12-36-41-933Z/inputs.json'), 'utf8'));
const account = process.env.CLOUDFLARE_ACCOUNT_ID ?? previous.accountId;
const routes = [value('--combo', 'noesis-execute')!, value('--audit-combo', 'noesis-verify')!];
const key = process.env.OMNIROUTE_API_KEY;
const readiness = {
  checkedAt: new Date().toISOString(),
  reportKeyPresent: Boolean(key), jevKeyPresent: Boolean(await loadTypesafeKey()),
  wranglerPresent: Boolean(Bun.which('wrangler')),
  cloudflareAccountPresent: /^[a-f0-9]{32}$/.test(account ?? ''),
  routes,
  routeProbes: [] as Array<{ route: string; status?: number; ready: boolean; error?: string }>,
};
for (const route of routes) {
  try {
    const response = await fetch('http://127.0.0.1:20128/v1/chat/completions', {
      method: 'POST', headers: { 'Content-Type': 'application/json', ...(key ? { Authorization: `Bearer ${key}` } : {}) },
      body: JSON.stringify({ model: route, messages: [{ role: 'user', content: 'Reply with READY.' }], max_tokens: 64 }),
      signal: AbortSignal.timeout(30000),
    });
    readiness.routeProbes.push({ route, status: response.status, ready: response.ok });
    await response.body?.cancel();
  } catch (error) {
    readiness.routeProbes.push({ route, ready: false, error: error instanceof Error ? error.name : 'transport-error' });
  }
}
await writeFile(join(output, 'runtime-readiness.json'), JSON.stringify(readiness, null, 2) + '\n');
console.log(JSON.stringify(readiness));
if (!args.includes('--execute')) process.exit(0);
if (!readiness.reportKeyPresent || !readiness.jevKeyPresent || !readiness.wranglerPresent ||
    !readiness.cloudflareAccountPresent || readiness.routeProbes.some(p => !p.ready)) {
  throw new Error('Live report prerequisites unavailable; restore the configured environment. No report generation started.');
}
const jobs = ['sheshnarayan', 'gary-abitbol', 'mohan-kumar', 'business-partners-triad']
  .flatMap(subject => ['en', 'fr'].map(language => ({ subject, language })));
const selected = value('--only');
const runJobs = selected ? jobs.filter(j => `${j.subject}-${j.language}` === selected) : jobs;
if (!runJobs.length) throw new Error('Unknown --only report');
const results: unknown[] = [];
for (const job of runJobs) {
  const runner = job.subject === 'business-partners-triad' ? 'run-reference-triad.mts' : 'run-reference-report.mts';
  const command = [process.execPath, join(import.meta.dir, runner), '--package-dir', root,
    '--language', job.language, '--account-id', account,
    '--primary-registry', join(root, 'reports-reference/editorial-primary-only-registry.json'),
    '--combo', routes[0], '--audit-combo', routes[1], '--serial-voices',
    ...(job.subject === 'business-partners-triad' ? [] : ['--subject', job.subject])];
  const log = join(output, `${job.subject}-${job.language}.generation.log`);
  console.log(JSON.stringify({ stage: 'start', ...job, log }));
  const processRun = Bun.spawn(command, { stdout: Bun.file(log), stderr: 'pipe', env: { ...process.env, CLOUDFLARE_ACCOUNT_ID: account } });
  const stderr = await new Response(processRun.stderr).text();
  const exitCode = await processRun.exited;
  await writeFile(join(output, `${job.subject}-${job.language}.stderr.log`), stderr);
  results.push({ ...job, exitCode, log, generatedAt: new Date().toISOString(), finalAccepted: false });
  await writeFile(join(output, 'generation-matrix.json'), JSON.stringify(results, null, 2) + '\n');
  console.log(JSON.stringify({ stage: 'generation-finished', ...job, exitCode, finalAccepted: false }));
  if (exitCode) throw new Error(`${job.subject}-${job.language} stopped; preserve and inspect its checkpoint before continuing.`);
}
console.log('Generation stage complete. Source/coherence/parity and all-page render review remain required.');
