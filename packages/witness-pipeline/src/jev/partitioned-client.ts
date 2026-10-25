// ─── Bounded, auditable Jev context-partition adapter ───────────────
// Groups complete source records by engine, subdividing between records
// and complete framework passages when needed. Every batch retains the
// full section and compact subject index. Conservative aggregation:
// worst-case wins. Evidence callbacks persist the complete review context.

import { createHash } from 'node:crypto';
import {
  score,
  JEV_MODEL,
  type ChoiceResponse,
  type JevClient,
  type JevRequest,
  type JevResult,
  type NoulResponse,
  type Question,
  type ScoreResponse,
} from './client.js';

// ─── Public types ────────────────────────────────────────────────────

export interface PartitionedJevClientOptions {
  /** Maximum serialized wire-body chars per batch request. Default 60 000. */
  maxRequestChars?: number;
  /**
   * Called with full evidence after all batches complete (or on failure).
   * The parent saves this as a sidecar; the adapter never writes files.
   */
  onEvidence?: (evidence: PartitionEvidence) => void;
}

export interface SourceRecord {
  engine_id: string;
  result: unknown;
  metadata?: unknown;
  error?: unknown;
}

export interface SubjectRecords {
  subject: string;
  records: SourceRecord[];
}

export interface PartitionEvidence {
  /** sha-256 of the original full JevRequest JSON */
  originalRequestHash: string;
  originalWireBodyHash?: string;
  originalWireBodyBytes?: number;
  /** The original JevRequest that entered the adapter */
  originalRequest: JevRequest;
  /** Per-batch evidence */
  batches: BatchEvidence[];
  /** Coverage: every subject+engine_id that appeared in the original */
  coverageIndex: Record<string, string[]>;
  /** Conservative aggregation result, if all batches succeeded */
  aggregatedResult?: JevResult;
  /** Structured failure, if any batch failed or aggregation rejected */
  failure?: PartitionFailure;
}

export interface BatchEvidence {
  batchId: string;
  /** Repeated record context across framework slices counts once for record coverage. */
  recordPartitionId?: string;
  scope: 'overview' | string; // engine_id or 'overview'
  subjects: string[];
  engineIds: string[];       // empty for overview
  requestHash: string;
  /** Hash and byte count of the exact JSON body serialized by the base client. */
  wireBodyHash: string;
  wireBodyBytes: number;
  request: JevRequest;
  response?: JevResult;
  error?: string;
}

export interface PartitionFailure {
  kind: 'size_exceeded' | 'batch_error' | 'aggregation_error' | 'malformed_answer';
  message: string;
  batchId?: string;
  detail?: unknown;
}

// ─── Internal helpers ────────────────────────────────────────────────

const FULL_RECORDS_PREAMBLE = '## Full supplied source records';
const DEFAULT_MAX_REQUEST_CHARS = 60_000;

function sha256(data: string): string {
  return createHash('sha256').update(data).digest('hex');
}

function serializeSize(obj: unknown): number {
  return JSON.stringify(obj).length;
}

function wireBody(request: JevRequest): string {
  return JSON.stringify({ state: request.state, model: request.model ?? JEV_MODEL, questions: request.questions });
}

/**
 * Parse engine_facts into { compactPrefix, fullRecordsPreamble, subjects }.
 * compactPrefix = everything before `## Full supplied source records`.
 * fullRecordsPreamble = the instruction paragraph after the header, before the JSON array.
 * subjects = parsed SubjectRecords[] from the JSON array at the end.
 */
function parseEngineFacts(engineFacts: string): {
  compactPrefix: string;
  fullRecordsPreamble: string;
  subjects: SubjectRecords[];
} {
  const preambleIdx = engineFacts.indexOf(FULL_RECORDS_PREAMBLE);
  if (preambleIdx < 0) {
    // No full records section — nothing to partition.
    return { compactPrefix: engineFacts, fullRecordsPreamble: '', subjects: [] };
  }
  const compactPrefix = engineFacts.slice(0, preambleIdx);
  const afterHeader = engineFacts.slice(preambleIdx);

  // Find the start of the JSON array.
  const jsonStart = afterHeader.indexOf('[');
  if (jsonStart < 0) {
    return { compactPrefix, fullRecordsPreamble: afterHeader, subjects: [] };
  }
  const fullRecordsPreamble = afterHeader.slice(0, jsonStart);
  const jsonStr = afterHeader.slice(jsonStart);
  const subjects: SubjectRecords[] = JSON.parse(jsonStr);
  return { compactPrefix, fullRecordsPreamble, subjects };
}

