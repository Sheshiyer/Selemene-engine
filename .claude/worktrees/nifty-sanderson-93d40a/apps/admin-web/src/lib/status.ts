export type StatusTone = "ok" | "warning" | "danger";

const WARNING_STATUSES = new Set(["degraded", "idle", "lagging", "ahead", "warning"]);
const DANGER_STATUSES = new Set([
  "unavailable",
  "failure",
  "failed",
  "error",
  "danger",
  "revoked",
  "locked"
]);

export function statusTone(status: string): StatusTone {
  const normalized = status.trim().toLowerCase();
  if (DANGER_STATUSES.has(normalized)) {
    return "danger";
  }
  if (WARNING_STATUSES.has(normalized)) {
    return "warning";
  }
  return "ok";
}

export function statusPillClass(status: string): string {
  return `pill ${statusTone(status)}`;
}
