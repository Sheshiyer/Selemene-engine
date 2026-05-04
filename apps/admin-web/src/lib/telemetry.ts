"use client";

export interface TelemetryEvent {
  name: string;
  props?: Record<string, unknown>;
  ts?: number;
}

const isDev = process.env.NODE_ENV === "development";

/**
 * Emit a structured telemetry event.
 *
 * In development: logs to console.debug.
 * In production: POSTs to /api/telemetry via navigator.sendBeacon (fire-and-forget,
 * survives page unloads) with a fetch fallback.
 */
export function trackEvent(name: string, props?: Record<string, unknown>): void {
  const event: TelemetryEvent = { name, props, ts: Date.now() };

  if (isDev) {
    console.debug("[telemetry]", event);
    return;
  }

  const payload = JSON.stringify(event);
  if (typeof navigator !== "undefined" && navigator.sendBeacon) {
    const blob = new Blob([payload], { type: "application/json" });
    navigator.sendBeacon("/api/telemetry", blob);
  } else {
    // Fallback for environments without sendBeacon (SSR guard, older browsers)
    void fetch("/api/telemetry", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: payload,
      keepalive: true
    }).catch(() => undefined);
  }
}

/** Report a Core Web Vital measurement. */
export function trackVital(metric: { name: string; value: number; id: string }): void {
  trackEvent("web_vital", { metric: metric.name, value: metric.value, id: metric.id });
}
