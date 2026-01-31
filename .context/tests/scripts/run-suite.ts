#!/usr/bin/env npx ts-node

/**
 * Full Test Suite Runner
 *
 * Executes all test cases for both control and treatment groups,
 * with multiple runs per test for statistical significance.
 *
 * Usage:
 *   npx ts-node scripts/run-suite.ts
 *   npx ts-node scripts/run-suite.ts --cases=01,02,03
 *   npx ts-node scripts/run-suite.ts --runs=5
 */

import * as fs from 'fs';
import * as path from 'path';
import { runTest, loadConfig } from './run-test';

const ROOT = path.resolve(__dirname, '..');

async function getTestCases(filter?: string[]): Promise<string[]> {
  const casesDir = path.join(ROOT, 'cases');
  const files = fs.readdirSync(casesDir);

  let cases = files
    .filter((f) => f.endsWith('.md') && f !== 'TEMPLATE.md')
    .map((f) => f.replace('.md', ''));

  if (filter && filter.length > 0) {
    cases = cases.filter((c) => filter.some((f) => c.includes(f)));
  }

  return cases.sort();
}

interface SuiteOptions {
  cases?: string[];
  runsOverride?: number;
  groups?: Array<'control' | 'treatment'>;
  parallel?: boolean;
}

async function runSuite(options: SuiteOptions = {}): Promise<void> {
  const config = await loadConfig();
  const cases = await getTestCases(options.cases);
  const runs = options.runsOverride ?? config.runs_per_test;
  const groups = options.groups ?? (['control', 'treatment'] as const);

  console.log('='.repeat(60));
  console.log('Test Suite Configuration');
  console.log('='.repeat(60));
  console.log(`Model: ${config.model}`);
  console.log(`Temperature: ${config.temperature}`);
  console.log(`Runs per test: ${runs}`);
  console.log(`Test cases: ${cases.length}`);
  console.log(`Groups: ${groups.join(', ')}`);
  console.log(`Total API calls: ${cases.length * runs * groups.length}`);
  console.log('='.repeat(60));

  const suiteId = `suite-${Date.now()}`;
  const resultsDir = path.join(ROOT, 'results', suiteId);
  fs.mkdirSync(resultsDir, { recursive: true });

  // Save suite metadata
  fs.writeFileSync(
    path.join(resultsDir, 'metadata.json'),
    JSON.stringify(
      {
        id: suiteId,
        timestamp: new Date().toISOString(),
        config,
        cases,
        runs,
        groups
      },
      null,
      2
    )
  );

  const results: Array<{
    case_id: string;
    group: string;
    run: number;
    success: boolean;
    error?: string;
  }> = [];

  for (const caseId of cases) {
    console.log(`\nRunning test case: ${caseId}`);

    for (const group of groups) {
      console.log(`  Group: ${group}`);

      for (let run = 1; run <= runs; run++) {
        try {
          await runTest(caseId, group, run);
          results.push({ case_id: caseId, group, run, success: true });

          // Rate limiting - wait between API calls
          await new Promise((resolve) => setTimeout(resolve, 1000));
        } catch (error) {
          const errorMessage = error instanceof Error ? error.message : String(error);
          console.error(`    Run ${run} failed: ${errorMessage}`);
          results.push({
            case_id: caseId,
            group,
            run,
            success: false,
            error: errorMessage
          });
        }
      }
    }
  }

  // Save results summary
  fs.writeFileSync(path.join(resultsDir, 'results.json'), JSON.stringify(results, null, 2));

  console.log('\n' + '='.repeat(60));
  console.log('Suite Complete');
  console.log('='.repeat(60));
  console.log(`Results saved to: ${resultsDir}`);
  console.log(`Successful runs: ${results.filter((r) => r.success).length}/${results.length}`);
}

// CLI Entry Point
const args = process.argv.slice(2);
const casesArg = args.find((a) => a.startsWith('--cases='));
const runsArg = args.find((a) => a.startsWith('--runs='));

const options: SuiteOptions = {};

if (casesArg) {
  options.cases = casesArg.split('=')[1].split(',');
}

if (runsArg) {
  options.runsOverride = parseInt(runsArg.split('=')[1]);
}

runSuite(options).catch(console.error);

export { runSuite, getTestCases };
