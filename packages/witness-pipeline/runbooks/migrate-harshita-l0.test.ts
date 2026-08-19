import { describe, it, expect } from 'vitest';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { existsSync } from 'node:fs';
import { rm } from 'node:fs/promises';

const execFileAsync = promisify(execFile);

const WORKTREE_ROOT = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/Selemene-engine-worktrees/plan-l0-kundali';
const SCRIPT = `${WORKTREE_ROOT}/runbooks/migrate-harshita-l0.sh`;
const DST = '/Volumes/madara/2026/twc-vault/01-Projects/tryambakam-noesis/723/witness-agents-archive/.premium-assets-witness-harshita/harshita';

describe('migrate-harshita-l0', () => {
  it('copies new-l0-flow artifacts to the canonical factory path', async () => {
    if (existsSync(DST)) {
      await rm(DST, { recursive: true, force: true });
    }

    await execFileAsync('bash', [SCRIPT], { cwd: WORKTREE_ROOT });

    expect(existsSync(`${DST}/source-pack/manifest.json`)).toBe(true);
    expect(existsSync(`${DST}/source-pack/reading.md`)).toBe(true);
    expect(existsSync(`${DST}/source-pack/reflection-questions.md`)).toBe(true);
    expect(existsSync(`${DST}/source-pack/engines.json`)).toBe(true);
    expect(existsSync(`${DST}/local/reading.html`)).toBe(true);
    expect(existsSync(`${DST}/local/reading.pdf`)).toBe(true);
  });
});
