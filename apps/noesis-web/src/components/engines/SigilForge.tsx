import GenericEngineView from "./GenericEngineView";

interface SigilForgeProps {
  result: Record<string, unknown>;
}

export default function SigilForge({ result }: SigilForgeProps) {
  return <GenericEngineView result={result} />;
}
