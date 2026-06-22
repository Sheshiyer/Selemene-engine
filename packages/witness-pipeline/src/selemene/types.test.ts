import { describe, it, expect } from 'vitest';
import { ENGINE_ROUTING, ENGINE_ID_MAP, REVERSE_ENGINE_MAP, type SelemeneEngineId } from './types.js';

describe('engine routing', () => {
  it('routes vimshottari as aletheios-primary', () => {
    expect(ENGINE_ROUTING['vimshottari']).toBe('aletheios-primary');
  });

  it('routes biofield as pichet-primary', () => {
    expect(ENGINE_ROUTING['biofield']).toBe('pichet-primary');
  });

  it('routes panchanga as dyad-synthesis', () => {
    expect(ENGINE_ROUTING['panchanga']).toBe('dyad-synthesis');
  });

  it('maps every Selemene id to a witness alias and back', () => {
    const ids = Object.keys(ENGINE_ROUTING) as SelemeneEngineId[];
    for (const id of ids) {
      const alias = ENGINE_ID_MAP[id];
      expect(alias).toBeDefined();
      expect(REVERSE_ENGINE_MAP[alias]).toBe(id);
    }
  });
});
