// ─── parseBlocks — markdown → typed prose blocks ───────────────────────
// A small line-by-line block parser tailored to the synthesis markdown
// the witness-agents pipeline emits. We don't need full CommonMark —
// just the shapes that actually appear: headings, paragraphs, tables,
// lists, blockquotes, hr, code.
//
// The parser also classifies tables by shape so the renderer can pick
// the right geometric carrier (yantra-lattice / sigil-cascade /
// bento-trio / dasha-waveform).

export type ProseBlock =
  | { kind: "heading"; level: 1 | 2 | 3 | 4; text: string }
  | { kind: "paragraph"; text: string }
  | { kind: "blockquote"; lines: string[] }
  | { kind: "hr" }
  | { kind: "list"; ordered: boolean; items: string[] }
  | { kind: "code"; lang: string; text: string }
  | {
      kind: "table";
      headers: string[];
      rows: string[][];
      classification: TableClassification;
    };

export type TableClassification =
  | "yantra-lattice" // ≥4 cols, compact cells, comparison matrix shape
  | "sigil-cascade" // last col is long-prose (>15 words avg)
  | "bento-trio" // ≤4 rows, ≤2 cols, severity/count/score rollup
  | "dasha-waveform"; // cells contain date ranges or dasha lord names

const DASHA_LORDS = new Set([
  "rahu", "ketu", "sun", "moon", "mars", "mercury", "jupiter", "venus", "saturn",
]);

/** Decide which geometric carrier should render this table. */
export function classifyTable(
  headers: string[],
  rows: string[][],
): TableClassification {
  const nCols = headers.length;
  const nRows = rows.length;

  // 1. Bento Trio — short categorical rollups
  //    (Severity × Count, Score × Value, Status × Total, etc.)
  if (nRows <= 4 && nCols <= 2) {
    const headerJoined = headers.join(" ").toLowerCase();
    if (/count|severity|score|status|total|delta|tally/.test(headerJoined)) {
      return "bento-trio";
    }
    return "bento-trio";
  }

  // 2. Dasha Waveform — when any cell mentions dasha lords or date ranges
  const flat = rows.flat().join(" ").toLowerCase();
  const hasDashaLord = Array.from(DASHA_LORDS).some((l) =>
    new RegExp(`\\b${l}\\b`).test(flat),
  );
  const hasYearRange =
    /\b(19|20)\d{2}\s*[-–—]\s*(19|20)\d{2}\b/.test(flat) ||
    /\bmd\b|\bmahadasha\b|\bantardasha\b/.test(flat);
  if (hasDashaLord && hasYearRange) {
    return "dasha-waveform";
  }

  // 3. Sigil Cascade — when the LAST column has long-prose cells
  if (nCols >= 2) {
    const lastColWordCounts = rows
      .map((r) => (r[nCols - 1] ?? "").trim().split(/\s+/).filter(Boolean).length);
    const avgLastColWords =
      lastColWordCounts.reduce((s, n) => s + n, 0) / Math.max(1, lastColWordCounts.length);
    if (avgLastColWords > 15) {
      return "sigil-cascade";
    }
  }

  // 4. Yantra Lattice — compact N×M comparison matrix (default for wide tables)
  if (nCols >= 3) {
    return "yantra-lattice";
  }

  // Fallback — sigil cascade for narrow tables that aren't bento
  return "sigil-cascade";
}

// ── Parser internals ───────────────────────────────────────────────────

/** Tokenize a markdown table row "| a | b | c |" → ["a", "b", "c"] */
function splitTableRow(line: string): string[] {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((c) => c.trim());
}

/** Detect if a line is a table separator like "|---|---|---|". */
function isTableSeparator(line: string): boolean {
  const trimmed = line.trim();
  if (!trimmed.startsWith("|")) return false;
  // Each cell must match -+ optionally with : for alignment
  const cells = splitTableRow(trimmed);
  return cells.length > 0 && cells.every((c) => /^:?-+:?$/.test(c));
}

