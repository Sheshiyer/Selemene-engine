// ─── Grounding — Public API ───────────────────────────────────────────────
// Read-only Cloudflare Vectorize corpus adapter for witness-wisdom-corpus.
// Do NOT re-export from the package's main src/index.ts to keep scope narrow.

export type {
  CorpusPassage,
  CorpusReceipt,
  CorpusReceiptState,
  CorpusPassageMetadata,
  VectorizeCorpusAdapterOptions,
  CommandRunner,
  CommandRunnerResult,
  EngineOutputsForGrounding,
  HumanDesignEngineOutput,
  GeneKeysEngineOutput,
  VimshottariEngineOutput,
  SelectedCorpusIds,
} from './types.js';

export { VectorizeCorpusAdapter, createVectorizeCorpusAdapter } from './vectorize-corpus-adapter.js';
export { selectCorpusIds } from './corpus-id-selector.js';
