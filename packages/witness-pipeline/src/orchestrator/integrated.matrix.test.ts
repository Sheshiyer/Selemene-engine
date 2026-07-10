import { describe, it, expect, vi } from 'vitest';
import { IntegratedReadingOrchestrator } from './integrated.js';
import { parseModeDoc } from '../modes/parser.js';
import { createSourcePack } from '../assets/factory.js';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import type { SelemeneEngineOutput } from '../index.js';

const mockEngines: SelemeneEngineOutput[] = [
  { engine_id: 'panchanga', result: { tithi_name: 'Test' }, witness_prompt: 'Observe.', consciousness_level: 2, metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' }, envelope_version: '1' },
  { engine_id: 'vimshottari', result: { dasha: 'Test' }, witness_prompt: 'Observe.', consciousness_level: 2, metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' }, envelope_version: '1' },
  { engine_id: 'human-design', result: { type: 'Generator' }, witness_prompt: 'Observe.', consciousness_level: 2, metadata: { calculation_time_ms: 1, backend: 'native', precision_achieved: 'standard', cached: false, timestamp: '2026-06-22T00:00:00Z', engine_version: '1' }, envelope_version: '1' },
];

// Minimal inline fixtures
const motherSonReq = {
  subjects: [
    { role: 'mother', name: 'Aarav', birth_date: '1970-01-01', birth_time_confidence: 'exact' as const, birth_location_query: 'Bengaluru, India' },
    { role: 'son', name: 'Vikram', birth_date: '1995-01-01', birth_time_confidence: 'exact' as const, birth_location_query: 'Bengaluru, India' },
  ],
  relationship_context: { type: 'family', mapping_goal: 'understand lineage', sensitivity_level: 'high' as const },
};
const businessReq = {
  subjects: [
    { role: 'business-partner', name: 'Priya', relationship_label: 'CEO', birth_date: '1985-01-01', birth_time_confidence: 'exact' as const, birth_location_query: 'Mumbai' },
    { role: 'business-partner', name: 'Rahul', relationship_label: 'CTO', birth_date: '1986-01-01', birth_time_confidence: 'exact' as const, birth_location_query: 'Mumbai' },
  ],
  relationship_context: { type: 'business-partners', mapping_goal: 'map decisions', sensitivity_level: 'medium' as const },
};
const familyPentaReq = {
  subjects: [
    { role: 'mother', name: 'L' }, { role: 'father', name: 'R' }, { role: 'child1', name: 'A' }, { role: 'child2', name: 'B' }, { role: 'child3', name: 'C' },
  ].map(s => ({ ...s, birth_date: '1990-01-01', birth_time_confidence: 'exact' as const, birth_location_query: 'Chennai' })),
  relationship_context: { type: 'family', mapping_goal: 'witness field', sensitivity_level: 'high' as const },
};

const combos = [
  { name: 'mother-son L2 family', modeFile: 'mother-son-lineage.md', level: 2, req: motherSonReq, relType: 'family', expectHeader: true },
  { name: 'business L2 business-partners', modeFile: 'business-partners.md', level: 2, req: businessReq, relType: 'business-partners', expectHeader: true },
  { name: 'family-penta L3 family', modeFile: 'family-penta.md', level: 2, req: familyPentaReq, relType: 'family', expectHeader: true },
  { name: 'integrated-reading L3 solo (no rel)', modeFile: 'integrated-reading.md', level: 2, req: null, relType: null, expectHeader: false },
  { name: 'birth-blueprint L4/L5 solo', modeFile: 'birth-blueprint.md', level: 4, req: null, relType: null, expectHeader: false },
];

describe('cross-product matrix (mode x level x relationship_type)', () => {
  for (const c of combos) {
    it(`runs ${c.name} without crash, header when applicable, guardrails pass, createSourcePack succeeds`, async () => {
      const modePath = resolve(__dirname, `../../modes/${c.modeFile}`);
      const mode = parseModeDoc(modePath);

      const llm = vi.fn().mockResolvedValue('Clean descriptive pattern witness output. No prediction. No diagnosis. Observable facts only.');

      const orchestrator = new IntegratedReadingOrchestrator({ mode, llm });

      let input: any;
      if (c.req) {
        input = {
          subjectNames: c.req.subjects.map((s: any) => s.name),
          subjectRoles: c.req.subjects.map((s: any) => ({ role: s.role, name: s.name, label: s.relationship_label })),
          relationshipContext: c.req.relationship_context,
          engineResultsBySubject: c.req.subjects.length > 1 ? Array.from({ length: c.req.subjects.length }, () => mockEngines) : [mockEngines],
          consciousnessLevel: c.level,
          language: 'en',
        };
      } else {
        input = {
          subjectNames: ['Subject'],
          engineResultsBySubject: [mockEngines],
          consciousnessLevel: c.level,
          language: 'en',
        };
      }

      const result = await orchestrator.run(input);

      expect(result.mode).toBeDefined();
      expect(result.passes.length).toBeGreaterThan(0);

      if (c.expectHeader) {
        expect(result.relationship_header).toBeDefined();
        // Phase 6 Folio header is declarative (not raw type slug)
        // e.g. "Mother-Son Lineage Mapping — non-predictive pattern witness"
        expect(result.relationship_header).toMatch(/— non-predictive pattern witness/);
        expect(result.assembled.startsWith(result.relationship_header!)).toBe(true);
        // Basic Folio typography cue: markdown # heading for ink-iron style
        expect(result.relationship_header!.startsWith('# ')).toBe(true);
      } else {
        expect(result.relationship_header).toBeUndefined();
      }

      expect(result.passes.every((p) => p.rubric.guardrail_gate === 'pass')).toBe(true);

      const dir = mkdtempSync(join(tmpdir(), 'matrix-'));
      try {
        const pack = await createSourcePack({
          personId: `matrix-${c.name.replace(/\s+/g, '-')}`,
          readingMarkdown: result.assembled,
          engineResults: input.engineResultsBySubject.flat(),
          outputDir: dir,
        });
        expect(pack.manifest.quality.gate_status).toBe('ready');
      } finally {
        rmSync(dir, { recursive: true, force: true });
      }
    });
  }
});
