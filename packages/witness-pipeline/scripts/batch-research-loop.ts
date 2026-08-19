#!/usr/bin/env node
// Auto-Research Loop: Batch fetch engines + run L0 + aggregate lessons
// Usage: pnpm tsx scripts/batch-research-loop.ts <batch-number> [--skip-fetch] [--skip-run]
// Reads humdes-batch-config.json from /tmp

import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';
import { execSync } from 'node:child_process';

interface Person {
  name: string; date: string; time: string; timezone: string;
  latitude: number; longitude: number; hd_type: string; profile: string; authority: string;
}

const BATCH_INPUTS_DIR = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/witness-agents/.batch-inputs';
const SOLOS_DIR = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/Solos';
const RESEARCH_DIR = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/auto-research';
const CONFIG_PATH = '/tmp/humdes-batch-config.json';

function slugify(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

async function fetchEngines(person: Person): Promise<any[]> {
  const slug = slugify(person.name);
  const batchFile = join(BATCH_INPUTS_DIR, `${slug}.json`);
  
  // Check if already fetched
  try {
    const existing = JSON.parse(readFileSync(batchFile, 'utf8'));
    if (existing.engines && existing.engines.length >= 14) {
      console.log(`  [${slug}] Already has ${existing.engines.length} engines, skipping fetch`);
      return existing.engines;
    }
  } catch {}

  // Fetch via the Selemene API
  const fetcherScript = `
import { fetchAllEngines, loadSelemeneKey, SELEMENE_BASE_URL } from './scripts/integratedreading/selemene/fetcher.js';
import { writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const birth = ${JSON.stringify({ date: person.date, time: person.time, timezone: person.timezone, latitude: person.latitude, longitude: person.longitude, name: person.name })};

(async () => {
  const key = await loadSelemeneKey();
  if (!key) { process.stdout.write('ERROR:NO_KEY'); process.exit(1); }
  const results = await fetchAllEngines(birth, { api_key: key, base_url: SELEMENE_BASE_URL, timeout_ms: 60000 });
  mkdirSync('.batch-inputs', { recursive: true });
  writeFileSync('.batch-inputs/${slug}.json', JSON.stringify({ birth_data: birth, engines: results }, null, 2));
  process.stdout.write('OK:' + results.length);
})().catch(e => { process.stdout.write('ERROR:' + e.message); process.exit(1); });
`;

  const tmpScript = `/tmp/fetch-${slug}.ts`;
  writeFileSync(tmpScript, fetcherScript.trim());
  
  try {
    const output = execSync(`npx tsx ${tmpScript}`, {
      cwd: '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/witness-agents',
      timeout: 120000,
      encoding: 'utf8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    if (output.startsWith('OK:')) {
      const count = parseInt(output.split(':')[1]);
      console.log(`  [${slug}] Fetched ${count} engines`);
      const data = JSON.parse(readFileSync(batchFile, 'utf8'));
      return data.engines;
    }
    throw new Error(output);
  } catch (e: any) {
    console.error(`  [${slug}] Fetch error: ${e.stderr || e.message}`);
    return [];
  }
}

function setupSoloDir(person: Person): string {
  const slug = slugify(person.name);
  const soloDir = join(SOLOS_DIR, slug);
  const newL0Dir = join(soloDir, 'new-l0-flow');
  
  mkdirSync(newL0Dir, { recursive: true });
  
  const request = {
    report_level: 'L0',
    report_mode: 'integrated-kundali-l0',
    subjects: [{
      role: 'primary',
      name: person.name,
      birth_date: person.date,
      birth_time: person.time,
      birth_time_confidence: 'exact' as const,
      birth_location_query: person.timezone,
      normalized_location: {
        display_name: person.timezone,
        latitude: person.latitude,
        longitude: person.longitude,
        timezone: person.timezone,
        provider: 'manual' as const,
        confidence: 'manual' as const,
      },
    }],
    output: { format: 'source-pack', include_rubric: true, include_pattern_extraction: true },
  };
  
  writeFileSync(join(newL0Dir, 'request.json'), JSON.stringify(request, null, 2));
  
  // Copy engines from batch-input
  const batchFile = join(BATCH_INPUTS_DIR, `${slug}.json`);
  const engines = JSON.parse(readFileSync(batchFile, 'utf8')).engines.filter((e: any) => !e._error);
  writeFileSync(join(newL0Dir, 'engines.json'), JSON.stringify(engines, null, 2));
  
  return soloDir;
}

function runL0(soloDir: string): { summary: any; lessons: string } {
  const runnerWorkdir = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine-worktrees/plan-l0-kundali/packages/witness-pipeline';
  
  const output = execSync(`pnpm tsx scripts/solo-l0-runner.ts "${soloDir}"`, {
    cwd: runnerWorkdir,
    timeout: 300000,
    encoding: 'utf8',
    stdio: ['pipe', 'pipe', 'pipe'],
  });

  // Parse summary from output
  const summaryMatch = output.match(/=== SUMMARY ===\s*(\{[\s\S]*?\n\})/);
  const summary = summaryMatch ? JSON.parse(summaryMatch[1]) : null;
  
  const lessonsPath = join(soloDir, 'lessons.md');
  const lessons = readFileSync(lessonsPath, 'utf8');
  
  return { summary, lessons };
}

function extractRubricRow(lessons: string): any[] {
  // Parse the markdown table
  const tableMatch = lessons.match(/\| Pass \| Word Fit \|.*?\n\|------\|.*?\n([\s\S]*?)\n\n/);
  if (!tableMatch) return [];
  const rows: any[] = [];
  for (const line of tableMatch[1].trim().split('\n')) {
    const cols = line.split('|').map(c => c.trim()).filter(Boolean);
    if (cols.length >= 7) {
      rows.push({
        pass: cols[0],
        word_fit: cols[1],
        facts: cols[2],
        layers: cols[3],
        guard: cols[4],
        placeholder: cols[5],
        fidelity: cols[6],
      });
    }
  }
  return rows;
}

async function main() {
  const batchNum = parseInt(process.argv[2] || '1');
  const skipFetch = process.argv.includes('--skip-fetch');
  const skipRun = process.argv.includes('--skip-run');
  
  const config = JSON.parse(readFileSync(CONFIG_PATH, 'utf8'));
  const batch: Person[] = config.batches[batchNum - 1];
  
  if (!batch) {
    console.error(`Batch ${batchNum} not found. Batches: 1-${config.batches.length}`);
    process.exit(1);
  }

  mkdirSync(RESEARCH_DIR, { recursive: true });
  mkdirSync(join(RESEARCH_DIR, `batch-${batchNum}`), { recursive: true });
  
  console.log(`\n=== Auto-Research Batch ${batchNum} ===`);
  console.log(`${batch.length} people\n`);
  
  const results: any[] = [];
  
  for (let i = 0; i < batch.length; i++) {
    const person = batch[i];
    const slug = slugify(person.name);
    console.log(`[${i + 1}/${batch.length}] ${person.name} (${person.date})`);
    
    // Step 1: Fetch engines
    if (!skipFetch) {
      await fetchEngines(person);
    }
    
    // Step 2: Setup solo
    const soloDir = setupSoloDir(person);
    
    // Step 3: Run L0
    if (!skipRun) {
      try {
        const { summary, lessons } = runL0(soloDir);
        const rubricRows = extractRubricRow(lessons);
        
        results.push({
          name: person.name,
          slug,
          date: person.date,
          time: person.time,
          hd_type: person.hd_type,
          profile: person.profile,
          authority: person.authority,
          summary,
          rubric_row_count: rubricRows.length,
          fidelity_pass_count: rubricRows.filter(r => r.fidelity === 'pass').length,
          word_fit_pass_count: rubricRows.filter(r => r.word_fit === 'pass').length,
          placeholder_pass_count: rubricRows.filter(r => r.placeholder === 'pass').length,
        });
        
        console.log(`  -> verification=${summary?.verification}, fidelity=${rubricRows.filter(r => r.fidelity === 'pass').length}/${rubricRows.length}, word_fit=${rubricRows.filter(r => r.word_fit === 'pass').length}/${rubricRows.length}`);
      } catch (e: any) {
        console.error(`  -> RUN ERROR: ${e.stderr || e.message}`);
        results.push({
          name: person.name, slug, date: person.date,
          error: e.stderr || e.message,
        });
      }
    }
  }
  
  // Aggregate results
  const batchDir = join(RESEARCH_DIR, `batch-${batchNum}`);
  writeFileSync(join(batchDir, 'results.json'), JSON.stringify(results, null, 2));
  
  // Generate batch lessons summary
  const passing = results.filter(r => !r.error && r.summary?.verification === 'PASS');
  const failing = results.filter(r => !r.error && r.summary?.verification !== 'PASS');
  const errored = results.filter(r => r.error);
  
  const fidelityScores = results.filter(r => r.fidelity_pass_count !== undefined);
  const avgFidelity = fidelityScores.length > 0 
    ? fidelityScores.reduce((s, r) => s + r.fidelity_pass_count, 0) / (fidelityScores.length * 12) 
    : 0;
  
  const lessonsSummary = [
    `# Auto-Research Batch ${batchNum} — Lessons`,
    '',
    `Generated: ${new Date().toISOString()}`,
    `People: ${batch.length} | Passed: ${passing.length} | Failed: ${failing.length} | Errors: ${errored.length}`,
    `Average fidelity pass rate: ${(avgFidelity * 100).toFixed(1)}%`,
    '',
    '## Per-Person Results',
    '| Name | Date | HD Type | Profile | Auth | Verification | Fidelity | Word Fit |',
    '|------|------|---------|---------|------|-------------|----------|----------|',
  ];
  
  for (const r of results) {
    if (r.error) {
      lessonsSummary.push(`| ${r.name} | ${r.date} | - | - | - | ERROR | - | - |`);
    } else {
      lessonsSummary.push(`| ${r.name} | ${r.date} | ${r.hd_type} | ${r.profile} | ${r.authority} | ${r.summary?.verification} | ${r.fidelity_pass_count}/12 | ${r.word_fit_pass_count}/12 |`);
    }
  }
  
  lessonsSummary.push('');
  lessonsSummary.push('## Cross-Person Patterns');
  lessonsSummary.push(`- Fidelity pass rate: ${(avgFidelity * 100).toFixed(1)}% (target: 100%)`);
  lessonsSummary.push(`- Word fit pass rate: ${fidelityScores.length > 0 ? (fidelityScores.reduce((s,r) => s + r.word_fit_pass_count, 0) / (fidelityScores.length * 12) * 100).toFixed(1) : 'N/A'}% (target: 100%)`);
  lessonsSummary.push(`- Errors: ${errored.map(r => r.name + ': ' + r.error).join('; ') || 'none'}`);
  
  lessonsSummary.push('');
  lessonsSummary.push('## Optimization Candidates');
  if (avgFidelity < 1.0) lessonsSummary.push('- Fidelity gate: check if all engine facts are included in stub LLM output');
  const avgWordFit = fidelityScores.length > 0 ? fidelityScores.reduce((s,r) => s + r.word_fit_pass_count, 0) / fidelityScores.length : 0;
  if (avgWordFit < 12) lessonsSummary.push('- Word count: stub LLM produces fixed-length output; recalibrate pass target words or LLM parameters');
  lessonsSummary.push('- Replace stub LLM with production model for real narrative quality baseline');
  lessonsSummary.push('- Cross-reference fidelity scores with HD type/authority to identify per-type gaps');
  
  writeFileSync(join(batchDir, 'lessons.md'), lessonsSummary.join('\n') + '\n');
  
  console.log(`\n=== BATCH ${batchNum} COMPLETE ===`);
  console.log(`Passed: ${passing.length}, Failed: ${failing.length}, Errors: ${errored.length}`);
  console.log(`Avg fidelity: ${(avgFidelity * 100).toFixed(1)}%`);
  console.log(`Results saved to ${batchDir}/`);
}

main().catch(e => { console.error(e); process.exit(1); });