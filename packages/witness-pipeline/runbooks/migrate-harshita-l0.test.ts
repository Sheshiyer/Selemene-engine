import { describe, it, expect } from 'vitest';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { existsSync, readFileSync } from 'node:fs';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);

const SCRIPT_PATH = fileURLToPath(
  new URL('../../../runbooks/migrate-harshita-l0.sh', import.meta.url),
);

type Mapping = {
  sourcePath: string;
  destinationPath: string;
  contents: string;
};

const FILE_MAPPINGS: readonly Mapping[] = [
  {
    sourcePath: 'source-pack/manifest.json',
    destinationPath: 'source-pack/manifest.json',
    contents: 'manifest\n',
  },
  {
    sourcePath: 'source-pack/reading.md',
    destinationPath: 'source-pack/reading.md',
    contents: '# Reading sample\n',
  },
  {
    sourcePath: 'source-pack/reflection-questions.md',
    destinationPath: 'source-pack/reflection-questions.md',
    contents: '# Reflection questions\n',
  },
  {
    sourcePath: 'engines.json',
    destinationPath: 'source-pack/engines.json',
    contents: '{"name":"harshita"}\n',
  },
  {
    sourcePath: 'report.html',
    destinationPath: 'local/reading.html',
    contents: '<html><body>reading</body></html>\n',
  },
  {
    sourcePath: 'report.pdf',
    destinationPath: 'local/reading.pdf',
    contents: '%PDF-1.4\n',
  },
];

const createSourceFixture = async (
  sourceRoot: string,
  missingSourcePath?: string,
) => {
  for (const { sourcePath, contents } of FILE_MAPPINGS) {
    if (sourcePath === missingSourcePath) {
      continue;
    }

    const absoluteSourcePath = join(sourceRoot, sourcePath);
    await mkdir(dirname(absoluteSourcePath), { recursive: true });
    await writeFile(absoluteSourcePath, contents, 'utf8');
  }
};

const runMigration = (sourceRoot: string, destinationRoot: string) =>
  execFileAsync('/bin/bash', [SCRIPT_PATH], {
    cwd: sourceRoot,
    env: {
      ...process.env,
      SELEMENE_MIGRATION_SOURCE: sourceRoot,
      SELEMENE_MIGRATION_DESTINATION: destinationRoot,
    },
  });

const expectMigratedContents = (destinationRoot: string) => {
  for (const { destinationPath, contents } of FILE_MAPPINGS) {
    expect(readFileSync(join(destinationRoot, destinationPath), 'utf8')).toBe(contents);
  }
};

describe('migrate-harshita-l0', () => {
  it('copies configured source artifacts to the configured destination', async () => {
    const workspace = await mkdtemp(join(tmpdir(), 'migrate-harshita-l0-'));
    const sourceRoot = join(workspace, 'source');
    const destinationRoot = join(workspace, 'destination');

    try {
      await createSourceFixture(sourceRoot);
      await runMigration(sourceRoot, destinationRoot);
      expectMigratedContents(destinationRoot);
    } finally {
      await rm(workspace, { recursive: true, force: true });
    }
  });

  it('fails with a missing required source file and leaves destination absent', async () => {
    const workspace = await mkdtemp(join(tmpdir(), 'migrate-harshita-l0-'));
    const sourceRoot = join(workspace, 'source');
    const destinationRoot = join(workspace, 'destination');

    try {
      await createSourceFixture(sourceRoot, FILE_MAPPINGS[0].sourcePath);

      await expect(runMigration(sourceRoot, destinationRoot)).rejects.toMatchObject({
        code: 1,
      });

      expect(existsSync(destinationRoot)).toBe(false);
    } finally {
      await rm(workspace, { recursive: true, force: true });
    }
  });
});
