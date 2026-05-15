// ─── parseBlocks — markdown AST post-processor ─────────────────────────
// Splits a markdown source into typed Block[] for downstream rendering.
//
// Per design v2 § 5.6, § 5.7, § 5.9: we detect Native-comparison tables
// (which become HexagonTrios), other tables (which become SigilCascades),
// and `> ⌬ ACT:` blockquotes (which become DecisionPlates). Everything
// else stays as html and flows through the existing VerseFlow.
//
// The walker uses marked's lexer (token stream) — NOT regex on HTML —
// because marked already gives us a clean AST including table cells with
// inline tokens preserved. We then re-render only the html-kind blocks
// via marked.parser().

import { marked, type Token, type Tokens } from "marked";
import { enhanceTermsInHtml } from "./enhanceTerms";
import { injectMicroYantraPlaceholders } from "./microYantraEnhance";

/** Subject-column header tokens that, when seen as the first column of a
 *  table, classify the table as a Native-comparison → HexagonTrio. */
const SUBJECT_COL_HEADERS = new Set([
  "native",
  "subject",
  "person",
  "member",
  "partner",
  "chart",
  "individual",
]);

export interface SigilCascadeEntry {
  term: string;
  value?: string;
  subEntries?: SigilCascadeEntry[];
}

export interface DecisionMarker {
  action: string;
  window?: string;
  date?: string;
  quote?: string;
}

export type Block =
  | { kind: "html"; html: string }
  | {
      kind: "hex-trio";
      subjects: string[];
      columns: string[];
      rows: string[][];
    }
  | { kind: "cascade"; entries: SigilCascadeEntry[] }
  | { kind: "decision"; marker: DecisionMarker };

// ─── helpers ────────────────────────────────────────────────────────────

/** Strip markdown emphasis/strong/links to extract a clean text label
 *  while preserving sensible spacing. */
function flattenInline(tokens: Token[] | undefined): string {
  if (!tokens) return "";
  return tokens
    .map((t) => {
      if (t.type === "text") return (t as Tokens.Text).text;
      if (t.type === "strong") return flattenInline((t as Tokens.Strong).tokens);
      if (t.type === "em") return flattenInline((t as Tokens.Em).tokens);
      if (t.type === "codespan") return (t as Tokens.Codespan).text;
      if (t.type === "link") return flattenInline((t as Tokens.Link).tokens);
      if (t.type === "del") return flattenInline((t as Tokens.Del).tokens);
      if (t.type === "br") return " ";
      if (t.type === "escape") return (t as Tokens.Escape).text;
      // For unknown inline types, fall back to the raw token text.
      return (t as { text?: string; raw?: string }).text ?? (t as { raw?: string }).raw ?? "";
    })
    .join("")
    .trim();
}

/** Decide whether a table is a Native × Vedic-dimension comparison. */
function isHexTrioCandidate(table: Tokens.Table): boolean {
  if (table.rows.length < 2 || table.rows.length > 6) return false;
  if (table.header.length < 2) return false;
  const firstHeader = flattenInline(table.header[0].tokens).toLowerCase().trim();
  // Strip any trailing parenthetical, dashes, etc.
  const normalized = firstHeader.replace(/[^a-z]/g, "");
  return SUBJECT_COL_HEADERS.has(normalized) ||
    [...SUBJECT_COL_HEADERS].some((s) => normalized.startsWith(s));
}

function tableToHexTrio(table: Tokens.Table): Extract<Block, { kind: "hex-trio" }> {
  // First column = subject names; remaining columns = dimensions
  const columns = table.header.slice(1).map((h) => flattenInline(h.tokens));
  const subjects: string[] = [];
  const rows: string[][] = [];
  for (const row of table.rows) {
    const cells = row.map((c) => flattenInline(c.tokens));
    subjects.push(cells[0] ?? "");
    rows.push(cells.slice(1));
  }
  return { kind: "hex-trio", subjects, columns, rows };
}

function tableToCascade(table: Tokens.Table): Extract<Block, { kind: "cascade" }> {
  // For non-Native tables we flatten each row into "col1 col2 ... → coln"
  // semantics: the first cell becomes the term, subsequent cells become
  // (column-header: value) sub-entries. This preserves the table's
  // information density without forcing a hex layout.
  const headers = table.header.map((h) => flattenInline(h.tokens));
  const entries: SigilCascadeEntry[] = table.rows.map((row) => {
    const cells = row.map((c) => flattenInline(c.tokens));
    const term = cells[0] ?? "";
    if (cells.length === 2) {
      return { term, value: cells[1] };
    }
    const subEntries: SigilCascadeEntry[] = [];
    for (let i = 1; i < cells.length; i++) {
      if (!cells[i]) continue;
      subEntries.push({
        term: headers[i] ?? "",
        value: cells[i],
      });
    }
    return { term, subEntries };
  });
  return { kind: "cascade", entries };
}

