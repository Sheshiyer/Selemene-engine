import GenericEngineView from "./GenericEngineView";

interface EnneagramProps {
  result: Record<string, unknown>;
}

export default function Enneagram({ result }: EnneagramProps) {
  return <GenericEngineView result={result} />;
}
