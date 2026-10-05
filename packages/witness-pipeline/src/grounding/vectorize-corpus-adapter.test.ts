// ─── VectorizeCorpusAdapter — unit tests ──────────────────────────────────
// All wrangler CLI calls are intercepted via the injectable CommandRunner.
// No real wrangler or network calls are made.

import { describe, it, expect } from 'vitest';
import { VectorizeCorpusAdapter } from './vectorize-corpus-adapter.js';
import type { CommandRunner, CommandRunnerResult } from './types.js';

// ─── Helpers ─────────────────────────────────────────────────────────────────

const ACCOUNT_ID = '9d9d23b27f32e70ae3afb6a1aa2c0f10';
const INDEX = 'witness-wisdom-corpus';

function makeRunner(result: Partial<CommandRunnerResult>): CommandRunner {
  return {
    async run(_cmd, _args, _opts): Promise<CommandRunnerResult> {
      return {
        stdout: result.stdout ?? '',
        stderr: result.stderr ?? '',
        exitCode: result.exitCode ?? 0,
      };
    },
  };
}

function makeAdapter(runner: CommandRunner): VectorizeCorpusAdapter {
  return new VectorizeCorpusAdapter({
    accountId: ACCOUNT_ID,
    indexName: INDEX,
    commandRunner: runner,
  });
}

// ─── Fixtures ─────────────────────────────────────────────────────────────────

const VALID_VECTOR = {
  id: 'sw:gk:17:shadow_description',
  values: [],
  metadata: {
    system: 'gene-keys',
    category: 'archetype',
    number: 17,
    field: 'shadow_description',
    name: 'The 17th Way',
    text: 'The shadow of Opinion can manifest as rigid, defensive mindsets.',
  },
};

const VALID_QUERY_MATCH = {
  id: 'sw:hd:type:generator:desc',
  score: 0.91,
  values: [],
  metadata: {
    system: 'human-design',
    category: 'type',
    field: 'description',
    name: 'Generator',
    text: 'The Generator has a defined Sacral center and builds life force through response.',
  },
};

// ─── Success path: getVectors ─────────────────────────────────────────────────