/**
 * Rebuild engine_facts for a specific engine_id batch:
 * compactPrefix + preamble + JSON array of only that engine's records per subject.
 */
function buildBatchEngineFacts(
  compactPrefix: string,
  preamble: string,
  batchSubjects: SubjectRecords[],
): string {
  return compactPrefix + preamble + JSON.stringify(batchSubjects);
}

/**
 * Build the overview batch subjects: each subject with an empty records array.
 * The overview gets the compact prefix (which already contains all the
 * named summaries) but no full raw records — it judges cross-subject
 * relationships and framework claims.
 */
function buildOverviewSubjects(subjects: SubjectRecords[]): SubjectRecords[] {
  return subjects.map((s) => ({ subject: s.subject, records: [] }));
}

/**
 * Build a scoped fact_grounding question for an engine-specific batch.
 * Instructs Jev to judge only claims from this engine and treat
 * "no relevant claims" as grounded.
 */
function buildScopedGroundingQuestion(
  engineId: string,
  subjectNames: string[],
): Question {
  return score(
    `Rate how well this section is grounded in the supplied deterministic engine facts for ${subjectNames.length} subject(s): ${subjectNames.join(', ')}. ` +
    `Scope your judgement to claims attributable to the "${engineId}" engine only. ` +
    `Explicitly ignore claims that belong solely to other engines. ` +
    `If no claims in the section relate to this engine, treat that as fully grounded (score 3).`,
    [
      'No engine facts referenced; generic prose',
      'A few engine facts referenced for one subject only',
      'Engine facts referenced for every subject with some gaps',
      'Dense, specific engine facts for every subject woven through the section',
    ],
  );
}

/**
 * Copy the original questions, replacing only fact_grounding with the scoped version.
 */
function scopedQuestions(
  originalQuestions: Record<string, Question>,
  engineId: string,
  subjectNames: string[],
): Record<string, Question> {
  const q: Record<string, Question> = {};
  for (const [key, val] of Object.entries(originalQuestions)) {
    if (key === 'fact_grounding') {
      q[key] = buildScopedGroundingQuestion(engineId, subjectNames);
    } else {
      q[key] = val; // unchanged
    }
  }
  return q;
}

/**
 * Overview questions: same as original but fact_grounding scopes to
 * cross-subject relationships and framework claims.
 */
function overviewQuestions(
  originalQuestions: Record<string, Question>,
  subjectNames: string[],
): Record<string, Question> {
  const q: Record<string, Question> = {};
  for (const [key, val] of Object.entries(originalQuestions)) {
    if (key === 'fact_grounding') {
      q[key] = score(
        `Rate how well this section is grounded in the supplied deterministic engine facts for ${subjectNames.length} subject(s): ${subjectNames.join(', ')}. ` +
        `This is the overview batch: judge cross-subject relationship claims and attributed framework passages. ` +
        `The full compact prefix with all named summaries is present; use it for cross-subject grounding.`,
        [
          'No engine facts referenced; generic prose',
          'A few engine facts referenced for one subject only',
          'Engine facts referenced for every subject with some gaps',
          'Dense, specific engine facts for every subject woven through the section',
        ],
      );
    } else {
      q[key] = val;
    }
  }
  return q;
}

// ─── Conservative aggregation ────────────────────────────────────────

interface ParsedAnswers {
  guardrail_clean?: NoulResponse;
  relationship_framing_ok?: NoulResponse;
  fact_grounding?: ScoreResponse;
  register_fit?: ChoiceResponse;
}

