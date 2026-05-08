import GenericEngineView from "./GenericEngineView";

interface VedicClockProps {
  result: Record<string, unknown>;
}

export default function VedicClock({ result }: VedicClockProps) {
  return <GenericEngineView result={result} />;
}
