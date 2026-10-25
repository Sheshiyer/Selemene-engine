import { promises as fs } from 'node:fs';
import { basename, dirname, isAbsolute, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, '../..');
const argv = process.argv.slice(2);
const value = (key: string, fallback?: string) => {
  const i = argv.indexOf(key);
  return i >= 0 ? argv[i + 1] : fallback;
};
const has = (flag: string) => argv.includes(flag);
// Reference execution is a separate, checkpointed route using real corpus reads.
if (has('--reference') || has('--reference-preflight') || has('--preflight')) {
  if (has('--subjects') || has('--languages')) throw new Error('Reference route currently takes --subject and --language for one checkpointed run.');
  if (has('--reference-preflight') && !has('--preflight')) process.argv.push('--preflight');
  await import('./run-reference-report.mjs');
  process.exit(0);
}

const packageArg = value('--package-dir');
if (!packageArg) throw new Error('Usage: run-integrated-report.mts --package-dir <HumDes output package> [--subjects gary-abitbol,mohan-kumar,sheshnarayan] [--languages en,fr]');

const packageDir = isAbsolute(packageArg) ? packageArg : resolve(repo, packageArg);
const people = (value('--subjects', 'gary-abitbol,mohan-kumar,sheshnarayan') ?? '').split(',').filter(Boolean);
const languages = (value('--languages', 'en,fr') ?? '').split(',').filter(Boolean);
const includeTriad = !has('--no-triad');
const jev = value('--jev', 'shadow')!;
const jevRetry = Number(value('--jev-retry', '1'));
const selectedIds = value('--only')?.split(',').filter(Boolean);
const allowed = new Set(['gary-abitbol', 'mohan-kumar', 'sheshnarayan']);
if (people.length !== 3 || people.some((p) => !allowed.has(p)) || new Set(people).size !== 3) {
  throw new Error('This triad workflow requires exactly gary-abitbol, mohan-kumar, and sheshnarayan.');
}
if (languages.length === 0 || languages.some((l) => !['en', 'fr'].includes(l))) throw new Error('Supported languages: en,fr');
if (jev !== 'shadow' && jev !== 'off') throw new Error('This report command allows Jev shadow or off; active mode is reserved for a separately approved calibration change.');
if (!Number.isInteger(jevRetry) || jevRetry < 0 || jevRetry > 2) throw new Error('--jev-retry must be 0, 1, or 2.');

const enginesDir = resolve(packageDir, 'jev-shadow/engines');
const outputDir = resolve(packageDir, 'reports-12');
await fs.mkdir(outputDir, { recursive: true });
for (const slug of people) await fs.access(join(enginesDir, `${slug}.prod.json`));

const enTitles: Record<string, string> = {
  opening: 'Opening and scope', 'source-map': 'Source map and limits', 'chart-overview': 'Identity stack and chart overview',
  'type-profile': 'Type, profile, authority and definition', 'centers-channels': 'Centers and complete channels',
  'gates-cross': 'Gate inventory and incarnation cross', 'gene-keys-overlay': 'Attributed Gene Keys overlay',
  panchanga: 'Birth Panchanga', vimshottari: 'Vimshottari calculation snapshot', numerology: 'Numerology and supporting systems',
  integration: 'Integrated patterns and differences', synthesis: 'Synthesis and reflection',
  'signatures': 'Three individual chart signatures', mechanics: 'Types, profiles, authorities and definitions',
  'channels-centers': 'Centers and complete channels', 'pairwise-gates': 'Pairwise gates and cross references',
  'penta-source': 'Business Penta image evidence', timing: 'Timing snapshots across partners',
  'gene-keys-numbers': 'Gene Keys and numerology overlays', 'collaboration-patterns': 'Present collaboration patterns',
  questions: 'Partner conversation questions',
};
const frTitles: Record<string, string> = {
  opening: 'Ouverture et périmètre', 'source-map': 'Sources et limites', 'chart-overview': 'Empilement identitaire et vue d’ensemble du thème',
  'type-profile': 'Type, profil, autorité et définition', 'centers-channels': 'Centres et canaux complets',
  'gates-cross': 'Inventaire des portes et croix d’incarnation', 'gene-keys-overlay': 'Calque Gene Keys attribué à la source',
  panchanga: 'Panchanga de naissance', vimshottari: 'Instantané du calcul Vimshottari', numerology: 'Numérologie et systèmes complémentaires',
  integration: 'Motifs intégrés et différences', synthesis: 'Synthèse et réflexion',
  signatures: 'Signatures individuelles des trois thèmes', mechanics: 'Types, profils, autorités et définitions',
  'channels-centers': 'Centres et canaux complets', 'pairwise-gates': 'Portes et références croisées, deux à deux',
  'penta-source': 'Éléments de la capture Business Penta', timing: 'Instantanés temporels des partenaires',
  'gene-keys-numbers': 'Calques Gene Keys et numérologiques', 'collaboration-patterns': 'Motifs présents de collaboration',
  questions: 'Questions de dialogue entre partenaires',
};

