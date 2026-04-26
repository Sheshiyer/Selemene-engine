#!/usr/bin/env npx ts-node

/**
 * Scoring Script
 *
 * Applies rubrics to test outputs and generates scores.
 * Can be run in manual mode (human scorer) or automated mode (LLM scorer).
 *
 * Usage:
 *   npx ts-node scripts/score.ts --run=01-simple-endpoint --mode=auto
 *   npx ts-node scripts/score.ts --suite=suite-1234567890 --mode=manual
 */

import Anthropic from '@anthropic-ai/sdk';
import * as fs from 'fs';
import * as path from 'path';
import * as readline from 'readline';
import { TestRun, Score, TestConfig } from './types';

const ROOT = path.resolve(__dirname, '..');

function loadRubrics(): Record<string, string> {
  const rubricsDir = path.join(ROOT, 'rubrics');
  const files = fs.readdirSync(rubricsDir);

  const rubrics: Record<string, string> = {};
  for (const file of files) {
    if (file.endsWith('.md')) {
      const name = file.replace('.md', '').replace(/-/g, '_');
      rubrics[name] = fs.readFileSync(path.join(rubricsDir, file), 'utf-8');
    }
  }
  return rubrics;
}

function loadTestCase(caseId: string): string {
  const casePath = path.join(ROOT, 'cases', `${caseId}.md`);
  return fs.readFileSync(casePath, 'utf-8');
}

function loadTestRun(caseId: string, group: string, runNumber: number): TestRun | null {
  const runPath = path.join(ROOT, 'outputs', caseId, group, `run-${runNumber}.json`);
  if (!fs.existsSync(runPath)) {
    return null;
  }
  return JSON.parse(fs.readFileSync(runPath, 'utf-8'));
}

async function scoreWithLLM(
  testRun: TestRun,
  testCase: string,
  rubrics: Record<string, string>
): Promise<Score[]> {
  const client = new Anthropic();

  const systemPrompt = `You are an expert code reviewer scoring AI-generated code against specific rubrics.

Your task is to evaluate the generated code output and assign scores from 0-10 for each dimension.

Be objective and consistent. Apply deductions as specified in the rubrics.

Return your evaluation as JSON in this exact format:
{
  "scores": [
    {
      "dimension": "pattern_adherence",
      "score": <0-10>,
      "notes": "<brief explanation>",
      "deductions": [{"reason": "<reason>", "amount": <number>}]
    },
    ...for each dimension...
  ]
}`;

  const userPrompt = `# Test Case
${testCase}

# Generated Output to Score
${testRun.output}

# Scoring Rubrics

## Pattern Adherence
${rubrics.pattern_adherence}

## Completeness
${rubrics.completeness}

## Correctness
${rubrics.correctness}

## Security
${rubrics.security}

## Consistency
${rubrics.consistency}

Please score the generated output against each rubric dimension. Return JSON only.`;

  const response = await client.messages.create({
    model: 'claude-sonnet-4-20250514',
    max_tokens: 2000,
    temperature: 0,
    system: systemPrompt,
    messages: [{ role: 'user', content: userPrompt }]
  });

  const text = response.content
    .filter((block) => block.type === 'text')
    .map((block) => (block as { type: 'text'; text: string }).text)
    .join('');

  // Extract JSON from response
  const jsonMatch = text.match(/\{[\s\S]*\}/);
  if (!jsonMatch) {
    throw new Error('Could not parse scoring response');
  }

  const parsed = JSON.parse(jsonMatch[0]);

  return parsed.scores.map(
    (s: { dimension: string; score: number; notes: string; deductions: Array<{ reason: string; amount: number }> }) => ({
      run_id: testRun.id,
      dimension: s.dimension,
      score: s.score,
      notes: s.notes,
      deductions: s.deductions || []
    })
  );
}

async function scoreManually(
  testRun: TestRun,
  testCase: string,
  rubrics: Record<string, string>
): Promise<Score[]> {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });

  const question = (prompt: string): Promise<string> =>
    new Promise((resolve) => rl.question(prompt, resolve));

  console.log('\n' + '='.repeat(60));
  console.log('Manual Scoring');
  console.log('='.repeat(60));
  console.log(`\nTest Run: ${testRun.id}`);
  console.log(`\nGenerated Output:\n${testRun.output.slice(0, 2000)}...`);

  const scores: Score[] = [];
  const dimensions = ['pattern_adherence', 'completeness', 'correctness', 'security', 'consistency'];

  for (const dimension of dimensions) {
    console.log(`\n--- ${dimension.replace(/_/g, ' ').toUpperCase()} ---`);
    console.log('Rubric summary available in rubrics/ folder');

    const scoreStr = await question(`Score (0-10): `);
    const score = parseFloat(scoreStr);

    const notes = await question('Notes: ');

    scores.push({
      run_id: testRun.id,
      dimension,
      score: isNaN(score) ? 0 : score,
      notes,
      deductions: []
    });
  }

  rl.close();
  return scores;
}

async function scoreRun(
  caseId: string,
  group: 'control' | 'treatment',
  runNumber: number,
  mode: 'auto' | 'manual'
): Promise<Score[]> {
  const testRun = loadTestRun(caseId, group, runNumber);
  if (!testRun) {
    throw new Error(`Test run not found: ${caseId}/${group}/run-${runNumber}`);
  }

  const testCase = loadTestCase(caseId);
  const rubrics = loadRubrics();

  const scores = mode === 'auto'
    ? await scoreWithLLM(testRun, testCase, rubrics)
    : await scoreManually(testRun, testCase, rubrics);

  // Save scores
  const scoresDir = path.join(ROOT, 'results', 'scores', caseId, group);
  fs.mkdirSync(scoresDir, { recursive: true });
  fs.writeFileSync(
    path.join(scoresDir, `run-${runNumber}-scores.json`),
    JSON.stringify(scores, null, 2)
  );

  console.log(`Scores saved for ${caseId}/${group}/run-${runNumber}`);
  return scores;
}

// CLI Entry Point
const args = process.argv.slice(2);
const caseArg = args.find((a) => a.startsWith('--case='));
const groupArg = args.find((a) => a.startsWith('--group='));
const runArg = args.find((a) => a.startsWith('--run='));
const modeArg = args.find((a) => a.startsWith('--mode='));

if (!caseArg || !groupArg) {
  console.error('Usage: npx ts-node score.ts --case=<id> --group=<control|treatment> [--run=<n>] [--mode=auto|manual]');
  process.exit(1);
}

const caseId = caseArg.split('=')[1];
const group = groupArg.split('=')[1] as 'control' | 'treatment';
const runNumber = runArg ? parseInt(runArg.split('=')[1]) : 1;
const mode = (modeArg?.split('=')[1] ?? 'auto') as 'auto' | 'manual';

scoreRun(caseId, group, runNumber, mode).catch(console.error);

export { scoreRun, loadRubrics, scoreWithLLM };