function parseAnswers(answers: Record<string, unknown> | undefined): ParsedAnswers {
  if (!answers) return {};
  const out: ParsedAnswers = {};
  const g = answers.guardrail_clean;
  if (g && typeof g === 'object' && (g as any).type === 'noul' && typeof (g as any).noul === 'number' && Number.isFinite((g as any).noul)) {
    out.guardrail_clean = g as NoulResponse;
  }
  const f = answers.relationship_framing_ok;
  if (f && typeof f === 'object' && (f as any).type === 'noul' && typeof (f as any).noul === 'number' && Number.isFinite((f as any).noul)) {
    out.relationship_framing_ok = f as NoulResponse;
  }
  const s = answers.fact_grounding;
  if (s && typeof s === 'object' && (s as any).type === 'score' && typeof (s as any).score === 'number' && Number.isFinite((s as any).score) && typeof (s as any).confidence === 'number' && Number.isFinite((s as any).confidence)) {
    out.fact_grounding = s as ScoreResponse;
  }
  const r = answers.register_fit;
  if (r && typeof r === 'object' && (r as any).type === 'choice' && typeof (r as any).choice === 'string' && typeof (r as any).confidence === 'number' && Number.isFinite((r as any).confidence)) {
    out.register_fit = r as ChoiceResponse;
  }
  return out;
}

function validateParsedAnswers(
  parsed: ParsedAnswers,
  requiredKeys: string[],
  batchId: string,
): PartitionFailure | null {
  for (const key of requiredKeys) {
    const val = (parsed as any)[key];
    if (val === undefined) {
      return { kind: 'malformed_answer', message: `Missing or malformed answer for "${key}" in batch ${batchId}`, batchId };
    }
    if (key === 'guardrail_clean' || key === 'relationship_framing_ok') {
      const n = val as NoulResponse;
      if (n.noul < 0 || n.noul > 1) {
        return { kind: 'malformed_answer', message: `noul out of [0,1] for "${key}" in batch ${batchId}: ${n.noul}`, batchId };
      }
    }
    if (key === 'fact_grounding') {
      const s = val as ScoreResponse;
      if (s.score < 0 || s.score > 3) {
        return { kind: 'malformed_answer', message: `score out of [0,3] for "${key}" in batch ${batchId}: ${s.score}`, batchId };
      }
      if (s.confidence < 0 || s.confidence > 1) {
        return { kind: 'malformed_answer', message: `confidence out of [0,1] for "${key}" in batch ${batchId}: ${s.confidence}`, batchId };
      }
    }
    if (key === 'register_fit') {
      const r = val as ChoiceResponse;
      if (r.confidence < 0 || r.confidence > 1) {
        return { kind: 'malformed_answer', message: `confidence out of [0,1] for "${key}" in batch ${batchId}: ${r.confidence}`, batchId };
      }
    }
  }
  return null;
}

/**
 * Conservative aggregation across batches.
 * - noul: minimum (most conservative — worst safety)
 * - score: minimum score, minimum confidence
 * - choice: all must agree; min confidence; disagreement = explicit error
 */
