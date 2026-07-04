import type { ExtractedPattern } from './types.js';

export interface PatternVectorStoreResult {
  upserted: number;
  skipped: number;
}

export interface PatternVectorStore {
  upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult>;
}

export class NoopPatternVectorStore implements PatternVectorStore {
  async upsertPatterns(patterns: ExtractedPattern[]): Promise<PatternVectorStoreResult> {
    return { upserted: 0, skipped: patterns.length };
  }
}
