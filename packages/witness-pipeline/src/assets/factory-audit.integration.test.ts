import { describe, it, expect } from 'vitest';
import { createSourcePack } from './factory.js';
import { runChainAudit } from './audit.js';
import type { SelemeneEngineOutput } from '../index.js';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

function makeEngine(id: string): SelemeneEngineOutput {
  return {
    engine_id: id as SelemeneEngineOutput['engine_id'],
    result: {},
    witness_prompt: 'x',
    consciousness_level: 2,
    metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' },
    envelope_version: '1',
  };
}

describe('Factory + Audit integration', () => {
  it('creates pack then passes audit', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'witness-pack-'));
    try {
      const engines = [makeEngine('panchanga'), makeEngine('vimshottari'), makeEngine('human-design')];
      const pack = await createSourcePack({
        personId: 'int-test',
        readingMarkdown: 'A sufficiently long reading that will pass the audit length gate without any warnings being triggered at all.',
        engineResults: engines,
        outputDir: dir,
      });
      const audit = runChainAudit({
        personId: 'int-test',
        readingMarkdown: 'A sufficiently long reading that will pass the audit length gate without any warnings being triggered at all.',
        engineResults: engines,
      });
      expect(pack.manifest.quality.gate_status).toBe('ready');
      expect(audit.passed).toBe(true);
      expect(audit.facts_count).toBe(3);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
