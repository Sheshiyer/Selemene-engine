#!/usr/bin/env npx ts-node

/**
 * Statistical Analysis Script
 *
 * Analyzes scored results and generates statistical report.
 *
 * Usage:
 *   npx ts-node scripts/analyze.ts --output=report.md
 */

import * as fs from 'fs';
import * as path from 'path';
import { TestConfig, Score, AnalysisResult, SuiteReport } from './types';

const ROOT = path.resolve(__dirname, '..');

function loadConfig(): TestConfig {
  return JSON.parse(fs.readFileSync(path.join(ROOT, 'config.json'), 'utf-8'));
}

function loadAllScores(): Map<string, Score[]> {
  const scoresDir = path.join(ROOT, 'results', 'scores');
  const scores = new Map<string, Score[]>();

  if (!fs.existsSync(scoresDir)) {
    return scores;
  }

  const cases = fs.readdirSync(scoresDir);
  for (const caseId of cases) {
    const casePath = path.join(scoresDir, caseId);
    if (!fs.statSync(casePath).isDirectory()) continue;

    const groups = fs.readdirSync(casePath);
    for (const group of groups) {
      const groupPath = path.join(casePath, group);
      if (!fs.statSync(groupPath).isDirectory()) continue;

      const files = fs.readdirSync(groupPath).filter((f) => f.endsWith('-scores.json'));
      for (const file of files) {
        const scoreData: Score[] = JSON.parse(fs.readFileSync(path.join(groupPath, file), 'utf-8'));
        const key = `${caseId}/${group}`;
        const existing = scores.get(key) || [];
        scores.set(key, [...existing, ...scoreData]);
      }
    }
  }

  return scores;
}

// Statistical functions
function mean(values: number[]): number {
  if (values.length === 0) return 0;
  return values.reduce((a, b) => a + b, 0) / values.length;
}

function standardDeviation(values: number[]): number {
  if (values.length < 2) return 0;
  const m = mean(values);
  const squareDiffs = values.map((v) => Math.pow(v - m, 2));
  return Math.sqrt(mean(squareDiffs));
}

function pooledStandardDeviation(group1: number[], group2: number[]): number {
  const n1 = group1.length;
  const n2 = group2.length;
  if (n1 < 2 || n2 < 2) return 1;

  const var1 = Math.pow(standardDeviation(group1), 2);
  const var2 = Math.pow(standardDeviation(group2), 2);

  return Math.sqrt(((n1 - 1) * var1 + (n2 - 1) * var2) / (n1 + n2 - 2));
}

function cohensD(group1: number[], group2: number[]): number {
  const pooledSD = pooledStandardDeviation(group1, group2);
  if (pooledSD === 0) return 0;
  return (mean(group2) - mean(group1)) / pooledSD;
}

// Mann-Whitney U test (simplified)
function mannWhitneyU(group1: number[], group2: number[]): { u: number; p: number } {
  const n1 = group1.length;
  const n2 = group2.length;

  // Combine and rank
  const combined = [
    ...group1.map((v) => ({ value: v, group: 1 })),
    ...group2.map((v) => ({ value: v, group: 2 }))
  ].sort((a, b) => a.value - b.value);

  // Assign ranks
  let rank = 1;
  const ranks: number[] = [];
  for (let i = 0; i < combined.length; i++) {
    ranks.push(rank);
    rank++;
  }

  // Sum ranks for each group
  let r1 = 0;
  let idx = 0;
  for (const item of combined) {
    if (item.group === 1) {
      r1 += ranks[idx];
    }
    idx++;
  }

  // Calculate U
  const u1 = r1 - (n1 * (n1 + 1)) / 2;
  const u2 = n1 * n2 - u1;
  const u = Math.min(u1, u2);

  // Approximate p-value using normal approximation
  const mu = (n1 * n2) / 2;
  const sigma = Math.sqrt((n1 * n2 * (n1 + n2 + 1)) / 12);
  const z = (u - mu) / sigma;

  // Two-tailed p-value (simplified)
  const p = 2 * (1 - normalCDF(Math.abs(z)));

  return { u, p };
}

function normalCDF(x: number): number {
  const a1 = 0.254829592;
  const a2 = -0.284496736;
  const a3 = 1.421413741;
  const a4 = -1.453152027;
  const a5 = 1.061405429;
  const p = 0.3275911;

  const sign = x < 0 ? -1 : 1;
  x = Math.abs(x) / Math.sqrt(2);

  const t = 1.0 / (1.0 + p * x);
  const y = 1.0 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * Math.exp(-x * x);

  return 0.5 * (1.0 + sign * y);
}

function interpretEffectSize(d: number, config: TestConfig): 'negligible' | 'small' | 'medium' | 'large' {
  const absD = Math.abs(d);
  const thresholds = config.analysis.effect_size_thresholds;

  if (absD >= thresholds.large) return 'large';
  if (absD >= thresholds.medium) return 'medium';
  if (absD >= thresholds.small) return 'small';
  return 'negligible';
}

