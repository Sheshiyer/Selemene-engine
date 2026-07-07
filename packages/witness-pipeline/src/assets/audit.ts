// ─── Asset Chain Audit ─────────────────────────────────────────────────
// Validates a source pack against deterministic-fact gates.

import type { SelemeneEngineOutput } from '../index.js';

export interface AuditInput {
  personId: string;
  readingMarkdown: string;
  engineResults: SelemeneEngineOutput[];
  deterministicOnly?: boolean;
}

export interface AuditResult {
  person_id: string;
  blockers: string[];
  warnings: string[];
  facts_count: number;
  passed: boolean;
}

export const DETERMINISTIC_ENGINES = new Set([
  'panchanga',
  'vimshottari',
  'human-design',
  'gene-keys',
  'numerology',
  'biorhythm',
  'vedic-clock',
  'transits',
  'enneagram',
]);

const ORACLE_ENGINES = new Set(['tarot', 'i-ching', 'sacred-geometry', 'sigil-forge']);
const SOMATIC_ENGINES = new Set(['biofield', 'face-reading', 'nadabrahman']);

export function runChainAudit(input: AuditInput): AuditResult {
  const blockers: string[] = [];
  const warnings: string[] = [];

  const deterministic = input.engineResults.filter(
    (e) => DETERMINISTIC_ENGINES.has(e.engine_id) && !e._error,
  );
  const factsCount = deterministic.length;

  if (factsCount < 3) {
    blockers.push(`Only ${factsCount} deterministic engines present; need at least 3.`);
  }

  if (input.deterministicOnly) {
    for (const e of input.engineResults) {
      if (ORACLE_ENGINES.has(e.engine_id)) {
        blockers.push(`Oracle engine ${e.engine_id} present but deterministic-only mode is enabled.`);
      }
      if (SOMATIC_ENGINES.has(e.engine_id)) {
        blockers.push(`Somatic engine ${e.engine_id} present but deterministic-only mode is enabled.`);
      }
    }
  }

  if (input.readingMarkdown.length < 100) {
    warnings.push('Reading markdown is very short.');
  }

  return {
    person_id: input.personId,
    blockers,
    warnings,
    facts_count: factsCount,
    passed: blockers.length === 0,
  };
}

// ─── L0 Final Artifact Verification (post-PDF) ───────────────────────────

/**
 * The five canonical deterministic systems required for L0 Integrated Kundali.
 * These must be present (non-error) in the final engines for a complete report.
 */
export const CANONICAL_L0_FIVE = [
  'panchanga',
  'vimshottari',
  'human-design',
  'gene-keys',
  'transits',
] as const;

export interface L0FinalVerificationInput {
  engines: SelemeneEngineOutput[];
  readingMarkdown: string;
  pdfExists: boolean;
  readingMdExists: boolean;
  requireCitations?: boolean;
}

export interface L0FinalVerificationResult {
  passed: boolean;
  blockers: string[];
  details: {
    pdf_exists: boolean;
    reading_md_exists: boolean;
    canonical_five_present: string[];
    missing_canonical: string[];
    citation_counts?: Record<string, number>;
  };
}

/**
 * Post-render final gate for L0 flows.
 * - Confirms PDF + source-pack/reading.md exist.
 * - Confirms the five canonical deterministic engines are present and non-error.
 * - Optionally counts system-name citations in the reading markdown.
 * Use this after renderFolioPdf to make the 5-system check part of the normal flow
 * instead of an ad-hoc manual step.
 */
export function verifyL0FinalArtifacts(
  input: L0FinalVerificationInput,
): L0FinalVerificationResult {
  const blockers: string[] = [];
  const details: L0FinalVerificationResult['details'] = {
    pdf_exists: input.pdfExists,
    reading_md_exists: input.readingMdExists,
    canonical_five_present: [],
    missing_canonical: [],
  };

  if (!input.pdfExists) {
    blockers.push('report.pdf is missing');
  }
  if (!input.readingMdExists) {
    blockers.push('source-pack/reading.md is missing');
  }

  const present = input.engines
    .filter((e) => !e._error)
    .map((e) => e.engine_id);

  const missing = CANONICAL_L0_FIVE.filter((id) => !present.includes(id));
  details.canonical_five_present = CANONICAL_L0_FIVE.filter((id) => present.includes(id));
  details.missing_canonical = missing;

  if (missing.length > 0) {
    blockers.push(`Missing canonical deterministic engines: ${missing.join(', ')} (need all 5 for L0)`);
  }

  if (input.requireCitations) {
    const lower = (input.readingMarkdown || '').toLowerCase();
    const hits: Record<string, number> = {};
    const nameRes: Array<[string, RegExp]> = [
      ['panchanga', /\bpanchanga\b/g],
      ['vimshottari', /\bvimshottari\b/g],
      ['human-design', /\bhuman design\b/g],
      ['gene-keys', /\bgene key(s)?\b/g],
      ['transits', /\btransit(s)?\b/g],
    ];
    for (const [key, re] of nameRes) {
      hits[key] = ((lower.match(re) as RegExpMatchArray) || []).length;
    }
    details.citation_counts = hits;
    // Note: we count only; zero counts are allowed (the engine presence + rubric facts are the hard requirement).
  }

  return {
    passed: blockers.length === 0,
    blockers,
    details,
  };
}
