// Lightweight event bus for surfacing 402 quota-exceeded errors to a
// global modal. Any module (typically src/lib/api.ts) can call
// emitQuotaExceeded(); the QuotaExceededHost component subscribes and
// renders the modal.

import type { QuotaExceededDetail } from "@/components/QuotaExceededModal";

type Listener = (detail?: QuotaExceededDetail) => void;
const listeners = new Set<Listener>();

export function emitQuotaExceeded(detail?: QuotaExceededDetail): void {
  for (const fn of listeners) fn(detail);
}

export function subscribeToQuotaExceeded(fn: Listener): () => void {
  listeners.add(fn);
  return () => {
    listeners.delete(fn);
  };
}
