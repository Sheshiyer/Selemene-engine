interface ReadingDetailPageProps {
  params: Promise<{
    readingId: string;
  }>;
}

export default async function ReadingDetailPage({ params }: ReadingDetailPageProps) {
  const { readingId } = await params;

  return (
    <section className="biofield-panel">
      <p className="biofield-eyebrow">Reading detail scaffold</p>
      <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
        {readingId}
      </h2>
      <p className="biofield-copy">
        This route is reserved for the persisted reading detail contract. Metrics, quality panels, and artifact metadata will be bound here once the Noesis biofield routes are implemented.
      </p>
    </section>
  );
}
