// ─── Integrated reading view — Next.js page ─────────────────────────────
// Server Component: loads the witness-agents output from disk and feeds
// it to the client component for animated rendering.
//
// Route: /integrated/[slug]
// Example: /integrated/witnessalchemist-x-harshita-x-mohan-l3

import { notFound } from "next/navigation";
import { marked } from "marked";
import { loadIntegratedReading, listAvailableReadings } from "@/lib/integrated/loader";
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

  // Pre-render each pass's markdown to HTML server-side
  const passesWithHtml = reading.passes.map((p) => ({
    ...p,
    html: marked.parse(p.markdown, { gfm: true, breaks: false }) as string,
  }));

  return <IntegratedReadingView reading={{ ...reading, passes: passesWithHtml }} />;
}

export async function generateStaticParams() {
  const slugs = await listAvailableReadings();
  return slugs.map((slug) => ({ slug }));
}

// Disable static generation in dev — we read from disk live
export const dynamic = "force-dynamic";
