// ─── buildSectionsFromPayload — adapt any payload to depth-reading data ──
// Bridge between the backend's reading payload (daily-practice OR
// integrated-reading) and the depth-reading viewer's SectionData[]
// + proseBySection contract.
//
// Auto-detects payload shape:
//   - Integrated: `passes` array with ≥11 parts → full 15-section arc
//   - Daily: short witness_layer.inference body → 5-section mini-arc
//     (cover · witness · part-1 · closing · quine)
//
// The depth-reading viewer then filters out sections without prose, so
// the user sees exactly as many planes as the payload has content for.

import { buildSectionsForSubject, type SectionData } from "@/depth-reading/data/sections";
import type { ReadingPayload, ReadingPart } from "./payloadLoader";

export interface BuiltReading {
  sections: SectionData[];
  proseBySection: Record<string, string>;
  /** Display name from subject metadata, fallback "Subject". */
  subjectName: string;
  /** "integrated" | "daily" | "unknown" — for downstream UI hints. */
  shape: "integrated" | "daily" | "unknown";
}

/** Pull the body text out of a daily-practice witness_layer.inference,
 *  trying common field names. Returns "" if nothing found. */
function extractDailyInferenceText(payload: ReadingPayload): string {
  const inf = payload.witness_layer?.inference;
  if (!inf || typeof inf !== "object") return "";
  // Prefer markdown if present, then content/text/body
  const candidates = ["markdown", "content", "text", "body"];
  for (const key of candidates) {
    const v = (inf as Record<string, unknown>)[key];
    if (typeof v === "string" && v.trim()) return v.trim();
  }
  // Fallback: try response.text/response.markdown nested shape
  const resp = (inf as Record<string, unknown>)["response"];
  if (resp && typeof resp === "object") {
    for (const key of candidates) {
      const v = (resp as Record<string, unknown>)[key];
      if (typeof v === "string" && v.trim()) return v.trim();
    }
  }
  return "";
}

/** Friendly compendium snapshot from the subject metadata. */
function buildCompendiumProse(payload: ReadingPayload): string {
  const s = payload.subject ?? {};
  const lines: string[] = [];
  if (s.name) lines.push(`**${s.name}**`);
  if (s.birth_date) lines.push(`Birth: ${s.birth_date}${s.birth_time ? " · " + s.birth_time : ""}`);
  if (s.location_label) lines.push(`Place: ${s.location_label}`);
  if (s.timezone) lines.push(`Timezone: ${s.timezone}`);
  lines.push("");
  lines.push(
    "The chart at a glance. Lagna, Atmakaraka, birth Nakshatra, current dasha — wired into the gallery as one structural snapshot before the chapters open.",
  );
  return lines.join("\n");
}

/** Build a section set + prose map from any reading payload. */
export function buildSectionsFromPayload(payload: ReadingPayload): BuiltReading {
  const subjectName = payload.subject?.name?.trim() || "Subject";
  const passes: ReadingPart[] = Array.isArray(payload.passes) ? payload.passes : [];
  const isIntegrated = passes.length >= 11;
  const shape: BuiltReading["shape"] = isIntegrated
    ? "integrated"
    : passes.length > 0 || payload.witness_layer
    ? "daily"
    : "unknown";

  // Start from the full 15-section template. The DepthReadingClient will
  // simply skip rendering planes whose section.id has no prose.
  const allSections = buildSectionsForSubject(subjectName);
  const proseBySection: Record<string, string> = {};

  // ─── Always-present opener sections ───────────────────────────────────
  proseBySection["cover"] = [
    subjectName,
    "",
    isIntegrated
      ? "An integrated reading composed from sixteen mirrors."
      : "A witness reading — the field caught in a single pass.",
    "",
    "Scroll down. Each plane is one chapter of the reading.",
    "Click any plane to open its full text.",
  ].join("\n");

  proseBySection["witness-layer"] = [
    "This reading is not predictive. It is a structured form of self-inquiry — engines synthesizing a single field, then dissolving themselves on the way out.",
    "",
    "Hold the threshold question loosely: *what is this consciousness asking to become unable to need?*",
    "",
    "The chapters that follow are mirrors, not prescriptions.",
  ].join("\n");

  proseBySection["compendium"] = buildCompendiumProse(payload);

  // ─── Parts: integrated maps passes[0..10] to part-1..part-11.
  //     Daily injects the single inference body into part-1 and stops. ──
  if (isIntegrated) {
    passes.slice(0, 11).forEach((pass, i) => {
      proseBySection[`part-${i + 1}`] = pass.markdown;
    });
  } else {
    const inferenceText = extractDailyInferenceText(payload);
    if (inferenceText) {
      proseBySection["part-1"] = inferenceText;
    }
    // Allow daily to also include any explicit "passes" if 1-3 short ones
    // are returned (some daily workflows do this for multi-witness reads)
    passes.slice(0, 4).forEach((pass, i) => {
      proseBySection[`part-${i + 1}`] = pass.markdown;
    });
  }

  // ─── Always-present closer sections ───────────────────────────────────
  proseBySection["closing"] = [
    "The reading ends where it began — with you, holding a slightly different shape than when you started.",
    "",
    "Everything you read above was already true about you. The chapters didn't add anything — they exposed structure.",
    "",
    "What you do with the exposure is the rest of the practice.",
  ].join("\n");

  proseBySection["quine"] = [
    "*The Quine principle: the system succeeds when you no longer need it.*",
    "",
    "If this reading made you feel like you need to come back for another one tomorrow, it failed.",
    "If it gave you a structural recognition you can carry forward without consulting it again, it succeeded.",
    "",
    "Anti-dependency is the test. Coherence is the proof. Body is the medium. Breath is the interface.",
    "",
    "∴",
  ].join("\n");

  // Filter to only sections that have prose (so daily shows 5 planes,
  // integrated shows 15). Preserves arc order.
  const visibleSections = allSections.filter((s) => proseBySection[s.id]);

  return {
    sections: visibleSections,
    proseBySection,
    subjectName,
    shape,
  };
}
