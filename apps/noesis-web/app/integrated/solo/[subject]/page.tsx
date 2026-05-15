// ─── Solo Integrated Reading — route /integrated/solo/[subject] ─────────
// Loads a single-subject solo synthesis from witness-agents output on
// disk and renders it through the integrated view as 11 parts (per the
// L1-L3 Vedic Kundali baseline structure: Opening + Part I-XI).
//
// Examples:
//   /integrated/solo/witnessalchemist
//   /integrated/solo/chitra
//   /integrated/solo/harshita

import { notFound } from "next/navigation";
import { loadSoloReading, listSoloReadings } from "@/lib/integrated/soloLoader";
import { parseMarkdownBlocks, type Block } from "@/lib/integrated/parseBlocks";
import { IntegratedReadingView } from "../../[slug]/IntegratedReadingView";

interface PageProps {
  params: Promise<{ subject: string }>;
}

export default async function SoloReadingPage({ params }: PageProps) {
  const { subject } = await params;
  const reading = await loadSoloReading(subject);
  if (!reading) notFound();

  // Pre-process each Part's markdown into typed Block[] server-side.
  const passesWithBlocks = reading.passes.map((p) => ({
    ...p,
    blocks: parseMarkdownBlocks(p.markdown) as Block[],
  }));

  return (
    <IntegratedReadingView
      reading={{
        ...reading,
        passes: passesWithBlocks,
      }}
    />
  );
}

export async function generateStaticParams() {
  const subjects = await listSoloReadings();
  return subjects.map((subject) => ({ subject }));
}

export const dynamic = "force-dynamic";
