// ─── Deterministic Leakage Gate ──────────────────────────────────────
// Pure, no-LLM scanner that blocks reader-facing prose containing private
// pipeline data. Runs before Jev, before source-audit, before final
// verification — it is the cheapest, most reliable acceptance floor.
//
// Design principles:
//   - Deterministic regex patterns only. Same text in → same result out.
//   - Block severity terminates acceptance. Warn severity is surfaced but
//     never silently upgrades to block.
//   - Explicit allow-list for readable chart vocabulary the reader is
//     entitled to see: "Vedic Kundali", "Human Design", "Gene Keys",
//     "Vimshottari", "Panchanga", "Rahu", "Ketu", etc.
//   - Avoids false positives on ordinary prose braces/brackets: JSON
//     detection requires BOTH a brace/bracket AND at least one JSON-shaped
//     key marker (identifier + colon, quoted string on both sides, etc.).
//     A sentence containing "{note}" or a "[3]" citation marker will not
//     trigger the JSON block pattern.
//
// This module is imported by evidence-map.ts, rubric.ts (types only), and
// final-verification.ts (enforcement). It has no runtime dependencies
// beyond the standard string API.

export type LeakageSeverity = 'block' | 'warn';

export interface LeakageViolation {
  /** Stable pattern id (e.g. 'json_key', 'receipt_field', 'tool_name'). */
  pattern_id: string;
  /** Exact matched substring, trimmed to a bounded window. */
  matched_text: string;
  /** 1-based line number where the match starts. */
  line_number: number;
  severity: LeakageSeverity;
}

export interface LeakageGateResult {
  passed: boolean;
  violations: LeakageViolation[];
}

interface LeakagePattern {
  id: string;
  severity: LeakageSeverity;
  regex: RegExp;
  /** Optional post-match filter to eliminate false positives. */
  filter?: (match: RegExpExecArray, fullText: string) => boolean;
}

// ─── Readable-vocabulary allow list ──────────────────────────────────
// Reader is entitled to these terms in natural prose. Written as case-
// insensitive literal phrases; we do NOT allow the hyphenated engine ids
// (e.g. `human-design`, `vedic-kundali`) which are internal keys.
const READABLE_TERMS = [
  'Vedic Kundali',
  'Vedic Jyotish',
  'Human Design',
  'Gene Keys',
  'Vimshottari',
  'Panchanga',
  'Transits',
  'Biorhythm',
  'Enneagram',
  'Tarot',
  'I Ching',
  'Biofield',
  'Nadabrahman',
  'Numerology',
  'Face Reading',
  'Sacred Geometry',
  'Sigil Forge',
  'Vedic Clock',
  'Rahu-Ketu',
  'Sade Sati',
  'Nakshatra',
  'Lagna',
  'Mahadasha',
  'Antardasha',
  'Pratyantardasha',
];

// A window around each match kept in `matched_text`.
const MATCH_WINDOW = 80;

// ─── Block patterns ───────────────────────────────────────────────────
// Every block pattern must be justified against the corrective plan §2.4.

