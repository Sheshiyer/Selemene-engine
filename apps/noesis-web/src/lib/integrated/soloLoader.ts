// ─── Solo reading loader ────────────────────────────────────────────────
// Reads a single-subject solo synthesis from the witness-agents output
// (e.g. /Volumes/madara/.../723/{subject}-reading/.runs/{ts}/06_synthesis_{slug}.md)
// and splits it on the `## Part [IVX]+` headings into the 11 traditional
// Vedic Kundali parts. Each part becomes a PassMetric that the existing
// IntegratedReadingView pipeline can render directly.
//
// Solo synthesis structure (witness-agents L1-L3 baseline):
//   ## Opening — A Note Before Reading
//   ## Part I — The Convergence Map
//   ## Part II — The Vedic Foundation
//   ## Part III — The Karmic Architecture (Past → Present)
//   ## Part IV — Career & Dharma
//   ## Part V — Wealth & Money
//   ## Part VI — Love, Marriage, Spouse
//   ## Part VII — Health & Energy Body
//   ## Part VIII — Family, Roots, Soul Lineage
//   ## Part IX — The Master Timeline
//   ## Part X — Practices & Anti-Dependency Architecture
//   ## Part XI — Final Synthesis

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";
import type { IntegratedReading, PassMetric } from "./loader";

const SOLO_ROOT =
  process.env.SOLO_READING_ROOT ?? "/Volumes/madara/2026/twc-vault/01-Projects/723";

/** Subject directories that have a solo synthesis on disk. */
const SUBJECT_FOLDERS: Record<string, { folder: string; displayName: string; slug: string }> = {
  witnessalchemist: {
    folder: "witnessalchemist-reading",
    displayName: "WitnessAlchemist",
    slug: "witnessalchemist",
  },
  harshita: {
    folder: "harshita-reading",
    displayName: "Harshita",
    slug: "harshita",
  },
  mohan: {
    folder: "mohan-reading",
    displayName: "Mohan Kumar V",
    slug: "mohan-kumar-v",
  },
  chitra: {
    folder: "chitra-reading",
    displayName: "Chitra Shivanagowda",
    slug: "chitra-shivanagowda",
  },
  varsha: {
    folder: "varsha-reading",
    displayName: "Varsha S",
    slug: "varsha-s",
  },
};

export interface SoloReading extends Omit<IntegratedReading, "passes" | "subjects" | "mode"> {
  mode: "solo";
  subjects: string[]; // [displayName]
  /** Parts split from the solo synthesis markdown */
  passes: Array<PassMetric & { markdown: string }>;
}

/** List available solo readings. */
export async function listSoloReadings(): Promise<string[]> {
  const available: string[] = [];
  for (const key of Object.keys(SUBJECT_FOLDERS)) {
    try {
      const md = await findLatestSoloSynthesis(key);
      if (md) available.push(key);
    } catch {
      /* skip */
    }
  }
  return available;
}

async function findLatestSoloSynthesis(subjectKey: string): Promise<string | null> {
  const meta = SUBJECT_FOLDERS[subjectKey];
  if (!meta) return null;
  const runsRoot = join(SOLO_ROOT, meta.folder, ".runs");
  let entries: string[];
  try {
    entries = await readdir(runsRoot);
  } catch {
    return null;
  }
  const tsDirs = entries.filter((e) => /^\d{4}-/.test(e)).sort().reverse();
  for (const ts of tsDirs) {
    const candidate = join(runsRoot, ts, `06_synthesis_${meta.slug}.md`);
    try {
      await stat(candidate);
      return candidate;
    } catch {
      /* try next */
    }
  }
  return null;
}

/** Roman numeral → arabic converter (handles I-XII) */
function fromRoman(roman: string): number {
  const m: Record<string, number> = { I: 1, V: 5, X: 10, L: 50, C: 100 };
  let total = 0;
  for (let i = 0; i < roman.length; i++) {
    const cur = m[roman[i]];
    const next = m[roman[i + 1]];
    if (next > cur) {
      total -= cur;
    } else {
      total += cur;
    }
  }
  return total;
}

/** Split the solo synthesis on `## Part [IVX]+` headings into PassMetric[]. */
function splitIntoParts(md: string): Array<PassMetric & { markdown: string }> {
  // Strip BOM / leading whitespace
  const text = md.replace(/^﻿/, "").trim();

  // Find each "## Part ROMAN — TITLE" heading and its body up to the next "## Part" heading
  const partRe = /^## Part ([IVX]+)\s*[—-]\s*([^\n]+?)\s*$/gm;
  const matches: Array<{ index: number; roman: string; title: string; arabic: number }> = [];
  let m: RegExpExecArray | null;
  while ((m = partRe.exec(text)) !== null) {
    matches.push({
      index: m.index,
      roman: m[1],
      title: m[2].trim(),
      arabic: fromRoman(m[1]),
    });
  }

  if (matches.length === 0) {
    // Fallback: treat the whole thing as one part
    return [
      {
        id: "monolith",
        title: "Integrated Solo Reading",
        words: text.split(/\s+/).filter(Boolean).length,
        xrefs: 0,
        target_words: 12000,
        latency_ms: 0,
        model: "synthesis",
        markdown: text,
      },
    ];
  }

  const parts: Array<PassMetric & { markdown: string }> = [];
  for (let i = 0; i < matches.length; i++) {
    const start = matches[i].index;
    const end = i + 1 < matches.length ? matches[i + 1].index : text.length;
    const body = text.slice(start, end).trim();
    const wordCount = body.split(/\s+/).filter(Boolean).length;
    // Rough cross-ref count — emphasized phrases inside the body
    const xrefs = (body.match(/\*\*[^*]+\*\*/g) || []).length;

    parts.push({
      id: matches[i].roman.toLowerCase(),
      title: matches[i].title,
      words: wordCount,
      xrefs,
      target_words: 1000,
      latency_ms: 0,
      model: "synthesis",
      markdown: body,
    });
  }
  return parts;
}

/** Load a solo reading by subject key. Returns null if not found. */
export async function loadSoloReading(subjectKey: string): Promise<SoloReading | null> {
  const meta = SUBJECT_FOLDERS[subjectKey];
  if (!meta) return null;

  const synthPath = await findLatestSoloSynthesis(subjectKey);
  if (!synthPath) return null;

  const md = await readFile(synthPath, "utf-8");
  const parts = splitIntoParts(md);
  const totalWords = parts.reduce((s, p) => s + p.words, 0);
  const totalXrefs = parts.reduce((s, p) => s + p.xrefs, 0);

  // Try to read the per-subject ingestion JSON for birth-meta if present
  const tsDirMatch = synthPath.match(/\.runs\/([^/]+)\//);
  const runTimestamp = tsDirMatch ? tsDirMatch[1] : "";

  return {
    slug: `solo-${subjectKey}`,
    runTimestamp,
    mode: "solo",
    subjects: [meta.displayName],
    consciousnessLevel: 5,
    registerBand: "l4_l5",
    totalWords,
    totalXrefs,
    targetWordsBand: { min: 9000, max: 15000 },
    passes: parts,
    topologySvg: "",
  };
}