/** Try to extract a structured DecisionMarker from a `> ⌬ ACT:` blockquote.
 *  Recognised fields (any order, any line):
 *    ⌬ ACT:    <action>
 *    WINDOW:   <window>
 *    DATE:     <date>
 *    QUOTE:    <quote>
 *  Anything that looks like an italic line at the end is treated as the
 *  quote if no QUOTE: field is present. */
function blockquoteToDecision(bq: Tokens.Blockquote): Extract<Block, { kind: "decision" }> | null {
  const flat = flattenInline(bq.tokens).trim();
  // Must begin with the ⌬ ACT: marker
  const m = flat.match(/^[⌬◇⬡]\s*ACT\s*:?\s*([\s\S]+)$/i);
  if (!m) return null;

  const body = m[1];
  // Split into lines / segments using `;`, `·`, or actual newlines
  const segments = body.split(/[\n;·]|\s{3,}/).map((s) => s.trim()).filter(Boolean);

  let action = "";
  let window: string | undefined;
  let date: string | undefined;
  let quote: string | undefined;

  const FIELD_RE = /^(WINDOW|DATE|QUOTE|TIME|WHEN)\s*[:—-]\s*(.+)$/i;
  for (const seg of segments) {
    const fm = seg.match(FIELD_RE);
    if (fm) {
      const key = fm[1].toUpperCase();
      const val = fm[2].trim();
      if (key === "WINDOW" || key === "TIME") window = val;
      else if (key === "DATE" || key === "WHEN") date = val;
      else if (key === "QUOTE") quote = val;
    } else if (!action) {
      action = seg;
    } else if (!quote) {
      // Heuristic: italicised trailing segment counts as quote
      quote = seg.replace(/^["“”]|["“”]$/g, "");
    }
  }

  if (!action) return null;
  return { kind: "decision", marker: { action, window, date, quote } };
}

// ─── main entry point ───────────────────────────────────────────────────

/** Parse a markdown source into typed Block[]. */
export function parseMarkdownBlocks(md: string): Block[] {
  const tokens = marked.lexer(md, { gfm: true });
  const blocks: Block[] = [];

  // Buffer of consecutive html-rendered tokens so we don't emit a separate
  // html block per paragraph — keeps the VerseFlow's sentence-splitting
  // logic able to work on a coherent prose chunk.
  const htmlBuffer: Token[] = [];
  const flushHtml = () => {
    if (htmlBuffer.length === 0) return;
    // marked.parser expects a TokensList — i.e. a Token[] with a `links`
    // property. We re-attach the top-level links from the lexer result.
    const list = htmlBuffer.slice() as Token[] & { links: Record<string, { href: string; title: string | null }> };
    list.links = (tokens as unknown as { links?: Record<string, { href: string; title: string | null }> }).links ?? {};
    const rawHtml = marked.parser(list as never);
    // W6 § 5.10: wrap technical terms with [data-engine-term] markers
    // that VerseFlow hydrates into <EngineTermLink> click-targets.
    const linked = enhanceTermsInHtml(rawHtml);
    // W8 § 5.5: inject [data-micro-yantra] placeholder spans next to
    // common Vedic / HD / Gene-Keys terms. VerseFlow hydrates these
    // into real <MicroYantra> components on mount. Runs AFTER W6 so
    // its skip-list (tags, attrs, code blocks) already excludes the
    // engine-term wrappers' internals.
    const html = injectMicroYantraPlaceholders(linked);
    blocks.push({ kind: "html", html });
    htmlBuffer.length = 0;
  };

  for (const token of tokens) {
    if (token.type === "table") {
      flushHtml();
      const t = token as Tokens.Table;
      if (isHexTrioCandidate(t)) {
        blocks.push(tableToHexTrio(t));
      } else {
        blocks.push(tableToCascade(t));
      }
      continue;
    }
    if (token.type === "blockquote") {
      const decision = blockquoteToDecision(token as Tokens.Blockquote);
      if (decision) {
        flushHtml();
        blocks.push(decision);
        continue;
      }
    }
    htmlBuffer.push(token);
  }
  flushHtml();
  return blocks;
}