const BLOCK_PATTERNS: LeakagePattern[] = [
  // Cloudflare corpus IDs. `sw:` is a stable internal namespace.
  {
    id: 'cf_passage_id',
    severity: 'block',
    regex: /\bsw:[a-zA-Z0-9][a-zA-Z0-9._:-]{2,}/g,
  },

  // Hyphenated engine identifiers as bare tokens (internal keys, not readable names).
  {
    id: 'engine_id_token',
    severity: 'block',
    regex: /\b(vedic-kundali|human-design|gene-keys|vedic-clock|sacred-geometry|sigil-forge|face-reading|biofield|nadabrahman|numerology-full|panchanga|vimshottari|transits|biorhythm|enneagram|tarot|i-ching)\b/gi,
    // Skip the match if it is inside a fenced code block context (defensive).
    filter: (m, full) => !insideCodeFence(m.index, full),
  },

  // Receipt / audit field names embedded in prose.
  {
    id: 'receipt_field',
    severity: 'block',
    regex: /\b(output_hash|source_audit|passages_hash|engine_facts_hash|synthesis_input_hash|audit_input_hash|input_sha256|output_sha256|source_hashes|source_ids|primary_source_ids|cf_source_ids|synthesis_input_text|prior_section_content_hashes|repair_input_text)\b/g,
  },

  // Backend / tool names.
  {
    id: 'tool_name',
    severity: 'block',
    regex: /\b(native-rust|swiss-ephemeris|pyswisseph|humdes-authenticated-source|humdes-extractor|witness-wisdom-corpus|vectorize|cloudflare-vectorize)\b/gi,
  },

  // Pipeline internal identifiers as bare tokens (persona names are allowed
  // only as narrative characters; here we block them appearing beside pipeline
  // workflow words).
  {
    id: 'pipeline_internal',
    severity: 'block',
    regex: /\b(witness-dyad|pass-gate|partitioned-client|mode_matrix|rubric_gate|guardrail_gate|dyad_receipt|retrieval_receipt|final-verification|reference-execution|reference-source-audit|jev\s+judgment|Jev\s+verdict|Jev\s+block(?:ed|s|ing)?)\b/g,
  },

  // Prompt ordinals — model instructions leaked into prose.
  {
    id: 'prompt_ordinal',
    severity: 'block',
    regex: /\b(Quoted Passage \d+|Passage \d+|Retrieved passage \d*|REPORT_BEGIN|REPORT_END|SOURCES?[- ]?END|ENGINE[- ]?BLOCK)\b/g,
  },

  // Acceptance / review status labels.
  {
    id: 'review_status',
    severity: 'block',
    regex: /\b(finalAccepted|changes-required|malformed|insufficient|llm-error|shadow verdict|active verdict|coverage_gap_sections|deterministic_fact_gate|integrated_layering_gate)\b/g,
  },

  // JSON-shaped fragment in prose. Requires a brace/bracket AND at least
  // one quoted-key marker to avoid false positives on "{note}" or "[3]".
  {
    id: 'json_key_fragment',
    severity: 'block',
    // Match either:
    //   * an object literal with a quoted key: "foo":
    //   * an array of objects starting with {"foo"
    //   * a bare identifier followed by colon inside braces of >= 20 chars
    regex: /(\{[^{}\n]{0,200}"[A-Za-z_][A-Za-z0-9_-]*"\s*:[^{}\n]{0,200}\})|(\[\s*\{[^\]\n]{0,300}"[A-Za-z_][A-Za-z0-9_-]*"\s*:)/g,
  },

  // Bare receipt-style ISO 8601 with milliseconds and Z in non-narrative
  // positions. Narrative dates like "May 2026" pass. This targets
  // strings like `2026-09-30T04:33:11.014Z`.
  {
    id: 'receipt_timestamp',
    severity: 'block',
    regex: /\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z\b/g,
  },
];

// ─── Warn patterns ────────────────────────────────────────────────────

const WARN_PATTERNS: LeakagePattern[] = [
  // Backend / precision label commentary in prose.
  {
    id: 'backend_label',
    severity: 'warn',
    regex: /\b(backend|precision_achieved|precision label|source precision)\b/gi,
  },

  // Consciousness register vocabulary — internal design language.
  {
    id: 'register_label',
    severity: 'warn',
    regex: /\b(consciousness_level|register band|L1-L3|L4-L5|register\s+L[1-5])\b/g,
  },
];

// ─── Helpers ──────────────────────────────────────────────────────────

/** Compute (1-based) line number for a character offset within `text`. */
function lineNumberOf(offset: number, text: string): number {
  if (offset <= 0) return 1;
  let n = 1;
  for (let i = 0; i < offset && i < text.length; i++) {
    if (text.charCodeAt(i) === 10) n++;
  }
  return n;
}

/** True when `offset` sits inside a fenced code block (```). */
function insideCodeFence(offset: number, text: string): boolean {
  let fenceOpen = false;
  let i = 0;
  while (i < offset && i < text.length) {
    if (i + 2 < text.length && text[i] === '`' && text[i + 1] === '`' && text[i + 2] === '`') {
      fenceOpen = !fenceOpen;
      i += 3;
      continue;
    }
    i++;
  }
  return fenceOpen;
}

function windowedMatch(m: RegExpExecArray): string {
  const start = Math.max(0, m.index - 8);
  const end = Math.min(m.input.length, m.index + m[0].length + 8);
  const excerpt = m.input.slice(start, end).replace(/\s+/g, ' ').trim();
  return excerpt.length > MATCH_WINDOW ? excerpt.slice(0, MATCH_WINDOW) + '…' : excerpt;
}

/**
 * Strip readable-term occurrences from `text` before scanning so a
 * legitimate mention of "Human Design" or "Vedic Kundali" cannot trip
 * the hyphenated engine-id pattern below or the register/backend label
 * warnings. Replacement preserves character positions with spaces so
 * line numbers remain correct for the remaining scan.
 */
function maskReadableTerms(text: string): string {
  let masked = text;
  for (const term of READABLE_TERMS) {
    const rx = new RegExp(escapeRegex(term), 'gi');
    masked = masked.replace(rx, (m) => ' '.repeat(m.length));
  }
  return masked;
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function runPatterns(patterns: LeakagePattern[], text: string, original: string): LeakageViolation[] {
  const violations: LeakageViolation[] = [];
  for (const pat of patterns) {
    // Regex is stateful (g); reset lastIndex defensively.
    pat.regex.lastIndex = 0;
    let m: RegExpExecArray | null;
    while ((m = pat.regex.exec(text)) !== null) {
      if (pat.filter && !pat.filter(m, text)) continue;
      violations.push({
        pattern_id: pat.id,
        matched_text: windowedMatch({ ...m, input: original } as unknown as RegExpExecArray),
        line_number: lineNumberOf(m.index, original),
        severity: pat.severity,
      });
      // Guard against zero-width matches infinite-looping.
      if (m[0].length === 0) pat.regex.lastIndex++;
    }
  }
  return violations;
}

/**
 * Run the deterministic leakage gate against a public section output.
 *
 * @param content Reader-facing markdown/prose to scan.
 * @returns pass/fail result plus the ordered list of violations.
 */
export function leakageGate(content: string): LeakageGateResult {
  const masked = maskReadableTerms(content);
  const violations = [
    ...runPatterns(BLOCK_PATTERNS, masked, content),
    ...runPatterns(WARN_PATTERNS, masked, content),
  ];
  // Stable sort by (line, severity, pattern_id) for deterministic receipts.
  violations.sort((a, b) => {
    if (a.line_number !== b.line_number) return a.line_number - b.line_number;
    if (a.severity !== b.severity) return a.severity === 'block' ? -1 : 1;
    return a.pattern_id.localeCompare(b.pattern_id);
  });
  const blocked = violations.some(v => v.severity === 'block');
  return { passed: !blocked, violations };
}

/** Convenience: true iff no block-severity violations exist. */
export function leakageGatePassed(result: LeakageGateResult): boolean {
  return result.passed;
}

/** Extract just the block-severity subset for compact receipts. */
export function blockViolations(result: LeakageGateResult): LeakageViolation[] {
  return result.violations.filter(v => v.severity === 'block');
}
