// ─── Integrated reading loader ──────────────────────────────────────────
// Server-side loader that reads the witness-agents output (markdown +
// metrics JSON + SVG) from disk and returns a structured object the
// client components can consume.
//
// Source layout (witness-agents produces):
//   /Volumes/madara/.../723/working/{slug}/.runs/{timestamp}/
//     ├─ composite-triad-{subjects}.md       (assembled markdown)
//     ├─ composite-triad-{subjects}.svg      (topology SVG)
//     ├─ metrics_composite-triad-{...}.json  (per-pass metrics + level info)
//     ├─ pass_alpha.md, pass_beta.md, ...   (per-pass markdown)
//
// We resolve the most-recent .runs/ dir for a given slug.

import { readdir, readFile, stat } from "node:fs/promises";
import { join } from "node:path";

const WORKING_ROOT =
  process.env.INTEGRATED_READING_ROOT ??
  "/Volumes/madara/2026/twc-vault/01-Projects/723/working";

export interface PassMetric {
  id: string;
  title: string;
  words: number;
  xrefs: number;
  target_words: number;
  latency_ms: number;
  model: string;
}

export interface IntegratedReading {
  slug: string;
  runTimestamp: string;
  mode: string;
  subjects: string[];
  consciousnessLevel: number;
  registerBand: "l1_l3" | "l4_l5";
  totalWords: number;
  totalXrefs: number;
  targetWordsBand: { min: number; max: number };
  passes: Array<PassMetric & { markdown: string }>;
  topologySvg: string;
}

/** Find the most-recent `.runs/<timestamp>/` directory inside `slugDir`. */
async function findLatestRunDir(slugDir: string): Promise<string> {
  const runsRoot = join(slugDir, ".runs");
  const entries = await readdir(runsRoot, { withFileTypes: true });
  const tsDirs = entries
    .filter((e) => e.isDirectory() && /^\d{4}-/.test(e.name))
    .map((e) => e.name)
    .sort()
    .reverse();
  if (tsDirs.length === 0) throw new Error(`No timestamped runs in ${runsRoot}`);
  return join(runsRoot, tsDirs[0]);
}

/** List all readings currently present on disk. */
export async function listAvailableReadings(): Promise<string[]> {
  try {
    const slugs = await readdir(WORKING_ROOT);
    return slugs.filter((s) => !s.startsWith("."));
  } catch {
    return [];
  }
}

/** Load a single reading by slug — returns null if not found. */
export async function loadIntegratedReading(slug: string): Promise<IntegratedReading | null> {
  const slugDir = join(WORKING_ROOT, slug);
  try {
    await stat(slugDir);
  } catch {
    return null;
  }

  const runDir = await findLatestRunDir(slugDir);
  const files = await readdir(runDir);

  // Locate the assembled markdown + metrics JSON + topology SVG
  const assembledMd = files.find((f) => f.endsWith(".md") && !/^pass_/.test(f));
  const metricsJson = files.find((f) => /^metrics_.+\.json$/.test(f));
  const topologySvgFile = files.find((f) => f.endsWith(".svg"));
  if (!assembledMd || !metricsJson) {
    throw new Error(`Run dir ${runDir} missing required files`);
  }

  const metricsRaw = await readFile(join(runDir, metricsJson), "utf-8");
  const metrics = JSON.parse(metricsRaw) as {
    pass_metrics: PassMetric[];
    total_words: number;
    total_xrefs: number;
    effective_consciousness_level?: number;
    register_band?: "l1_l3" | "l4_l5";
    target_words_band?: { min: number; max: number };
  };

  // Load per-pass markdown
  const passes: Array<PassMetric & { markdown: string }> = [];
  for (const m of metrics.pass_metrics) {
    const passFile = `pass_${m.id}.md`;
    let md = "";
    if (files.includes(passFile)) {
      md = await readFile(join(runDir, passFile), "utf-8");
    }
    passes.push({ ...m, markdown: md });
  }

  const topologySvg = topologySvgFile
    ? await readFile(join(runDir, topologySvgFile), "utf-8")
    : "";

  // Infer subjects + mode from slug + assembled filename
  const subjectsAndMode = inferSubjectsFromSlug(slug, assembledMd);

  return {
    slug,
    runTimestamp: runDir.split("/").pop() ?? "",
    mode: subjectsAndMode.mode,
    subjects: subjectsAndMode.subjects,
    consciousnessLevel: metrics.effective_consciousness_level ?? 5,
    registerBand: metrics.register_band ?? "l4_l5",
    totalWords: metrics.total_words,
    totalXrefs: metrics.total_xrefs,
    targetWordsBand: metrics.target_words_band ?? { min: 12000, max: 15000 },
    passes,
    topologySvg,
  };
}

/** Parse mode + subject names from slug + assembled filename. Convention:
 *    slug "witnessalchemist-x-harshita-x-mohan-l3" → mode inferred from
 *    assembled filename prefix ("composite-triad-..." → "composite-triad"),
 *    subjects are the slug parts split on "-x-" with the trailing band
 *    suffix (l1/l3/l5) stripped. */
function inferSubjectsFromSlug(
  slug: string,
  assembledFilename: string,
): { mode: string; subjects: string[] } {
  // mode = first two hyphenated tokens of the assembled file, e.g.
  // "composite-triad-witnessalchemist-..." → "composite-triad"
  const modeMatch = assembledFilename.match(/^([a-z-]+?-(?:dyad|triad|synastry|partners|penta|synergy))-/);
  const mode = modeMatch ? modeMatch[1] : "composite-triad";

  // Subjects: split on "-x-" then strip band suffix from last token
  const parts = slug.split(/-x-/);
  const last = parts[parts.length - 1];
  const cleanLast = last.replace(/-l[1-5]$/i, "");
  return {
    mode,
    subjects: [...parts.slice(0, -1), cleanLast].map((p) =>
      p.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" "),
    ),
  };
}