function analyzeCase(
  caseId: string,
  controlScores: Score[],
  treatmentScores: Score[],
  config: TestConfig
): AnalysisResult {
  // Calculate total scores per run
  const controlTotals = aggregateRunScores(controlScores);
  const treatmentTotals = aggregateRunScores(treatmentScores);

  const controlMean = mean(controlTotals);
  const treatmentMean = mean(treatmentTotals);
  const difference = treatmentMean - controlMean;

  const { p } = mannWhitneyU(controlTotals, treatmentTotals);
  const d = cohensD(controlTotals, treatmentTotals);

  return {
    case_id: caseId,
    control_mean: controlMean,
    treatment_mean: treatmentMean,
    difference,
    p_value: p,
    effect_size: d,
    significant: p < config.analysis.significance_level,
    effect_interpretation: interpretEffectSize(d, config)
  };
}

function aggregateRunScores(scores: Score[]): number[] {
  // Group scores by run_id and sum them
  const byRun = new Map<string, number>();

  for (const score of scores) {
    const current = byRun.get(score.run_id) || 0;
    byRun.set(score.run_id, current + score.score);
  }

  return Array.from(byRun.values());
}

function generateReport(config: TestConfig): SuiteReport {
  const allScores = loadAllScores();

  // Get unique case IDs
  const caseIds = new Set<string>();
  for (const key of allScores.keys()) {
    const [caseId] = key.split('/');
    caseIds.add(caseId);
  }

  const analysis: AnalysisResult[] = [];

  for (const caseId of caseIds) {
    const controlScores = allScores.get(`${caseId}/control`) || [];
    const treatmentScores = allScores.get(`${caseId}/treatment`) || [];

    if (controlScores.length > 0 && treatmentScores.length > 0) {
      analysis.push(analyzeCase(caseId, controlScores, treatmentScores, config));
    }
  }

  // Calculate summary
  const allControlMeans = analysis.map((a) => a.control_mean);
  const allTreatmentMeans = analysis.map((a) => a.treatment_mean);

  const report: SuiteReport = {
    timestamp: new Date().toISOString(),
    config,
    results: [],
    analysis,
    summary: {
      total_tests: analysis.length,
      significant_improvements: analysis.filter((a) => a.significant && a.difference > 0).length,
      control_average: mean(allControlMeans),
      treatment_average: mean(allTreatmentMeans),
      overall_effect_size: mean(analysis.map((a) => a.effect_size))
    }
  };

  return report;
}

function formatMarkdownReport(report: SuiteReport): string {
  let md = `# .context vs CLAUDE.md Comparison Report

Generated: ${report.timestamp}

## Executive Summary

| Metric | Value |
|--------|-------|
| Total Test Cases | ${report.summary.total_tests} |
| Significant Improvements | ${report.summary.significant_improvements} |
| Control Average | ${report.summary.control_average.toFixed(2)} |
| Treatment Average | ${report.summary.treatment_average.toFixed(2)} |
| Overall Effect Size | ${report.summary.overall_effect_size.toFixed(3)} |

## Interpretation

`;

  if (report.summary.treatment_average > report.summary.control_average) {
    const improvement = (
      ((report.summary.treatment_average - report.summary.control_average) /
        report.summary.control_average) *
      100
    ).toFixed(1);
    md += `The structured .context methodology showed a **${improvement}% improvement** over single CLAUDE.md approach.\n\n`;
  } else {
    md += `The single CLAUDE.md approach performed similarly or better than structured .context.\n\n`;
  }

  md += `## Detailed Results

| Test Case | Control | Treatment | Difference | p-value | Effect Size | Significant |
|-----------|---------|-----------|------------|---------|-------------|-------------|
`;

  for (const result of report.analysis) {
    md += `| ${result.case_id} | ${result.control_mean.toFixed(2)} | ${result.treatment_mean.toFixed(2)} | ${result.difference > 0 ? '+' : ''}${result.difference.toFixed(2)} | ${result.p_value.toFixed(4)} | ${result.effect_size.toFixed(3)} (${result.effect_interpretation}) | ${result.significant ? 'Yes' : 'No'} |\n`;
  }

  md += `
## Methodology

- **Model**: ${report.config.model}
- **Temperature**: ${report.config.temperature}
- **Runs per test**: ${report.config.runs_per_test}
- **Significance level**: ${report.config.analysis.significance_level}

## Statistical Notes

- Effect size measured using Cohen's d
- P-values calculated using Mann-Whitney U test
- Significance threshold: p < ${report.config.analysis.significance_level}
`;

  return md;
}

// CLI Entry Point
const args = process.argv.slice(2);
const outputArg = args.find((a) => a.startsWith('--output='));

const config = loadConfig();
const report = generateReport(config);

// Save JSON report
const reportDir = path.join(ROOT, 'results');
fs.mkdirSync(reportDir, { recursive: true });
fs.writeFileSync(path.join(reportDir, 'analysis.json'), JSON.stringify(report, null, 2));

// Save Markdown report
const markdown = formatMarkdownReport(report);
const outputPath = outputArg ? outputArg.split('=')[1] : path.join(reportDir, 'report.md');
fs.writeFileSync(outputPath, markdown);

console.log(`\nAnalysis complete!`);
console.log(`JSON report: ${path.join(reportDir, 'analysis.json')}`);
console.log(`Markdown report: ${outputPath}`);
console.log(`\n${markdown}`);

export { generateReport, analyzeCase, mean, standardDeviation, cohensD, mannWhitneyU };
