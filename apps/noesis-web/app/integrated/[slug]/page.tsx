// ─── Integrated reading view — Next.js page ─────────────────────────────
// Server Component: loads the witness-agents output from disk and feeds
// it to the client component for animated rendering.
//
// Route: /integrated/[slug]
// Example: /integrated/witnessalchemist-x-harshita-x-mohan-l3

import { notFound } from "next/navigation";
import { loadIntegratedReading, listAvailableReadings } from "@/lib/integrated/loader";
import { parseMarkdownBlocks, type Block } from "@/lib/integrated/parseBlocks";
import { IntegratedReadingView } from "./IntegratedReadingView";

interface PageProps {
  params: Promise<{ slug: string }>;
}

export default async function IntegratedReadingPage({ params }: PageProps) {
  const { slug } = await params;
  const reading = await loadIntegratedReading(slug);
  if (!reading) {
    notFound();
  }

  // Pre-process each pass's markdown into typed Block[] server-side so
  // tables, ⌬ ACT blockquotes, and prose all land at the client as a
  // structured stream rather than an opaque html string.
  const passesWithBlocks = reading.passes.map((p) => ({
    ...p,
    blocks: parseMarkdownBlocks(p.markdown) as Block[],
  }));

  return <IntegratedReadingView reading={{ ...reading, passes: passesWithBlocks }} />;
}

export async function generateStaticParams() {
  const slugs = await listAvailableReadings();
  return slugs.map((slug) => ({ slug }));
}

// Disable static generation in dev — we read from disk live
export const dynamic = "force-dynamic";
