import GenericEngineView from "./GenericEngineView";

interface NadabrahmanProps {
  result: Record<string, unknown>;
}

export default function Nadabrahman({ result }: NadabrahmanProps) {
  return <GenericEngineView result={result} />;
}
