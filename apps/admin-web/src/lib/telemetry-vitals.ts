import { trackVital } from "@/lib/telemetry";

// Shape emitted by Next.js reportWebVitals hook
interface NextWebVitalsMetric {
  id: string;
  name: string;
  startTime: number;
  value: number;
  label: "web-vital" | "custom";
}

export function reportWebVitals(metric: NextWebVitalsMetric): void {
  trackVital({ name: metric.name, value: metric.value, id: metric.id });
}
