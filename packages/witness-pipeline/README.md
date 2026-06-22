# @noesis/witness-pipeline

Premium integrated reading pipeline for Selemene consciousness engines. Ported from `witness-agents/` with dependency injection for testability.

## Overview

This package provides:

1. **Selemene Fetcher** — Typed client for Selemene engine API with parallel fetch
2. **Engine Routing** — Maps engines to Aletheios/Pichet/Dyad synthesis pillars
3. **Mode Parser** — Parses reading-mode Markdown documents into typed config
4. **Integrated Orchestrator** — Multi-pass reading orchestrator driven by mode documents
5. **Source Pack Factory** — Creates deterministic source packs (manifest, reading, questions)
6. **Chain Audit** — Validates engine results against deterministic-fact gates

## Installation

```bash
# From workspace root
npm install

# Or add to another package
npm install @noesis/witness-pipeline
```

## Quick Start

```typescript
import {
  fetchAllEngines,
  loadSelemeneKey,
  ENGINE_ROUTING,
  runChainAudit,
  createSourcePack,
} from '@noesis/witness-pipeline';

// 1. Fetch engine results
const apiKey = await loadSelemeneKey();
const birthData = {
  date: '1990-01-15',
  time: '14:30',
  timezone: 'America/New_York',
  latitude: 40.7128,
  longitude: -74.006,
};

const results = await fetchAllEngines(birthData, {
  api_key: apiKey!,
  engines: ['vimshottari', 'human-design', 'gene-keys', 'panchanga'],
});

// 2. Audit for deterministic gate
const audit = runChainAudit({
  personId: 'user-123',
  readingMarkdown: '...generated reading...',
  engineResults: results,
});

if (!audit.passed) {
  console.error('Audit blockers:', audit.blockers);
}

// 3. Create source pack
const pack = await createSourcePack({
  personId: 'user-123',
  readingMarkdown: '...generated reading...',
  engineResults: results,
  outputDir: './output/readings/user-123',
});

console.log('Manifest:', pack.manifest);
console.log('Paths:', pack.paths);
```

## Engine Routing

Each Selemene engine is routed to a synthesis pillar:

| Engine | Routing |
|--------|---------|
| vimshottari | aletheios-primary |
| human-design | aletheios-primary |
| enneagram | aletheios-primary |
| i-ching | aletheios-primary |
| numerology | aletheios-primary |
| biorhythm | pichet-primary |
| vedic-clock | pichet-primary |
| biofield | pichet-primary |
| face-reading | pichet-primary |
| nadabrahman | pichet-primary |
| panchanga | dyad-synthesis |
| gene-keys | dyad-synthesis |
| tarot | dyad-synthesis |
| sacred-geometry | dyad-synthesis |
| sigil-forge | dyad-synthesis |
| transits | dyad-synthesis |

**Aletheios-primary**: Structural patterns, temporal cycles, archetypal maps  
**Pichet-primary**: Somatic/vitality layer, bioelectric field, circadian rhythms  
**Dyad-synthesis**: Cross-domain engines requiring both pillars to weave

## Deterministic Gate

The audit system enforces a minimum of 3 deterministic engines for any reading:

**Deterministic engines** (reproducible from birth data alone):
- panchanga, vimshottari, human-design, gene-keys, numerology
- biorhythm, vedic-clock, transits, enneagram

**Oracle engines** (require moment-of-asking randomness):
- tarot, i-ching, sacred-geometry, sigil-forge

**Somatic engines** (require real-time body data):
- biofield, face-reading, nadabrahman

When `deterministicOnly: true`, oracle and somatic engines are blocked.

## Scripts

```bash
# Run unit tests
npm run test

# Run smoke test
npm run smoke

# Type check
npm run typecheck

# Build
npm run build
```

## API Reference

### Selemene Fetcher

```typescript
// Fetch all engines in parallel
fetchAllEngines(birthData: BirthData, opts: FetchOptions): Promise<SelemeneEngineOutput[]>

// Load API key from env or ~/.claude/.env
loadSelemeneKey(): Promise<string | undefined>

// Constants
SELEMENE_BASE_URL: string
SELEMENE_ENGINE_IDS: readonly string[]
ENGINE_ID_MAP: Record<SelemeneEngineId, WitnessEngineAlias>
ENGINE_ROUTING: Record<SelemeneEngineId, RoutingMode>
```

### Mode Parser

```typescript
// Parse a mode document from string
parseModeDocument(content: string): ParsedModeDoc

// Parse from file path
parseModeDoc(filePath: string): ParsedModeDoc

// Get pass template for a specific pass and register
getPassTemplate(doc: ParsedModeDoc, passId: string, register: RegisterBand): string

// Get target word count for a register
getTargetWordsForRegister(doc: ParsedModeDoc, register: RegisterBand): { min: number; max: number }

// Summarize lessons from mode doc
summarizeLessons(doc: ParsedModeDoc): string
```

### Orchestrator

```typescript
class IntegratedReadingOrchestrator {
  constructor(opts: OrchestratorOptions)
  run(input: OrchestratorInput): Promise<OrchestratorOutput>
}

interface OrchestratorOptions {
  mode: ParsedModeDoc;
  llm: (system: string, user: string, options: { max_tokens: number }) => Promise<string>;
}
```

### Source Pack Factory

```typescript
createSourcePack(input: SourcePackInput): Promise<SourcePack>

interface SourcePackInput {
  personId: string;
  readingMarkdown: string;
  engineResults: SelemeneEngineOutput[];
  outputDir: string;
  deterministicOnly?: boolean;
}
```

### Chain Audit

```typescript
runChainAudit(input: AuditInput): AuditResult

interface AuditResult {
  person_id: string;
  blockers: string[];
  warnings: string[];
  facts_count: number;
  passed: boolean;
}
```

## License

MIT
