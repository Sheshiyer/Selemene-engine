import { describe, it, expect } from 'vitest';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const execFileAsync = promisify(execFile);

const SCRIPT = fileURLToPath(new URL('../../../runbooks/migrate-harshita-l0.sh', import.meta.url));

function writeSyntheticNewL0Flow(src: string): void {
  mkdirSync(join(src, 'source-pack'), { recursive: true });
  writeFileSync(join(src, 'source-pack/manifest.json'), JSON.stringify({ person_id: 'synthetic-person' }));
  writeFileSync(join(src, 'source-pack/reading.md'), '# Synthetic reading');
  writeFileSync(join(src, 'source-pack/reflection-questions.md'), '- Synthetic question?');
  writeFileSync(join(src, 'engines.json'), JSON.stringify([]));
  writeFileSync(join(src, 'report.html'), '<html><body>Synthetic</body></html>');
  writeFileSync(join(src, 'report.pdf'), '%PDF-1.4 synthetic');
}

describe('migrate-harshita-l0', () => {
  it('copies new-l0-flow artifacts to the canonical factory path', async () => {
    const root = mkdtempSync(join(tmpdir(), 'migrate-l0-'));
    try {
      const src = join(root, 'new-l0-flow');
      const dst = join(root, '.premium-assets-witness-synthetic-person', 'synthetic-person');
      writeSyntheticNewL0Flow(src);

      await execFileAsync('bash', [SCRIPT, src, dst]);

      expect(existsSync(`${dst}/source-pack/manifest.json`)).toBe(true);
      expect(existsSync(`${dst}/source-pack/reading.md`)).toBe(true);
      expect(existsSync(`${dst}/source-pack/reflection-questions.md`)).toBe(true);
      expect(existsSync(`${dst}/source-pack/engines.json`)).toBe(true);
      expect(existsSync(`${dst}/local/reading.html`)).toBe(true);
      expect(existsSync(`${dst}/local/reading.pdf`)).toBe(true);
      expect(readFileSync(`${dst}/local/reading.pdf`, 'latin1')).toBe('%PDF-1.4 synthetic');
    } finally {
      rmSync(root, { recursive: true, force: true });
    }
  });

  it('refuses to run without explicit source and destination', async () => {
    await expect(execFileAsync('bash', [SCRIPT])).rejects.toMatchObject({ code: 2 });
  });
});