/** Parse the markdown source into typed blocks. */
export function parseProseBlocks(md: string): ProseBlock[] {
  if (!md) return [];
  // Normalize line endings, trim trailing whitespace per line
  const lines = md.replace(/\r\n/g, "\n").split("\n").map((l) => l.replace(/\s+$/, ""));
  const blocks: ProseBlock[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Blank line — skip
    if (line.trim() === "") {
      i++;
      continue;
    }

    // Horizontal rule
    if (/^---+\s*$/.test(line) || /^\*\*\*+\s*$/.test(line)) {
      blocks.push({ kind: "hr" });
      i++;
      continue;
    }

    // Heading
    const headingMatch = line.match(/^(#{1,4})\s+(.+)$/);
    if (headingMatch) {
      const level = headingMatch[1].length as 1 | 2 | 3 | 4;
      blocks.push({ kind: "heading", level, text: headingMatch[2].trim() });
      i++;
      continue;
    }

    // Code fence
    if (/^```/.test(line)) {
      const lang = line.replace(/^```/, "").trim();
      const codeLines: string[] = [];
      i++;
      while (i < lines.length && !/^```/.test(lines[i])) {
        codeLines.push(lines[i]);
        i++;
      }
      i++; // skip closing fence
      blocks.push({ kind: "code", lang, text: codeLines.join("\n") });
      continue;
    }

    // Blockquote (one or more contiguous "> …" lines)
    if (line.startsWith(">")) {
      const quoteLines: string[] = [];
      while (i < lines.length && lines[i].startsWith(">")) {
        quoteLines.push(lines[i].replace(/^>\s?/, ""));
        i++;
      }
      blocks.push({ kind: "blockquote", lines: quoteLines });
      continue;
    }

    // Table — line starts with "|" and the NEXT line is a separator
    if (line.trim().startsWith("|") && i + 1 < lines.length && isTableSeparator(lines[i + 1])) {
      const headers = splitTableRow(line);
      i += 2; // skip header + separator
      const rows: string[][] = [];
      while (i < lines.length && lines[i].trim().startsWith("|")) {
        rows.push(splitTableRow(lines[i]));
        i++;
      }
      blocks.push({
        kind: "table",
        headers,
        rows,
        classification: classifyTable(headers, rows),
      });
      continue;
    }

    // List (contiguous "- " or "* " or "1. " lines)
    const ulMatch = line.match(/^[-*+]\s+(.+)$/);
    const olMatch = line.match(/^\d+\.\s+(.+)$/);
    if (ulMatch || olMatch) {
      const ordered = !!olMatch;
      const items: string[] = [];
      while (i < lines.length) {
        const u = lines[i].match(/^[-*+]\s+(.+)$/);
        const o = lines[i].match(/^\d+\.\s+(.+)$/);
        if (ordered ? o : u) {
          items.push((ordered ? o![1] : u![1]).trim());
          i++;
        } else if (lines[i].trim() === "") {
          // Blank line inside a list — peek ahead; only break if next isn't a list item
          if (
            i + 1 < lines.length &&
            !lines[i + 1].match(/^[-*+]\s+/) &&
            !lines[i + 1].match(/^\d+\.\s+/)
          ) {
            break;
          }
          i++;
        } else {
          break;
        }
      }
      blocks.push({ kind: "list", ordered, items });
      continue;
    }

    // Paragraph — accumulate consecutive non-blank, non-special lines
    const paraLines: string[] = [];
    while (
      i < lines.length &&
      lines[i].trim() !== "" &&
      !lines[i].match(/^#{1,4}\s+/) &&
      !lines[i].startsWith(">") &&
      !/^```/.test(lines[i]) &&
      !lines[i].trim().startsWith("|") &&
      !lines[i].match(/^[-*+]\s+/) &&
      !lines[i].match(/^\d+\.\s+/) &&
      !/^---+\s*$/.test(lines[i])
    ) {
      paraLines.push(lines[i]);
      i++;
    }
    if (paraLines.length > 0) {
      blocks.push({ kind: "paragraph", text: paraLines.join(" ") });
    }
  }

  return blocks;
}

/** Render inline markdown (**bold**, *italic*, `code`) to HTML.
 *  Used by carriers + verse paragraphs. Escapes HTML first. */
export function renderInline(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/`([^`]+)`/g, '<code style="font-family:var(--font-mono,monospace);font-size:0.92em;opacity:0.92;">$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>")
    .replace(/(?<!\*)\*([^*]+)\*(?!\*)/g, "<em>$1</em>")
    .replace(/\b_([^_]+)_\b/g, "<em>$1</em>");
}

/** Split a long paragraph into sentence fragments — used by verse
 *  paragraphs to drive the 3-4 sentence focus-zone highlight. */
export function splitIntoSentences(text: string): string[] {
  return text
    .split(/(?<=[.!?])\s+(?=[A-Z“"'(])/)
    .map((s) => s.trim())
    .filter(Boolean);
}
