import { BiofieldReadingDetailPage } from "@/components/biofield-reading-detail-page";

interface ReadingDetailPageProps {
  params: Promise<{
    readingId: string;
  }>;
}

export default async function ReadingDetailPage({ params }: ReadingDetailPageProps) {
  const { readingId } = await params;

  return <BiofieldReadingDetailPage readingId={readingId} />;
}