const screenshotNote = join(packageDir, 'raw/business-penta-source.json');
const hasScreenshotNote = await fs.access(screenshotNote).then(() => true, () => false);
const runs: any[] = [];
for (const slug of people) {
  const name = slug === 'gary-abitbol' ? 'Gary Abitbol' : slug === 'mohan-kumar' ? 'Mohan Kumar' : 'Sheshnarayan Cumbipuram Nateshan';
  for (const language of languages) runs.push({
    id: `${slug}-${language}`,
    mode_file: 'human-design-integrated-12.md',
    report_level: 'L3', consciousness_level: 3, language,
    subjects: [{ slug, role: 'primary', name }],
    jev, jev_retry: jevRetry, source_pack: true,
    section_titles: language === 'fr' ? frTitles : enTitles,
    ...(slug === 'gary-abitbol' ? { source_notes: ['../humdes-extracted.json'] } : {}),
  });
}
if (includeTriad) for (const language of languages) runs.push({
  id: `business-partners-triad-${language}`,
  mode_file: 'business-partners-triad-12.md',
  report_level: 'L3', consciousness_level: 2, language,
  subjects: people.map((slug) => ({
    slug, role: 'business-partner',
    name: slug === 'gary-abitbol' ? 'Gary Abitbol' : slug === 'mohan-kumar' ? 'Mohan Kumar' : 'Sheshnarayan Cumbipuram Nateshan',
  })),
  relationship_context: {
    type: 'business-partners',
    mapping_goal: language === 'fr'
      ? 'Décrire les motifs symboliques présents entre les trois partenaires commerciaux déclarés, en respectant leur autonomie et sans prédire de résultat.'
      : 'Describe present symbolic patterns among the three declared business partners, preserving their agency and avoiding outcome predictions.',
    sensitivity_level: 'high',
  },
  jev, jev_retry: jevRetry, source_pack: true,
  section_titles: language === 'fr' ? frTitles : enTitles,
  ...(hasScreenshotNote ? { source_notes: ['../raw/business-penta-source.json'] } : {}),
});

const manifestPath = join(outputDir, 'manifest.json');
await fs.writeFile(manifestPath, JSON.stringify({ engines_dir: enginesDir, runs }, null, 2));

