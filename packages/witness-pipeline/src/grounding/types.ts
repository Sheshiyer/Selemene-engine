// ─── Grounding / Vectorize Corpus Adapter — Types ─────────────────────────
// All types for the read-only Cloudflare Vectorize corpus adapter.
// The adapter never writes to Vectorize; all operations are via wrangler CLI.
// No subject names or birth data are ever transmitted to the CLI.

// ─── Corpus passage metadata (mirrors ingest-wisdom-corpus.ts PassageMetadata)

export interface CorpusPassageMetadata {
  system: string;
  category: string;
  number?: number;
  field: string;
  name: string;
  text: string;
}

// ─── Raw wrangler vector record (as returned by get-vectors / query)

export interface WranglerVector {
  id: string;
  values?: number[];
  metadata?: CorpusPassageMetadata | Record<string, unknown>;
}

export interface WranglerGetVectorsResponse {
  vectors?: WranglerVector[];
  // wrangler sometimes wraps in result/success
  result?: {
    vectors?: WranglerVector[];
  };
}

export interface WranglerQueryMatch {
  id: string;
  score: number;
  values?: number[];
  metadata?: CorpusPassageMetadata | Record<string, unknown>;
}

export interface WranglerQueryResponse {
  matches?: WranglerQueryMatch[];
  result?: {
    matches?: WranglerQueryMatch[];
  };
  count?: number;
}

// ─── Corpus receipt — what the adapter returns per retrieval

export type CorpusReceiptState = 'success' | 'empty' | 'auth_error' | 'malformed' | 'timeout' | 'failure';

export interface CorpusPassage {
  /** Verbatim corpus ID — e.g. sw:gk:17:shadow_description */
  id: string;
  text: string;
  /** Full SHA-256 hex of the text */
  textHash: string;
  /** e.g. 'sw:gk' | 'sw:hd' | 'sw:vim' etc */
  idNamespace: string;
  /** System key: gene-keys | human-design | vimshottari | etc */
  system: string;
  /** category from metadata */
  category: string;
  /** field from metadata */
  field: string;
  /** Similarity score when retrieved via query; undefined for direct get */
  score?: number;
  /** How this passage was found: 'direct' = get-vectors; 'semantic' = query --vector-id */
  strategy: 'direct' | 'semantic';
  /** Provenance anchor — the corpus vector ID that was used as the query seed for 'semantic' */
  queryAnchorId?: string;
}

export interface CorpusReceipt {
  commandAttempts?: Array<{ exitCode: number; state: CorpusReceiptState }>;
  state: CorpusReceiptState;
  passages: CorpusPassage[];
  requestedIds: string[];
  /** Timestamps for latency observability */
  durationMs: number;
  reason?: string;
  /** Raw wrangler exit code if non-zero */
  exitCode?: number;
}

// ─── Adapter configuration

export interface VectorizeCorpusAdapterOptions {
  /** CF account ID — must be passed per-call, never from a module-level global */
  accountId: string;
  /** Vectorize index name (default: witness-wisdom-corpus) */
  indexName?: string;
  /** Timeout in ms for each wrangler invocation (default: 15000) */
  timeoutMs?: number;
  /** Injectable command runner — required for tests */
  commandRunner?: CommandRunner;
}

// ─── Injectable command runner interface

export interface CommandRunnerResult {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export interface CommandRunner {
  run(
    command: string,
    args: string[],
    options?: { timeoutMs?: number; env?: NodeJS.ProcessEnv },
  ): Promise<CommandRunnerResult>;
}

// ─── Corpus ID selector input — pure engine outputs, no birth data

export interface HumanDesignEngineOutput {
  type?: string;            // e.g. 'Generator'
  profile?: string;         // e.g. '2/4' or '2_4'
  authority?: string;       // e.g. 'Sacral_Authority'
  defined_centers?: string[];
  channels?: Array<{ channel?: string; gates?: [number, number] }> | string[];
}

export interface GeneKeysEngineOutput {
  gates?: Array<{ number: number; line?: number }> | number[];
  profile_sequence?: number[];
}

export interface VimshottariEngineOutput {
  current_dasha?: string;   // planet name
  current_antardasha?: string;
  nakshatra?: string | { number: number; name: string };
  nakshatra_number?: number;
}

export interface EngineOutputsForGrounding {
  'human-design'?: HumanDesignEngineOutput;
  'gene-keys'?: GeneKeysEngineOutput;
  'vimshottari'?: VimshottariEngineOutput;
  [key: string]: unknown;
}

// ─── Selector output

export interface SelectedCorpusIds {
  /** Ordered list of corpus IDs to fetch — no subject identifiers */
  ids: string[];
  /** Short rationale per ID for audit log */
  rationale: Record<string, string>;
}