function aggregateResults(
  batchResults: Array<{ batchId: string; result: JevResult }>,
  requestedKeys?: string[],
): { result?: JevResult; failure?: PartitionFailure } {
  const allParsed = batchResults.map((br) => ({
    batchId: br.batchId,
    parsed: parseAnswers(br.result.answers),
    model: br.result.model,
    usage: br.result.usage,
  }));

  // Validate every batch has all required keys
  const requiredKeys = ['guardrail_clean', 'fact_grounding', 'register_fit'];
  // relationship_framing_ok may or may not be present depending on input
  const hasFraming = allParsed.some((p) => p.parsed.relationship_framing_ok !== undefined);
  const keys = requestedKeys ?? (hasFraming ? [...requiredKeys, 'relationship_framing_ok'] : requiredKeys);
  if (!batchResults.length) return { failure: { kind: 'aggregation_error', message: 'No completed batches' } };

  for (const { batchId, parsed } of allParsed) {
    const failure = validateParsedAnswers(parsed, keys, batchId);
    if (failure) return { failure };
  }

  // noul minimum
  const guardrailValues = allParsed.map((p) => p.parsed.guardrail_clean!.noul);
  const minGuardrail = Math.min(...guardrailValues);

  let minFraming = 1;
  if (keys.includes('relationship_framing_ok')) {
    const framingValues = allParsed.map((p) => p.parsed.relationship_framing_ok?.noul ?? 1);
    minFraming = Math.min(...framingValues);
  }

  // score: minimum score, minimum confidence
  const scoreValues = allParsed.map((p) => p.parsed.fact_grounding!);
  const minScore = Math.min(...scoreValues.map((s) => s.score));
  const minScoreConfidence = Math.min(...scoreValues.map((s) => s.confidence));
  // Merge legends
  const mergedLegend: Record<string, string> = {};
  for (const s of scoreValues) {
    if (s.legend) Object.assign(mergedLegend, s.legend);
  }

  // choice: all must agree
  const choiceValues = allParsed.map((p) => p.parsed.register_fit!);
  const uniqueChoices = new Set(choiceValues.map((c) => c.choice));
  if (uniqueChoices.size > 1) {
    return {
      failure: {
        kind: 'aggregation_error',
        message: `Choice disagreement across batches: ${[...uniqueChoices].join(' vs ')}`,
        detail: { choices: choiceValues.map((c, i) => ({ batchId: allParsed[i].batchId, choice: c.choice, confidence: c.confidence })) },
      },
    };
  }
  const agreedChoice = choiceValues[0].choice;
  const minChoiceConfidence = Math.min(...choiceValues.map((c) => c.confidence));

  // model: list all actual models
  const models = [...new Set(allParsed.map((p) => p.model))];

  // usage: sum tokens
  const totalUsage = allParsed.reduce(
    (acc, p) => {
      if (p.usage) {
        acc.input_tokens += p.usage.input_tokens;
        acc.output_tokens += p.usage.output_tokens;
      }
      return acc;
    },
    { input_tokens: 0, output_tokens: 0 },
  );

  const aggregated: JevResult = {
    model: models.length === 1 ? models[0] : `partitioned(${models.join(', ')})`,
    answers: {
      guardrail_clean: { type: 'noul', noul: minGuardrail },
      fact_grounding: { type: 'score', score: minScore, legend: mergedLegend, probabilities: {}, confidence: minScoreConfidence },
      register_fit: { type: 'choice', choice: agreedChoice, probabilities: {}, confidence: minChoiceConfidence },
    },
    usage: totalUsage,
  };

  if (keys.includes('relationship_framing_ok')) {
    (aggregated.answers as any).relationship_framing_ok = { type: 'noul', noul: minFraming };
  }

  return { result: aggregated };
}

// ─── Coverage validation ─────────────────────────────────────────────

function buildCoverageIndex(subjects: SubjectRecords[]): Record<string, string[]> {
  const index: Record<string, string[]> = {};
  for (const s of subjects) {
    for (const r of s.records) {
      const key = `${s.subject}::${r.engine_id}`;
      index[key] = (index[key] ?? []).concat([r.engine_id]);
    }
  }
  return index;
}

function validateCoverage(
  subjects: SubjectRecords[],
  batches: Array<{ scope: string; subjects: SubjectRecords[] }>,
): PartitionFailure | null {
  const collect = (groups: SubjectRecords[]) => {
    const counts = new Map<string, number>();
    for (const subject of groups) for (const record of subject.records) {
      const key = JSON.stringify([subject.subject, record]);
      counts.set(key, (counts.get(key) ?? 0) + 1);
    }
    return counts;
  };
  const originals = collect(subjects);
  const actual = collect(batches.flatMap((batch) => batch.subjects));
  for (const [key, count] of originals) {
    if ((actual.get(key) ?? 0) < count) return { kind: "aggregation_error", message: "Missing coverage: " + key };
    if ((actual.get(key) ?? 0) > count) return { kind: "aggregation_error", message: "Duplicate coverage: " + key };
  }
  for (const key of actual.keys()) if (!originals.has(key)) return { kind: "aggregation_error", message: "Extra coverage not in original: " + key };

  return null;
}

// ─── Main adapter ────────────────────────────────────────────────────

