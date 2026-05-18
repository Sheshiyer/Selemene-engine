// ─── /r/[id] — depth-reading by reading-id (cross-app destination) ─────
// This route is where landing-page generated readings LAND. The user
// completes a workflow on `113.tryambakam.space`, the landing redirects
// to `https://48.tryambakam.space/r/{readingId}` (typically with the
// full payload encoded in the URL hash for zero-roundtrip load).
//
// Client Component (must read URL hash, which is browser-only). The
// page wrapper renders a static fallback during SSR/hydration, then
// once the payload resolves we mount the full DepthReadingClient.

import ReadingByIdClient from "./ReadingByIdClient";

interface PageProps {
  params: Promise<{ id: string }>;
}

export default async function Page({ params }: PageProps) {
  const { id } = await params;
  return <ReadingByIdClient readingId={id} />;
}

export const dynamic = "force-dynamic";
