import GenericEngineView from "./GenericEngineView";

interface SacredGeometryProps {
  result: Record<string, unknown>;
}

export default function SacredGeometry({ result }: SacredGeometryProps) {
  return <GenericEngineView result={result} />;
}
