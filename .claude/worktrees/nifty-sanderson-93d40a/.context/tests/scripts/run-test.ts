#!/usr/bin/env npx ts-node

/**
 * Single Test Runner
 *
 * Executes a single test case against either control or treatment group.
 *
 * Usage:
 *   npx ts-node scripts/run-test.ts --case=01-simple-endpoint --group=control --run=1
 */

import Anthropic from '@anthropic-ai/sdk';
import * as fs from 'fs';
import * as path from 'path';
import { TestConfig, TestRun } from './types';

const ROOT = path.resolve(__dirname, '..');

async function loadConfig(): Promise<TestConfig> {
  const configPath = path.join(ROOT, 'config.json');
  return JSON.parse(fs.readFileSync(configPath, 'utf-8'));
}

async function loadContext(group: 'control' | 'treatment', config: TestConfig): Promise<string> {
  const groupConfig = config.groups[group];
  const contextPath = path.join(ROOT, groupConfig.context_path);

  if (group === 'control') {
    // Single file
    return fs.readFileSync(contextPath, 'utf-8');
  } else {
    // Structured .context/ folder - concatenate relevant files
    const files = [
      'substrate.md',
      'architecture/overview.md',
      'architecture/patterns.md',
      'auth/overview.md',
      'auth/security.md',
      'api/endpoints.md',
      'api/responses.md',
      'database/models.md',
      'anti-patterns.md',
      'validation.md',
      'testing.md'
    ];

    let content = '';
    for (const file of files) {
      const filePath = path.join(contextPath, file);
      if (fs.existsSync(filePath)) {
        content += `\n\n---\n# File: ${file}\n\n`;
        content += fs.readFileSync(filePath, 'utf-8');
      }
    }
    return content;
  }
}

function loadProjectFiles(): string {
  const projectPath = path.join(ROOT, 'fixtures/project/src');
  const files = [
    'index.ts',
    'routes/auth.ts',
    'routes/tasks.ts',
    'services/auth-service.ts',
    'services/task-service.ts',
    'middleware/authenticate.ts',
    'middleware/validation.ts',
    'middleware/error-handler.ts',
    'errors/app-error.ts',
    'schemas/auth.ts',
    'schemas/task.ts',
    'types/user.ts',
    'types/task.ts'
  ];

  let content = '\n\n# Existing Codebase\n\n';
  for (const file of files) {
    const filePath = path.join(projectPath, file);
    if (fs.existsSync(filePath)) {
      content += `\n## File: src/${file}\n\`\`\`typescript\n`;
      content += fs.readFileSync(filePath, 'utf-8');
      content += '\n```\n';
    }
  }
  return content;
}

function loadTestCase(caseId: string): { prompt: string; metadata: Record<string, unknown> } {
  const casePath = path.join(ROOT, 'cases', `${caseId}.md`);
  const content = fs.readFileSync(casePath, 'utf-8');

  // Extract prompt section
  const promptMatch = content.match(/## Prompt\s*```[\s\S]*?```/);
  const prompt = promptMatch
    ? promptMatch[0].replace(/## Prompt\s*```\w*\n?/, '').replace(/```$/, '').trim()
    : '';

  return { prompt, metadata: {} };
}

async function runTest(
  caseId: string,
  group: 'control' | 'treatment',
  runNumber: number
): Promise<TestRun> {
  const config = await loadConfig();
  const context = await loadContext(group, config);
  const projectFiles = loadProjectFiles();
  const testCase = loadTestCase(caseId);

  const systemPrompt = `You are an expert TypeScript developer. You have been given documentation for a project and the existing codebase. Follow the documented patterns exactly.

${context}

${projectFiles}

When generating code:
1. Follow all patterns shown in the documentation
2. Match the existing code style exactly
3. Use the existing utilities and patterns
4. Do not introduce new patterns not in the documentation`;

  const client = new Anthropic();

  const response = await client.messages.create({
    model: config.model,
    max_tokens: config.max_tokens,
    temperature: config.temperature,
    system: systemPrompt,
    messages: [
      {
        role: 'user',
        content: testCase.prompt
      }
    ]
  });

  const output = response.content
    .filter((block) => block.type === 'text')
    .map((block) => (block as { type: 'text'; text: string }).text)
    .join('\n');

  const testRun: TestRun = {
    id: `${caseId}-${group}-${runNumber}-${Date.now()}`,
    timestamp: new Date().toISOString(),
    case_id: caseId,
    group,
    run_number: runNumber,
    output,
    tokens_used: response.usage.input_tokens + response.usage.output_tokens
  };

  // Save output
  const outputDir = path.join(ROOT, 'outputs', caseId, group);
  fs.mkdirSync(outputDir, { recursive: true });
  fs.writeFileSync(
    path.join(outputDir, `run-${runNumber}.json`),
    JSON.stringify(testRun, null, 2)
  );

  console.log(`Completed: ${caseId} / ${group} / run ${runNumber}`);
  console.log(`Tokens used: ${testRun.tokens_used}`);

  return testRun;
}

// CLI Entry Point
const args = process.argv.slice(2);
const caseArg = args.find((a) => a.startsWith('--case='));
const groupArg = args.find((a) => a.startsWith('--group='));
const runArg = args.find((a) => a.startsWith('--run='));

if (!caseArg || !groupArg) {
  console.error('Usage: npx ts-node run-test.ts --case=<case-id> --group=<control|treatment> [--run=<number>]');
  process.exit(1);
}

const caseId = caseArg.split('=')[1];
const group = groupArg.split('=')[1] as 'control' | 'treatment';
const runNumber = runArg ? parseInt(runArg.split('=')[1]) : 1;

runTest(caseId, group, runNumber).catch(console.error);

export { runTest, loadConfig, loadContext, loadProjectFiles, loadTestCase };