describe('VectorizeCorpusAdapter.getVectors', () => {
  it('returns success receipt with parsed passages on valid JSON', async () => {
    const payload = { vectors: [VALID_VECTOR] };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify(payload) }));

    const receipt = await adapter.getVectors(['sw:gk:17:shadow_description']);

    expect(receipt.state).toBe('success');
    expect(receipt.passages).toHaveLength(1);
    const p = receipt.passages[0];
    expect(p.id).toBe('sw:gk:17:shadow_description');
    expect(p.strategy).toBe('direct');
    expect(p.queryAnchorId).toBeUndefined();
    expect(p.system).toBe('gene-keys');
    expect(p.text).toBeTruthy();
    expect(p.textHash).toHaveLength(64);
    expect(p.idNamespace).toBe('sw:gk');
    expect(receipt.requestedIds).toContain('sw:gk:17:shadow_description');
    expect(receipt.durationMs).toBeGreaterThanOrEqual(0);
  });

  it('handles wrangler result wrapper shape', async () => {
    const payload = { result: { vectors: [VALID_VECTOR] }, success: true };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify(payload) }));

    const receipt = await adapter.getVectors(['sw:gk:17:shadow_description']);
    expect(receipt.state).toBe('success');
    expect(receipt.passages).toHaveLength(1);
  });

  it('strips wrangler spinner prefix lines and still parses JSON', async () => {
    const json = JSON.stringify({ vectors: [VALID_VECTOR] });
    const stdout = `🔐 Fetching vectors...\nSome spinner line\n${json}`;
    const adapter = makeAdapter(makeRunner({ stdout }));

    const receipt = await adapter.getVectors(['sw:gk:17:shadow_description']);
    expect(receipt.state).toBe('success');
    expect(receipt.passages).toHaveLength(1);
  });

  it('returns empty receipt when no vectors found', async () => {
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ vectors: [] }) }));
    const receipt = await adapter.getVectors(['sw:gk:99:shadow_description']);
    expect(receipt.state).toBe('empty');
    expect(receipt.passages).toHaveLength(0);
  });

  it('returns empty receipt when all IDs are outside sw: namespace', async () => {
    const adapter = makeAdapter(makeRunner({ stdout: '{}', exitCode: 0 }));
    const receipt = await adapter.getVectors(['report-pattern:abc123', 'rp:xyz']);
    expect(receipt.state).toBe('empty');
    expect(receipt.passages).toHaveLength(0);
    expect(receipt.reason).toMatch(/namespace filter/);
  });

  it('returns auth_error state on 401 stderr', async () => {
    const adapter = makeAdapter(
      makeRunner({ stdout: '', stderr: 'Unauthorized: 401 Invalid API token', exitCode: 1 }),
    );
    const receipt = await adapter.getVectors(['sw:gk:1:shadow_description']);
    expect(receipt.state).toBe('auth_error');
    expect(receipt.passages).toHaveLength(0);
    expect(receipt.exitCode).toBe(1);
  });

  it('returns malformed state when stdout is not parseable JSON', async () => {
    const adapter = makeAdapter(makeRunner({ stdout: 'NOT JSON at all >>><<<' }));
    const receipt = await adapter.getVectors(['sw:gk:1:shadow_description']);
    expect(receipt.state).toBe('malformed');
    expect(receipt.passages).toHaveLength(0);
  });

  it('rejects private subject entry in metadata', async () => {
    const privateVector = {
      id: 'sw:gk:17:shadow_description',
      values: [],
      metadata: {
        system: 'gene-keys',
        category: 'archetype',
        field: 'shadow_description',
        name: 'Test',
        text: 'Some text born_at: 1990-05-15 latitude: 28.6139',
        birth_date: '1990-05-15',
      },
    };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ vectors: [privateVector] }) }));
    const receipt = await adapter.getVectors(['sw:gk:17:shadow_description']);
    // passage rejected by private subject filter
    expect(receipt.passages).toHaveLength(0);
    expect(receipt.state).toBe('empty');
  });

  it('rejects generic placeholder text irrespective of gate number', async () => {
    const genericVector58 = {
      id: 'sw:hd:g:58:desc',
      values: [],
      metadata: { system: 'human-design', category: 'gate', field: 'description', name: 'Gate 58', text: 'Authentic Human Design gate 58 representing specific life themes and energy patterns.' },
    };
    const genericVector64 = {
      id: 'sw:hd:g:64:desc',
      values: [],
      metadata: { system: 'human-design', category: 'gate', field: 'description', name: 'Gate 64', text: 'Authentic Human Design gate 64 representing specific life themes and energy patterns.' },
    };
    const adapter = makeAdapter(
      makeRunner({ stdout: JSON.stringify({ vectors: [genericVector58, genericVector64] }) }),
    );
    const receipt = await adapter.getVectors(['sw:hd:g:58:desc', 'sw:hd:g:64:desc']);
    expect(receipt.passages).toHaveLength(0);
    expect(receipt.state).toBe('empty');
  });

  it('rejects live Gene Keys category boilerplate while retaining specific interpretation', async () => {
    const boilerplate = [
      ['shadow_description', 'The shadow frequency represents the unconscious pattern that creates limitation and suffering in this area of life.'],
      ['gift_description', 'The gift frequency expresses the balanced state of consciousness that serves the collective good.'],
      ['siddhi_description', 'The siddhi frequency embodies the highest potential of human consciousness in this archetypal pattern.'],
      ['life_theme', 'Transforming unconscious patterns into conscious service'],
    ].map(([field, text]) => ({ id: `sw:gk:49:${field}`, metadata: { system: 'gene-keys', category: 'archetype', field, text } }));
    const specific = { id: 'sw:gk:4:life_theme', metadata: { system: 'gene-keys', category: 'archetype', field: 'life_theme', text: 'Developing tolerance through deeper understanding' } };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ vectors: [...boilerplate, specific] }) }));
    const receipt = await adapter.getVectors([...boilerplate, specific].map(v => v.id));
    expect(receipt.passages.map(p => p.id)).toEqual([specific.id]);
    expect(receipt.state).toBe('success');
  });

  it('does not retain vector embedding values in passage receipt', async () => {
    const vectorWithValues = {
      ...VALID_VECTOR,
      values: Array.from({ length: 1024 }, (_, i) => i * 0.001),
    };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ vectors: [vectorWithValues] }) }));
    const receipt = await adapter.getVectors(['sw:gk:17:shadow_description']);
    expect(receipt.state).toBe('success');
    const p = receipt.passages[0];
    expect((p as unknown as Record<string, unknown>)['values']).toBeUndefined();
  });
});

// ─── Success path: queryByVectorId ───────────────────────────────────────────