export function createPartitionedJevClient(
  baseClient: JevClient,
  options: PartitionedJevClientOptions = {},
): JevClient {
  const maxRequestChars = options.maxRequestChars ?? DEFAULT_MAX_REQUEST_CHARS;

  return async (request: JevRequest): Promise<JevResult> => {
    // Only partition if state has engine_facts that contains the full records preamble.
    const state = request.state as Record<string, unknown> | undefined;
    const engineFacts = state?.engine_facts;
    if (typeof engineFacts !== 'string' || !engineFacts.includes(FULL_RECORDS_PREAMBLE)) {
      // No full records — pass through unchanged.
      return baseClient(request);
    }

    const originalRequestHash = sha256(JSON.stringify(request));
    const originalWireBodyHash = sha256(wireBody(request));
    const originalWireBodyBytes = Buffer.byteLength(wireBody(request));
    const { compactPrefix, fullRecordsPreamble, subjects } = parseEngineFacts(engineFacts);

    if (subjects.length === 0) {
      // Preamble present but no parseable records — pass through.
      return baseClient(request);
    }

    const subjectNames = (state?.subjects as string[]) ?? subjects.map((s) => s.subject);

    // Collect all unique engine_ids.
    const engineIds = [...new Set(subjects.flatMap((s) => s.records.map((r) => r.engine_id)))];

    // Build batches: one per engine_id + one overview.
    const batches: Array<{
      batchId: string;
      scope: string;
      engineFacts: string;
      questions: Record<string, Question>;
      subjects: SubjectRecords[];
    }> = [];

    // Overview batch: compact prefix only, no full records.
    const overviewSubjects = buildOverviewSubjects(subjects);
    const overviewEngineFacts = buildBatchEngineFacts(compactPrefix, fullRecordsPreamble, overviewSubjects);
    batches.push({
      batchId: 'overview',
      scope: 'overview',
      engineFacts: overviewEngineFacts,
      questions: overviewQuestions(request.questions, subjectNames),
      subjects: overviewSubjects,
    });

    // Per-engine batches.
    for (const engineId of engineIds) {
      const batchSubjects: SubjectRecords[] = [];
      for (const s of subjects) {
        const engineRecords = s.records.filter((r) => r.engine_id === engineId);
        {
          batchSubjects.push({ subject: s.subject, records: engineRecords });
        }
      }
      const batchEngineFacts = buildBatchEngineFacts(compactPrefix, fullRecordsPreamble, batchSubjects);
      batches.push({
        batchId: engineId,
        scope: engineId,
        engineFacts: batchEngineFacts,
        questions: scopedQuestions(request.questions, engineId, subjectNames),
        subjects: batchSubjects,
      });
    }

    // Validate coverage exactly-once.
    // Oversized engine groups are split only between complete source records.
    // Every shard retains the entire compact index, section, and framework
    // passages, so subject context and cross-system evidence remain visible.
    const boundedBatches: typeof batches = [];
    const batchSize = (batch: (typeof batches)[number]) => wireBody({
      state: { ...state, engine_facts: batch.engineFacts },
      questions: batch.questions,
      model: request.model,
    }).length;
    for (const batch of batches) {
      if (batch.scope === 'overview' || batchSize(batch) <= maxRequestChars) {
        boundedBatches.push(batch);
        continue;
      }
      let shardSubjects = buildOverviewSubjects(subjects);
      let shardIndex = 1;
      const makeShard = () => ({
        ...batch,
        batchId: `${batch.batchId}:${shardIndex}`,
        subjects: shardSubjects,
        engineFacts: buildBatchEngineFacts(compactPrefix, fullRecordsPreamble, shardSubjects),
      });
      for (const [subjectIndex, subject] of batch.subjects.entries()) {
        for (const record of subject.records) {
          shardSubjects[subjectIndex].records.push(record);
          if (batchSize(makeShard()) > maxRequestChars) {
            shardSubjects[subjectIndex].records.pop();
            if (shardSubjects.some((group) => group.records.length)) {
              boundedBatches.push(makeShard());
              shardIndex++;
              shardSubjects = buildOverviewSubjects(subjects);
            }
            shardSubjects[subjectIndex].records.push(record);
            // A single record exceeding the budget is retained intact; the
            // common size preflight below rejects it before any model call.
            if (batchSize(makeShard()) > maxRequestChars) {
              boundedBatches.push(makeShard());
              shardIndex++;
              shardSubjects = buildOverviewSubjects(subjects);
            }
          }
        }
      }
      if (shardSubjects.some((group) => group.records.length)) boundedBatches.push(makeShard());
    }
    batches.splice(0, batches.length, ...boundedBatches);
    const coverageFailure = validateCoverage(subjects, batches.filter((b) => b.scope !== 'overview'));
    if (coverageFailure) {
      const evidence: PartitionEvidence = {
        originalRequestHash,
        originalWireBodyHash,
        originalWireBodyBytes,
        originalRequest: request,
        batches: [],
        coverageIndex: buildCoverageIndex(subjects),
        failure: coverageFailure,
      };
      options.onEvidence?.(evidence);
      throw new Error(`Partition coverage validation failed: ${coverageFailure.message}`);
    }

    // Build batch requests and check sizes.
    const batchRequests: Array<{
      batchId: string;
      recordPartitionId: string;
      scope: string;
      subjects: SubjectRecords[];
      jevRequest: JevRequest;
    }> = [];

    for (const batch of batches) {
      // Reconstruct state for this batch: replace engine_facts, keep everything else.
      const batchState: Record<string, unknown> = { ...state };
      batchState.engine_facts = batch.engineFacts;

      const jevRequest: JevRequest = {
        state: batchState,
        questions: batch.questions,
        model: request.model,
      };

      const fullPassages = batchState.attributed_framework_passages;
      const frameworkSlices: JevRequest[] = [];
      if (wireBody(jevRequest).length > maxRequestChars && Array.isArray(fullPassages) && fullPassages.length) {
        const question = jevRequest.questions.fact_grounding;
        const sliceQuestions = question?.type === 'score' ? {
          ...jevRequest.questions,
          fact_grounding: { ...question, instructions: question.instructions + ' This request contains a complete-passage subset of the attributed framework evidence. Judge deterministic claims against the intact supplied records and framework claims only against the passages present in this slice. Do not penalize a claim solely because its framework passage belongs to another slice; all slices are judged and aggregated conservatively.' },
        } : jevRequest.questions;
        let current: unknown[] = [];
        const makeSlice = () => ({ ...jevRequest, questions: sliceQuestions, state: { ...batchState, attributed_framework_passages: current } });
        for (const passage of fullPassages) {
          current.push(passage);
          if (wireBody(makeSlice()).length > maxRequestChars) {
            current.pop();
            if (current.length) frameworkSlices.push(makeSlice());
            current = [passage];
          }
        }
        if (current.length) frameworkSlices.push(makeSlice());
      } else {
        frameworkSlices.push(jevRequest);
      }
      for (const [sliceIndex, sliceRequest] of frameworkSlices.entries()) {
      const sliceId = frameworkSlices.length > 1 ? `${batch.batchId}:framework:${sliceIndex + 1}` : batch.batchId;
      const size = wireBody(sliceRequest).length;
      if (size > maxRequestChars) {
        const failure: PartitionFailure = {
          kind: 'size_exceeded',
          message: `Batch "${sliceId}" serialized to ${size} chars, exceeding maxRequestChars ${maxRequestChars}. This batch is indivisible.`,
          batchId: sliceId,
        };
        const evidence: PartitionEvidence = {
          originalRequestHash,
          originalWireBodyHash,
          originalWireBodyBytes,
          originalRequest: request,
          batches: [{
            batchId: sliceId,
            recordPartitionId: batch.batchId,
            scope: batch.scope,
            subjects: batch.subjects.map((s) => s.subject),
            engineIds: batch.scope === 'overview' ? [] : [batch.scope],
            requestHash: sha256(JSON.stringify(sliceRequest)),
            wireBodyHash: sha256(wireBody(sliceRequest)),
            wireBodyBytes: Buffer.byteLength(wireBody(sliceRequest)),
            request: sliceRequest,
            error: failure.message,
          }],
          coverageIndex: buildCoverageIndex(subjects),
          failure,
        };
        options.onEvidence?.(evidence);
        throw new Error(`Partition size check failed: ${failure.message}`);
      }

      batchRequests.push({
        batchId: sliceId,
        recordPartitionId: batch.batchId,
        scope: batch.scope,
        subjects: batch.subjects,
        jevRequest: sliceRequest,
      });
      }
    }

    // Execute all batches. Preserve completed receipts on failure.
    const batchEvidence: BatchEvidence[] = [];
    const completedResults: Array<{ batchId: string; result: JevResult }> = [];
    let firstError: { batchId: string; error: unknown } | null = null;

    for (const br of batchRequests) {
      const requestHash = sha256(JSON.stringify(br.jevRequest));
      const be: BatchEvidence = {
        batchId: br.batchId,
        recordPartitionId: br.recordPartitionId,
        scope: br.scope,
        subjects: br.subjects.map((s) => s.subject),
        engineIds: br.scope === 'overview' ? [] : [br.scope],
        requestHash,
        wireBodyHash: sha256(wireBody(br.jevRequest)),
        wireBodyBytes: Buffer.byteLength(wireBody(br.jevRequest)),
        request: br.jevRequest,
      };

      try {
        const result = await baseClient(br.jevRequest);
        be.response = result;
        completedResults.push({ batchId: br.batchId, result });
      } catch (err) {
        be.error = err instanceof Error ? err.message : String(err);
        if (!firstError) firstError = { batchId: br.batchId, error: err };
      }

      batchEvidence.push(be);
    }

    // If any batch failed, emit evidence with completed receipts and throw.
    if (firstError) {
      // Still attempt aggregation of whatever completed, for evidence.
      let partialAgg: JevResult | undefined;
      let partialFailure: PartitionFailure | undefined;
      if (completedResults.length > 0) {
        const agg = aggregateResults(completedResults, Object.keys(request.questions));
        partialAgg = agg.result;
        partialFailure = agg.failure;
      }

      const failure: PartitionFailure = {
        kind: 'batch_error',
        message: `Batch "${firstError.batchId}" failed: ${firstError.error instanceof Error ? firstError.error.message : String(firstError.error)}`,
        batchId: firstError.batchId,
      };

      const evidence: PartitionEvidence = {
        originalRequestHash,
        originalWireBodyHash,
        originalWireBodyBytes,
        originalRequest: request,
        batches: batchEvidence,
        coverageIndex: buildCoverageIndex(subjects),
        aggregatedResult: partialAgg,
        failure,
      };
      options.onEvidence?.(evidence);

      // Re-throw the original error so the parent sees it.
      throw firstError.error instanceof Error ? firstError.error : new Error(String(firstError.error));
    }

    // Aggregate conservatively.
    const agg = aggregateResults(completedResults, Object.keys(request.questions));
    if (agg.failure) {
      const evidence: PartitionEvidence = {
        originalRequestHash,
        originalWireBodyHash,
        originalWireBodyBytes,
        originalRequest: request,
        batches: batchEvidence,
        coverageIndex: buildCoverageIndex(subjects),
        failure: agg.failure,
      };
      options.onEvidence?.(evidence);
      throw new Error(`Partition aggregation failed: ${agg.failure.message}`);
    }

    const evidence: PartitionEvidence = {
      originalRequestHash,
      originalWireBodyHash,
      originalWireBodyBytes,
      originalRequest: request,
      batches: batchEvidence,
      coverageIndex: buildCoverageIndex(subjects),
      aggregatedResult: agg.result,
    };
    options.onEvidence?.(evidence);

    return agg.result!;
  };
}

// ─── Exports for tests ───────────────────────────────────────────────

export const _internals = {
  sha256,
  parseEngineFacts,
  buildBatchEngineFacts,
  buildOverviewSubjects,
  buildScopedGroundingQuestion,
  scopedQuestions,
  overviewQuestions,
  aggregateResults,
  validateCoverage,
  buildCoverageIndex,
  parseAnswers,
  validateParsedAnswers,
  serializeSize,
  FULL_RECORDS_PREAMBLE,
  DEFAULT_MAX_REQUEST_CHARS,
};
