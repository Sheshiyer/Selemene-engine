// ─── Depth Reading — /depth/[subject] ──────────────────────────────────
// Server Component: loads the subject's solo synthesis from witness-agents
// output on disk, maps the 11 reading parts to 11 of the 15 sections,
// and hands the structured data to the DepthReadingClient for 3D mount.

import { notFound } from "next/navigation";
import { loadSoloReading, listSoloReadings } from "@/lib/integrated/soloLoader";
import { buildSectionsForSubject, type SectionData } from "@/depth-reading/data/sections";
import { DepthReadingClient } from "@/depth-reading/DepthReadingClient";

interface PageProps {
  params: Promise<{ subject: string }>;
}

export default async function DepthSubjectPage({ params }: PageProps) {
  const { subject } = await params;
  const reading = await loadSoloReading(subject);
  if (!reading) notFound();

  const sections = buildSectionsForSubject(subject);

  // Map the witness-agents prose by section id. The solo synthesis gives
  // us 11 parts (passes[0..10]); index them onto part-1..part-11. The
  // cover / witness-layer / compendium / closing / quine sections use
  // their own placeholder text (or eventually distinct sources).
  const proseBySection: Record<string, string> = {};

  // Cover / opener / closing — short evocative text built from metadata
  proseBySection["cover"] = [
    `${reading.subjects.join(" · ")}`,
    "",
    "An integrated reading composed from sixteen mirrors.",
    "Sixteen lenses, four cardinal directions, one unrepeatable consciousness.",
    "",
    "Scroll down. Each plane is one chapter of the reading.",
    "Click any plane to open its full text.",
  ].join("\n");

  proseBySection["witness-layer"] = [
    "This reading is not predictive. It is a structured form of self-inquiry — sixteen engines synthesizing a single field, then dissolving themselves on the way out.",
    "",
    "The threshold question is the same for every chart: *what is this consciousness asking to become unable to need?*",
    "",
    "Hold that question loosely. The chapters that follow are mirrors, not prescriptions.",
  ].join("\n");

  proseBySection["compendium"] = [
    `**${reading.subjects[0]}**`,
    `Total words: ${reading.totalWords.toLocaleString()}`,
    `Cross-references: ${reading.totalXrefs.toLocaleString()}`,
    `Register: ${reading.registerBand}`,
    `Mode: ${reading.mode}`,
    "",
    "The chart at a glance. Lagna, Atmakaraka, Birth Nakshatra, current Mahadasha — wired into the depth gallery as one structural snapshot before the chapters open.",
  ].join("\n");

  // 11 parts → part-1..part-11
  reading.passes.slice(0, 11).forEach((pass, i) => {
    proseBySection[`part-${i + 1}`] = pass.markdown;
  });

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

  return (
    <DepthReadingClient sections={sections} proseBySection={proseBySection} />
  );
}

export async function generateStaticParams() {
  const subjects = await listSoloReadings();
  return subjects.map((subject) => ({ subject }));
}

export const dynamic = "force-dynamic";