describe('VectorizeCorpusAdapter.queryByVectorId', () => {
  it('returns success receipt with semantic provenance', async () => {
    const payload = { matches: [VALID_QUERY_MATCH] };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify(payload) }));

    const receipt = await adapter.queryByVectorId('sw:gk:17:shadow_description', 5);
    expect(receipt.state).toBe('success');
    expect(receipt.passages).toHaveLength(1);
    const p = receipt.passages[0];
    expect(p.strategy).toBe('semantic');
    expect(p.queryAnchorId).toBe('sw:gk:17:shadow_description');
    expect(p.score).toBe(0.91);
    expect(p.id).toBe('sw:hd:type:generator:desc');
  });

  it('handles result wrapper shape from wrangler', async () => {
    const payload = { result: { matches: [VALID_QUERY_MATCH] }, success: true };
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify(payload) }));
    const receipt = await adapter.queryByVectorId('sw:gk:17:shadow_description');
    expect(receipt.state).toBe('success');
    expect(receipt.passages[0].strategy).toBe('semantic');
  });

  it('rejects anchor ID outside sw: namespace', async () => {
    const adapter = makeAdapter(makeRunner({ stdout: '{}' }));
    const receipt = await adapter.queryByVectorId('report-pattern:abc');
    expect(receipt.state).toBe('empty');
    expect(receipt.reason).toMatch(/corpus namespace/);
    expect(receipt.passages).toHaveLength(0);
  });

  it('filters non-sw: matches from semantic query results', async () => {
    const mixedMatches = [
      VALID_QUERY_MATCH,
      { id: 'report-pattern:xyz', score: 0.88, values: [], metadata: { text: 'outside namespace' } },
    ];
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ matches: mixedMatches }) }));
    const receipt = await adapter.queryByVectorId('sw:gk:17:shadow_description');
    expect(receipt.state).toBe('success');
    expect(receipt.passages).toHaveLength(1);
    expect(receipt.passages[0].id).toBe('sw:hd:type:generator:desc');
  });

  it('returns empty state when no valid matches', async () => {
    const adapter = makeAdapter(makeRunner({ stdout: JSON.stringify({ matches: [] }) }));
    const receipt = await adapter.queryByVectorId('sw:gk:1:shadow_description');
    expect(receipt.state).toBe('empty');
  });

  it('returns auth_error on 403 stderr', async () => {
    const adapter = makeAdapter(
      makeRunner({ stdout: '', stderr: 'Error: Forbidden 403', exitCode: 1 }),
    );
    const receipt = await adapter.queryByVectorId('sw:gk:1:shadow_description');
    expect(receipt.state).toBe('auth_error');
  });
});


describe('installed Wrangler and Vectorize contract', () => {
  it('batches at twenty IDs and passes account only through the process environment', async () => {
    const sizes: number[] = [];
    const adapter = makeAdapter({ async run(command, args, options) {
      expect(command).toBe('wrangler');
      expect(args).not.toContain('--account-id');
      expect(args).not.toContain('--format');
      expect(options?.env?.CLOUDFLARE_ACCOUNT_ID).toBe(ACCOUNT_ID);
      const ids = args.slice(args.indexOf('--ids') + 1);
      sizes.push(ids.length);
      return { stdout: JSON.stringify(ids.map(id => ({ id, metadata: { text: 'An attributed framework passage.' } }))), stderr: '', exitCode: 0 };
    }});
    const result = await adapter.getVectors(Array.from({ length: 35 }, (_, i) => `sw:gk:${i + 1}:life_theme`));
    expect(sizes).toEqual([20, 15]);
    expect(result.passages).toHaveLength(35);
  });
  it('rejects blank passages and generic gate 34 rather than using its ID as knowledge', async () => {
    const vectors = [{ id: 'sw:hd:g:34:desc', metadata: { text: 'Authentic Human Design gate 34 representing specific life themes and energy patterns.' } }, { id: 'sw:gk:1:life_theme', metadata: {} }];
    const result = await makeAdapter(makeRunner({ stdout: JSON.stringify(vectors) })).getVectors(vectors.map(v => v.id));
    expect(result.state).toBe('empty');
  });
});

describe('transient CF authentication reads', () => {
  it('retries once without changing account and retains both attempt states', async () => {
    let calls = 0;
    const adapter = makeAdapter({ async run(_command, _args, options) {
      expect(options?.env?.CLOUDFLARE_ACCOUNT_ID).toBe(ACCOUNT_ID);
      calls++;
      return calls === 1 ? { stdout: '', stderr: 'Authentication error [code: 10000]', exitCode: 1 }
        : { stdout: JSON.stringify([VALID_VECTOR]), stderr: '', exitCode: 0 };
    }});
    const result = await adapter.getVectors([VALID_VECTOR.id]);
    expect(calls).toBe(2);
    expect(result.state).toBe('success');
    expect(result.commandAttempts?.map(a => a.state)).toEqual(['auth_error', 'success']);
  });
});
