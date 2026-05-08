import GenericEngineView from "./GenericEngineView";

interface FaceReadingProps {
  result: Record<string, unknown>;
}

export default function FaceReading({ result }: FaceReadingProps) {
  return <GenericEngineView result={result} />;
}