if (has('--dry-run')) {
  console.log(JSON.stringify({ package_dir: packageDir, output_dir: outputDir, runs: runs.map(({ id, mode_file, language, subjects: runSubjects, jev: jevMode, jev_retry }) => ({ id, mode_file, language, subjects: runSubjects.map((s: any) => s.name), jev: jevMode, jev_retry })), manifest: manifestPath }, null, 2));
  process.exit(0);
}
const runnerPath = resolve(repo, 'packages/witness-pipeline/scripts/mode-matrix-runner.ts');
const matrixOut = join(outputDir, 'matrix-runs');
if (has('--verify-only')) {
  const existing = JSON.parse(await fs.readFile(join(matrixOut, 'matrix-summary.json'), 'utf8')) as Array<Record<string, any>>;
  const needed = runs.map((run) => run.id);
  const missing = needed.filter((id) => !existing.some((row) => row.id === id));
  if (missing.length) throw new Error(`Verification is missing report runs: ${missing.join(', ')}.`);
  for (const row of existing.filter((candidate) => needed.includes(candidate.id))) {
    if (row.status !== 'ok' || row.passes !== 12 || row.verification !== 'PASS') throw new Error(`${row.id}: report matrix gate is not clean.`);
    const run = runs.find((candidate) => candidate.id === row.id)!;
    if (jev === 'shadow') {
      if (row.jev_summary?.judged !== 12 || row.jev_summary?.skipped || row.jev_summary?.errors) throw new Error(`${row.id}: Jev did not judge all twelve sections.`);
      const receipts = JSON.parse(await fs.readFile(join(matrixOut, row.id, 'jev-receipts.json'), 'utf8'));
      const failures = receipts.filter((receipt: any) => receipt.blocked || receipt.verdicts?.guardrail === 'fail' || receipt.verdicts?.framing === 'fail' || receipt.verdicts?.grounding === 'fail');
      if (failures.length) throw new Error(`${row.id}: Jev has ${failures.length} guardrail/framing/grounding failure(s); review before treating this report as final.`);
    }
    const readingPath = join(matrixOut, row.id, 'reading.md');
    const reading = await fs.readFile(readingPath, 'utf8');
    await verifySourceAnchors(run, reading);
    await verifyTemporalMentions(run, reading);
    const suffix = row.id.startsWith('business-partners-triad-') ? `business-partners-triad.${run.language}.md` : `individual-${row.id}.md`;
    if ((await fs.readFile(join(outputDir, suffix), 'utf8')) !== reading) throw new Error(`${row.id}: indexed report differs from the matrix reading.`);
    const sourceManifest = JSON.parse(await fs.readFile(join(matrixOut, row.id, 'source-pack', 'manifest.json'), 'utf8'));
    if (sourceManifest.language !== run.language || sourceManifest.report_level !== 'L3') throw new Error(`${row.id}: source-pack language/report level mismatch.`);
    const sourcePackReading = await fs.readFile(join(matrixOut, row.id, 'source-pack', 'reading.md'), 'utf8');
    const proseReading = reading.replace(/\n\n## (?:Source chart images|Images sources des thèmes)\n[\s\S]*$/, '').trimEnd() + '\n';
    if (sourceManifest.reading_length !== sourcePackReading.length) throw new Error(`${row.id}: source-pack reading length is stale.`);
    if (sourcePackReading.trimEnd() !== proseReading.trimEnd()) throw new Error(`${row.id}: source-pack prose differs from the Jev-judged report text.`);
    await verifySourceFigures(row.id, reading);
    if (run.language === 'fr') verifyFrenchOutput(row.id, reading);
  }
  for (const language of languages) {
    const index = await fs.readFile(join(outputDir, `README.${language}.md`), 'utf8');
    for (const slug of people) if (!index.includes(`individual-${slug}-${language}.md`)) throw new Error(`${language}: index is missing ${slug}.`);
    if (includeTriad && !index.includes(`business-partners-triad.${language}.md`)) throw new Error(`${language}: index is missing triad report.`);
  }
  console.log(`REPORT_SET_VERIFIED ${outputDir}`);
  process.exit(0);
}
const tsxCli = resolve(repo, 'packages/witness-pipeline/node_modules/tsx/dist/cli.mjs');
const runnerArgs = [tsxCli, runnerPath, manifestPath, matrixOut];
if (has('--stub')) runnerArgs.push('--stub');
const only = selectedIds?.join(',');
if (only) runnerArgs.push('--only', only);
// The mode runner resolves mode files from its cwd's `modes/` directory.
if (!has('--finalize-only')) {
  execFileSync(process.execPath, runnerArgs, { cwd: resolve(repo, 'packages/witness-pipeline'), stdio: 'inherit' });
}
const matrix = JSON.parse(await fs.readFile(join(matrixOut, 'matrix-summary.json'), 'utf8')) as Array<Record<string, any>>;
const expectedIds = runs.map((run) => run.id);
const missing = expectedIds.filter((id) => !matrix.some((row) => row.id === id));
if (missing.length) throw new Error(`Matrix did not finish every requested report: ${missing.join(', ')}`);
const generatedIds = new Set(has('--finalize-only') ? expectedIds : (selectedIds ?? expectedIds));
for (const row of matrix.filter((candidate) => generatedIds.has(candidate.id))) {
  if (row.status !== 'ok' || row.passes !== 12 || row.verification !== 'PASS') {
    throw new Error(`Report ${row.id} failed its twelve-pass or final-verification gate; inspect ${join(matrixOut, row.id, 'result.json')}`);
  }
  if (jev === 'shadow' && (row.jev_summary?.judged !== 12 || row.jev_summary?.skipped || row.jev_summary?.errors)) {
    throw new Error(`Report ${row.id} lacks twelve judged Jev receipts; inspect ${join(matrixOut, row.id, 'jev-receipts.json')}`);
  }
  if (row.source_pack?.gate !== 'ready') throw new Error(`Report ${row.id} source-pack gate is ${row.source_pack?.gate ?? 'missing'}`);
  const receiptsPath = join(matrixOut, row.id, 'jev-receipts.json');
  if (jev === 'shadow') {
    const receipts = JSON.parse(await fs.readFile(receiptsPath, 'utf8'));
    const confidentFailures = receipts.filter((r: any) => r.blocked || r.verdicts?.guardrail === 'fail' || r.verdicts?.framing === 'fail' || r.verdicts?.grounding === 'fail');
    if (confidentFailures.length) throw new Error(`Report ${row.id} retains ${confidentFailures.length} Jev guardrail/framing/grounding failure(s); do not mark it final.`);
  }
}

function normalizeChannel(value: string): string {
  return value.replace(/[–—]/g, '-').replace(/\s+/g, '');
}

async function addSourceFigures(id: string, reading: string): Promise<string> {
  const figures: string[] = [];
  const fr = id.endsWith('-fr');
  if (id.startsWith('gary-abitbol-')) {
    const asset = join(packageDir, 'raw/bodygraph.svg');
    if (await fs.access(asset).then(() => true, () => false)) figures.push(fr
      ? `![Schéma du Design Humain issu du thème HumDes sélectionné](../raw/bodygraph.svg)`
      : `![Human Design bodygraph, captured from the selected HumDes chart](../raw/bodygraph.svg)`);
  }
  if (id.startsWith('business-partners-triad-')) {
    const asset = join(packageDir, 'raw/business-penta-context.png');
    if (await fs.access(asset).then(() => true, () => false)) figures.push(fr
      ? `![Image source du Penta commercial fournie pour cette lecture](../raw/business-penta-context.png)`
      : `![Business Penta source image provided for this reading](../raw/business-penta-context.png)`);
  }
  if (!figures.length || reading.includes(figures[0])) return reading;
  return `${reading.trimEnd()}\n\n## ${fr ? 'Images sources des thèmes' : 'Source chart images'}\n\n${figures.join('\n\n')}\n`;
}

async function verifySourceFigures(id: string, reading: string) {
  const checks = id.startsWith('gary-abitbol-')
    ? [['raw/bodygraph.svg', '../raw/bodygraph.svg']]
    : id.startsWith('business-partners-triad-')
      ? [['raw/business-penta-context.png', '../raw/business-penta-context.png']]
      : [];
  for (const [asset, link] of checks) {
    if (await fs.access(join(packageDir, asset)).then(() => true, () => false) && !reading.includes(`](${link})`)) {
      throw new Error(`${id}: final report is missing its captured chart-image link ${link}.`);
    }
  }
}

function datesFromEngineResult(engineId: string, result: any): Set<string> {
  const dates = new Set<string>();
  const add = (value: unknown) => {
    if (typeof value !== 'string') return;
    const match = value.match(/(?:19|20)\d{2}-\d{2}-\d{2}/);
    if (match) dates.add(match[0]);
  };
  if (!result || typeof result !== 'object') return dates;
  if (engineId === 'biorhythm') {
    add(result.target_date); add(result.calculated_for); add(result.calculated_at);
    for (const item of result.forecast ?? []) add(item?.date);
    for (const date of result.critical_days ?? []) add(date);
  } else if (['vedic-clock', 'transits'].includes(engineId)) {
    add(result.calculated_for); add(result.calculated_at); add(result.target_date);
  } else if (engineId === 'vimshottari') {
    return datesFromVimshottari(result);
  }
  return dates;
}

function datesFromVimshottari(result: any): Set<string> {
  const dates = new Set<string>();
  const visit = (value: unknown) => {
    if (typeof value === 'string') {
      const match = value.match(/(?:19|20)\d{2}-\d{2}-\d{2}/);
      if (match) dates.add(match[0]);
      return;
    }
    if (Array.isArray(value)) return value.forEach(visit);
    if (value && typeof value === 'object') Object.values(value as Record<string, unknown>).forEach(visit);
  };
  visit(result?.current_period);
  visit(result?.timeline);
  return dates;
}

async function verifyTemporalMentions(run: (typeof runs)[number], reading: string) {
  const temporalTerms = [
    { engine: 'biorhythm', pattern: /\b(?:bio[- ]?rhythm|biorythme)\b/i },
    { engine: 'transits', pattern: /\b(?:transits?|transits? astrologiques)\b/i },
    { engine: 'vedic-clock', pattern: /\b(?:vedic[- ]clock|dosha|organ|somatic|somatique|vitality|vitalité|body rhythm|rythme corporel)\b/i },
    { engine: 'vimshottari', pattern: /\b(?:vimshottari|mahadasha|antardasha|pratyantardasha)\b/i },
    { engine: 'chart-source', pattern: /\b(?:bodygraph|design_datetime_displayed|capture timestamp|date of birth|birth date|date de naissance|date de design affichée|horodatage de capture)\b/i },
  ];
  const sourceDates = new Map<string, Map<string, Set<string>>>();
  const vimshottariDates = new Map<string, Set<string>>();
  for (const subject of run.subjects) {
    const outputs = JSON.parse(await fs.readFile(join(enginesDir, `${subject.slug}.prod.json`), 'utf8')) as any[];
    const dates = new Map<string, Set<string>>();
    for (const engine of temporalTerms) {
      const record = outputs.find((candidate) => candidate.engine_id === engine.engine);
      dates.set(engine.engine, record ? datesFromEngineResult(engine.engine, record.result) : new Set());
    }
    sourceDates.set(subject.name, dates);
    const vimshottari = outputs.find((candidate) => candidate.engine_id === 'vimshottari');
    vimshottariDates.set(subject.name, datesFromVimshottari(vimshottari?.result));
  }

  const monthNames: Record<string, string> = {
    jan: '01', january: '01', janvier: '01', feb: '02', february: '02', février: '02', fevrier: '02',
    mar: '03', march: '03', mars: '03', apr: '04', april: '04', avril: '04', may: '05', mai: '05',
    jun: '06', june: '06', juin: '06', jul: '07', july: '07', juillet: '07', aug: '08', august: '08', août: '08', aout: '08',
    sep: '09', sept: '09', september: '09', septembre: '09', oct: '10', october: '10', octobre: '10',
    nov: '11', november: '11', novembre: '11', dec: '12', december: '12', décembre: '12', decembre: '12',
  };
  const dateTokens = (line: string) => {
    const tokens: string[] = [...line.matchAll(/\b(?:19|20)\d{2}-\d{2}-\d{2}\b/g)].map((match) => match[0]);
    for (const match of line.matchAll(/\b(\d{1,2})[./](\d{1,2})[./]((?:19|20)\d{2})\b/g)) {
      // Locale-specific numeric order is ambiguous; require an exact source form
      // instead of guessing whether a human-written date is DD/MM or MM/DD.
      tokens.push(`${match[0]} [ambiguous-format]`);
    }
    const monthPattern = Object.keys(monthNames).sort((a, b) => b.length - a.length).join('|');
    const monthFirst = new RegExp(`\\b(${monthPattern})\\.?\\s+(\\d{1,2})(?:st|nd|rd|th)?[,]?\\s+((?:19|20)\\d{2})\\b`, 'gi');
    const dayFirst = new RegExp(`\\b(\\d{1,2})\\s+(${monthPattern})\\.?[,]?\\s+((?:19|20)\\d{2})\\b`, 'gi');
    for (const match of line.matchAll(monthFirst)) tokens.push(`${match[3]}-${monthNames[match[1].toLowerCase()]}-${match[2].padStart(2, '0')}`);
    for (const match of line.matchAll(dayFirst)) tokens.push(`${match[3]}-${monthNames[match[2].toLowerCase()]}-${match[1].padStart(2, '0')}`);
    return tokens;
  };
  const aliases = new Map([
    ['gary-abitbol', /Gary Abitbol/i],
    ['mohan-kumar', /Mohan Kumar/i],
    ['sheshnarayan', /Sheshnarayan Cumbipuram Nateshan/i],
  ]);
  for (const [index, line] of reading.split(/\r?\n/).entries()) {
    const dates = dateTokens(line);
    if (!dates.length) continue;
    for (const date of dates) {
      const literalDate = date.replace(' [ambiguous-format]', '');
      const datePosition = line.indexOf(literalDate);
      const prefix = line.slice(0, Math.max(datePosition, 0));
      const nearestEngine = temporalTerms
        .map((entry) => ({ entry, index: entry.pattern.exec(prefix)?.index ?? -1 }))
        .filter((candidate) => candidate.index >= 0)
        .sort((a, b) => b.index - a.index)[0]?.entry;
      const exactVimshottariDate = !date.endsWith('[ambiguous-format]') &&
        run.subjects.some((subject) => vimshottariDates.get(subject.name)?.has(date));
      const engineForDate = exactVimshottariDate && /\b(?:vimshottari|mahadasha|antardasha|pratyantardasha)\b/i.test(prefix)
        ? temporalTerms.find((entry) => entry.engine === 'vimshottari')
        : nearestEngine;
      if (!engineForDate) continue;
      if (engineForDate.engine === 'chart-source') continue;
      const namedBeforeDate = run.subjects.flatMap((subject) => {
        const alias = aliases.get(subject.slug)!;
        const globalAlias = new RegExp(alias.source, `${alias.flags}g`);
        const match = [...prefix.matchAll(globalAlias)].at(-1);
        return match ? [{ subject, position: match.index ?? -1 }] : [];
      }).sort((a, b) => b.position - a.position);
      let owner = namedBeforeDate[0]?.subject;
      if (run.subjects.length > 1 && /\b(?:vimshottari|mahadasha|antardasha|pratyantardasha)\b/i.test(prefix) && !date.endsWith('[ambiguous-format]')) {
        const dateOwners = run.subjects.filter((subject) => vimshottariDates.get(subject.name)?.has(date));
        if (dateOwners.length === 1) owner = dateOwners[0];
      }
      if (!owner && run.subjects.length === 1) owner = run.subjects[0];
      if (!owner) throw new Error(`${run.id}: dated somatic/temporal-engine statement on line ${index + 1} does not identify its subject for ${date}.`);

      const permitted = sourceDates.get(owner.name)?.get(engineForDate.engine) ?? new Set<string>();
      if (date.endsWith('[ambiguous-format]') || !permitted.has(date)) {
        throw new Error(`${run.id}: ${engineForDate.engine} date ${date} on line ${index + 1} is not present in ${owner.name}'s exact engine date/timestamp fields.`);
      }
    }
  }
}

function verifyFrenchOutput(id: string, reading: string) {
  const englishProse = /\b(?:The engine|The chart|The screenshot|This section|This pass|Within Human Design|One open question|Open questions remaining|The pattern across|The three records|Exact complete channels|Gate-level overlap|The supplied records|The source data|The source record|The engine facts|A shared channel|No further|The following table|The chart does not|Open question:)\b/gi;
  const proseLines = reading.split(/\r?\n/).filter((line) => line.trim() && !/^\s*[#|>*`-]/.test(line));
  const leaks = proseLines.flatMap((line, index) => [...line.matchAll(englishProse)].map((match) => `${index + 1}:${match[0]}`));
  if (leaks.length > 0) throw new Error(`${id}: French report contains English prose fragments: ${leaks.slice(0, 5).join(', ')}.`);
}

async function verifySourceAnchors(run: (typeof runs)[number], reading: string) {
  const allowedChannels = new Set<string>();
  const activeGates = new Map<string, number[]>();
  for (const subject of run.subjects) {
    const outputs = JSON.parse(await fs.readFile(join(enginesDir, `${subject.slug}.prod.json`), 'utf8')) as any[];
    const hd = outputs.find((engine) => engine.engine_id === 'human-design')?.result;
    if (!hd) throw new Error(`${run.id}: ${subject.name} lacks a Human Design engine result.`);
    for (const key of ['hd_type', 'profile', 'authority', 'definition']) {
      const value = String(hd[key] ?? '').trim();
      if (!value || !reading.toLowerCase().includes(value.toLowerCase())) {
        throw new Error(`${run.id}: ${subject.name} report is missing source value ${key}=${value || '(missing)'}.`);
      }
    }
    for (const channel of hd.active_channels ?? []) {
      const normalized = normalizeChannel(String(channel));
      allowedChannels.add(normalized);
      if (!reading.toLowerCase().replace(/[–—]/g, '-').includes(normalized.toLowerCase())) {
        throw new Error(`${run.id}: ${subject.name} report omits engine-confirmed complete channel ${normalized}.`);
      }
    }
    for (const center of hd.defined_centers ?? []) if (!reading.toLowerCase().includes(String(center).toLowerCase())) {
      throw new Error(`${run.id}: ${subject.name} report omits defined center ${center}.`);
    }
    for (const activationSet of ['personality_activations', 'design_activations']) {
      for (const [planet, activation] of Object.entries(hd[activationSet] ?? {}) as Array<[string, { gate?: number; line?: number }]>) {
        if (!Number.isInteger(activation.gate) || !Number.isInteger(activation.line)) continue;
        const gateLine = new RegExp(`\\b${activation.gate}\\s*[.]\\s*${activation.line}\\b`);
        if (!gateLine.test(reading)) throw new Error(`${run.id}: ${subject.name} report omits ${activationSet} ${planet} ${activation.gate}.${activation.line}.`);
      }
    }
    const activationGates = [...Object.values(hd.personality_activations ?? {}), ...Object.values(hd.design_activations ?? {})]
      .map((activation: any) => Number(activation?.gate)).filter((gate: number) => Number.isInteger(gate));
    if (activationGates.length) activeGates.set(subject.slug, [...new Set(activationGates)].sort((a, b) => a - b));
  }
  // Validate asserted source inventories, not all prose mentions: explanatory
  // text may quote a rejected or explicitly hypothetical gate pair.
  const channelInventoryLines = reading.split(/\r?\n/)
    .filter((line) => /channels?|canaux?/i.test(line) && (/^\s*\|/.test(line) || /^\s*(?:complete channels?|canaux? complets?)\b/i.test(line)))
    .map((line) => /^\s*\|/.test(line) ? line.split('|').slice(1, 2).join('|') : line);
  const mentionedChannels = channelInventoryLines.flatMap((rawLine) => {
    const line = rawLine.replace(/\b(?:19|20)\d{2}-\d{2}-\d{2}\b/g, '');
    return [...line.matchAll(/\b(\d{1,2})\s*[-–—]\s*(\d{1,2})\b/g)].map((match) => normalizeChannel(`${match[1]}-${match[2]}`));
  });
  const unsupportedChannels = [...new Set(mentionedChannels.filter((channel) => !allowedChannels.has(channel)))];
  if (unsupportedChannels.length) throw new Error(`${run.id}: report names unsupported complete channel(s): ${unsupportedChannels.join(', ')}.`);
  if (run.relationship_context?.type === 'business-partners') {
    const notePath = join(packageDir, 'raw/business-penta-source.json');
    const penta = JSON.parse(await fs.readFile(notePath, 'utf8'));
    for (const [name, source] of Object.entries(penta.transcribed_legend) as Array<[string, { displayed_age: number; gates: number[] }]>) {
      if (!reading.includes(name) || !reading.includes(String(source.displayed_age))) throw new Error(`${run.id}: missing Penta source anchor for ${name}.`);
      for (const gate of source.gates) if (!new RegExp(`\\b${gate}\\b`).test(reading)) throw new Error(`${run.id}: missing Penta gate ${gate} for ${name}.`);
    }
    if (run.language === 'fr' && !reading.startsWith('# Partenaires commerciaux — synastrie professionnelle')) {
      throw new Error(`${run.id}: French business-partner relationship header is not localized.`);
    }
    const gateEntries = [...activeGates.entries()];
    const slugAliases = new Map([
      ['gary-abitbol', /Gary Abitbol/i],
      ['mohan-kumar', /Mohan Kumar/i],
      ['sheshnarayan', /Sheshnarayan Cumbipuram Nateshan/i],
    ]);
    for (let a = 0; a < gateEntries.length; a++) for (let b = a + 1; b < gateEntries.length; b++) {
      const [slugA, gatesA] = gateEntries[a];
      const [slugB, gatesB] = gateEntries[b];
      if (!gatesA || !gatesB) continue;
      const expected = gatesA.filter((gate) => gatesB.includes(gate));
      const aliasA = slugAliases.get(slugA)!;
      const aliasB = slugAliases.get(slugB)!;
      for (const line of reading.split(/\r?\n/)) {
        if (!(aliasA.test(line) && aliasB.test(line)) || !/[∩×↔&]/.test(line) || !line.includes('|')) continue;
        const cells = line.split('|').map((cell) => cell.trim()).filter(Boolean);
        if (cells.length < 3) continue;
        const overlapCell = cells[1];
        // Other sections may mention the same pair in a table label followed by
        // prose such as "none in the activation columns". Only parse a numeric
        // gate inventory as the pairwise overlap value.
        if (!/^(?:\d{1,2})(?:\s*[,·]\s*\d{1,2})*$/.test(overlapCell)) continue;
        const found = [...overlapCell.matchAll(/\b\d{1,2}\b/g)].map((match) => Number(match[0]));
        if (found.length && (found.length !== expected.length || found.some((gate, index) => gate !== expected[index]))) {
          throw new Error(`${run.id}: a pairwise gate row contradicts the computed ${slugA}/${slugB} intersection (${expected.join(', ')}).`);
        }
      }
    }
    const forbiddenInferences = [
      /frequency (?:two )?people can meet inside/i,
      /each defined on only one side/i,
      /not in the engine lists above/i,
      /not in the engine gate lists/i,
      /Gary Abitbol.{0,100}gate 57/i,
      /Porte 57.{0,100}Gary Abitbol/i,
    ];
    const contradiction = forbiddenInferences.find((pattern) => pattern.test(reading));
    if (contradiction) throw new Error(`${run.id}: report contains a known unsupported or source-contradicting synastry claim (${contradiction}).`);
  }
}

// A selective repair still finalizes the complete report set after its chosen
// rows pass; earlier successful rows may already be validated and persisted.
for (const run of runs) {
  const source = join(matrixOut, run.id, 'reading.md');
  if (!(await fs.access(source).then(() => true, () => false))) continue;
  const reading = await addSourceFigures(run.id, await fs.readFile(source, 'utf8'));
  await fs.writeFile(source, reading);
  await verifySourceAnchors(run, reading);
  await verifyTemporalMentions(run, reading);
  const suffix = run.id.startsWith('business-partners-triad-') ? `business-partners-triad.${run.language}.md` : `individual-${run.id}.md`;
  await fs.copyFile(source, join(outputDir, suffix));
}

const personLabels: Record<string, string> = {
  'gary-abitbol': 'Gary Abitbol',
  'mohan-kumar': 'Mohan Kumar',
  sheshnarayan: 'Sheshnarayan Cumbipuram Nateshan',
};
const individualLinks = (language: string) => people.map((slug) => `- [${personLabels[slug]}](individual-${slug}-${language}.md)`).join('\n');
for (const language of languages) {
  const heading = language === 'fr' ? 'Lectures intégrées de Design Humain' : 'Integrated Human Design reports';
  const soloHeading = language === 'fr' ? 'Lectures individuelles' : 'Individual readings';
  const triadHeading = language === 'fr' ? 'Synastrie des partenaires commerciaux' : 'Business-partner synastry';
  const sourceText = language === 'fr'
    ? 'Chaque lecture individuelle comporte douze passes séquencées avec examen Jev en mode shadow. La synastrie triangulaire comporte elle aussi douze passes. Les résultats et reçus Jev par passe sont dans matrix-runs/. Une vérification Cloudflare en lecture seule a confirmé le déploiement du Worker passerelle Selemene, mais cette génération a utilisé les fichiers locaux validés des moteurs, sans appel à la passerelle. Le Worker de mémoire de motifs Selemene est absent du compte vérifié; aucune récupération ni persistance Selemene via Vectorize/R2/D1 n’a donc été utilisée. Aucun accès Cortex n’est raccordé à cette route et aucune requête n’a été effectuée. Aucun écouteur d’événements en direct n’est relié à cette commande. Les dossiers de sources restent locaux.'
    : 'Each individual reading uses twelve sequenced passes with Jev shadow review. The three-partner synastry also uses twelve passes. Per-pass results and Jev receipts are in matrix-runs/. A read-only Cloudflare check confirmed the Selemene gateway Worker is deployed, but this generation used the validated local engine files and made no gateway call. The Selemene pattern-memory Worker is absent from the verified account, so no Selemene Vectorize/R2/D1 retrieval or persistence was used. Cortex is not connected to this report route and was not queried. No live event listener is wired to this command. Source packs remain local.';
  const content = `# ${heading}\n\n${sourceText}\n\n## ${soloHeading}\n${individualLinks(language)}\n\n## ${triadHeading}\n- [${triadHeading}](business-partners-triad.${language}.md)\n`;
  await fs.writeFile(join(outputDir, `README.${language}.md`), content);
}
console.log(`REPORT_SET_READY ${outputDir}`);
