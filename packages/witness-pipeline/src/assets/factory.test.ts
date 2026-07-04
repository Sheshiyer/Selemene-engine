import { describe, it, expect } from 'vitest';
import { createSourcePack } from './factory.js';
import type { SelemeneEngineOutput } from '../index.js';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

function makeEngine(id: string, deterministic = true, error?: string): SelemeneEngineOutput {
  return {
    engine_id: id as SelemeneEngineOutput['engine_id'],
    result: {},
    witness_prompt: 'x',
    consciousness_level: 2,
    metadata: {
      calculation_time_ms: 1,
      backend: deterministic ? 'native' : 'ts',
      precision_achieved: 'standard',
      cached: false,
      timestamp: '2026-06-22T00:00:00Z',
      engine_version: '1',
    },
    envelope_version: '1',
    _error: error,
  };
}

describe('createSourcePack', () => {
  it('creates a manifest and reflection questions', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'witness-pack-'));
    try {
      const engines: SelemeneEngineOutput[] = [
        makeEngine('panchanga'),
        makeEngine('vimshottari'),
        makeEngine('human-design'),
      ];
      const pack = await createSourcePack({
        personId: 'test-person',
        readingMarkdown: '# Reading',
        engineResults: engines,
        outputDir: dir,
      });
      expect(pack.manifest.person_id).toBe('test-person');
      expect(pack.manifest.quality.facts_count).toBe(3);
      expect(pack.manifest.quality.gate_status).toBe('ready');
      expect(pack.reflectionQuestions.length).toBeGreaterThan(0);
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it('blocks when fewer than 3 deterministic engines', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'witness-pack-'));
    try {
      const pack = await createSourcePack({
        personId: 'test-person',
        readingMarkdown: '# Reading',
        engineResults: [makeEngine('panchanga')],
        outputDir: dir,
      });
      expect(pack.manifest.quality.gate_status).toBe('blocked');
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it('stores pattern learning provenance when provided', async () => {
    const dir = mkdtempSync(join(tmpdir(), 'source-pack-learning-'));
    try {
      const pack = await createSourcePack({
        personId: 'p1',
        readingMarkdown: 'reading text',
        engineResults: [makeEngine('panchanga'), makeEngine('vimshottari'), makeEngine('human-design')],
        outputDir: dir,
        patternLearning: { extracted: 2, upserted: 1, skipped: 1 },
      });

      expect(pack.manifest.quality.pattern_learning).toEqual({ extracted: 2, upserted: 1, skipped: 1 });
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });
});
